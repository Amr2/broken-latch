use tauri::{AppHandle, WebviewWindow};
use windows::Win32::{
    Foundation::*,
    UI::WindowsAndMessaging::*,
    Graphics::Gdi::*,
};
use std::sync::{Arc, Mutex};

/// Manages a single transparent overlay window.
///
/// # Show / hide strategy
///
/// We NEVER call `ShowWindow(SW_HIDE)` / `ShowWindow(SW_SHOW)` after the
/// initial creation.  Toggling Win32 window visibility while a DirectX game
/// is in fullscreen causes the game to briefly lose its swap-chain ownership
/// (exit fullscreen), creating the "glitch / game minimises" bug.
///
/// Instead, we use **opacity-based soft hide/show**:
///   - `soft_hide` → `SetLayeredWindowAttributes(alpha=0)` + `WS_EX_TRANSPARENT`
///     The window is still "visible" to Win32 (no WS_VISIBLE change), fully
///     transparent to the eye, and fully click-through.
///   - `soft_show` → restore configured opacity + conditionally remove
///     `WS_EX_TRANSPARENT`
///
/// Because the window's Z-order never changes and its WS_VISIBLE flag never
/// changes, the DX swap chain is never disturbed — even in exclusive fullscreen.
pub struct OverlayWindow {
    window: WebviewWindow,
    /// Raw HWND stored as `isize` so `OverlayWindow` implements `Send + Sync`.
    hwnd_raw: isize,
    interactive_regions: Arc<Mutex<Vec<Rect>>>,
    /// Configured opacity (0.0–1.0) — restored on soft_show.
    config_opacity: f32,
    /// Whether this overlay is configured as click-through.
    /// When soft_show is called, WS_EX_TRANSPARENT is restored only if true.
    config_click_through: bool,
}

/// A screen-space rectangle used to describe an interactive widget area.
///
/// Coordinates are in pixels from the top-left corner of the primary monitor.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

// SAFETY: HWND values (stored as isize) are process-global handles that are
// safe to send/share across threads. The Tauri WebviewWindow owns the resource
// lifetime; we only store the raw numeric handle.
unsafe impl Send for OverlayWindow {}
unsafe impl Sync for OverlayWindow {}

impl OverlayWindow {
    fn hwnd(&self) -> HWND {
        HWND(self.hwnd_raw as *mut core::ffi::c_void)
    }

    /// Create a transparent overlay window.
    pub fn create(
        app: &AppHandle,
        label: &str,
        url: &str,
        width:  Option<u32>,
        height: Option<u32>,
        x: Option<i32>,
        y: Option<i32>,
        opacity: f32,
        click_through: bool,
        always_on_top: bool,
        screen_capture_visible: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (screen_w, screen_h) = unsafe {
            (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN))
        };

        let fullscreen = width.is_none() && height.is_none();
        let win_w = width.unwrap_or(screen_w as u32) as f64;
        let win_h = height.unwrap_or(screen_h as u32) as f64;

        let webview_url = make_webview_url(url);

        let mut builder = tauri::WebviewWindowBuilder::new(app, label, webview_url)
            .transparent(true)
            .decorations(false)
            .always_on_top(always_on_top)
            .skip_taskbar(true)
            .inner_size(win_w, win_h)
            .visible(false);  // shown below via SW_SHOWNOACTIVATE

        builder = match (x, y) {
            (Some(px), Some(py)) => builder.position(px as f64, py as f64),
            _ if fullscreen      => builder.position(0.0, 0.0),
            _                    => builder.center(),
        };

        let window = builder.build()?;

        let hwnd     = window.hwnd()?;
        let hwnd_raw = hwnd.0 as isize;

        apply_overlay_styles(hwnd, screen_capture_visible, click_through, always_on_top);

        let ov = Self {
            window,
            hwnd_raw,
            interactive_regions: Arc::new(Mutex::new(Vec::new())),
            config_opacity: opacity,
            config_click_through: click_through,
        };

        // Show without activating — SW_SHOWNOACTIVATE leaves the foreground
        // window unchanged so we never steal focus from the game.
        unsafe { ShowWindow(ov.hwnd(), SW_SHOWNOACTIVATE); }

        if let Err(e) = ov.set_opacity(opacity) {
            eprintln!("[overlay] set_opacity failed (non-fatal): {}", e);
        }

        Ok(ov)
    }

    // -----------------------------------------------------------------------
    // Soft show / hide  (opacity-based, Z-order unchanged)
    // -----------------------------------------------------------------------

    /// Hide the overlay without touching Win32 window visibility.
    ///
    /// Sets opacity to 0 and adds WS_EX_TRANSPARENT so the window is
    /// both invisible and fully click-through.  The window's TOPMOST
    /// Z-order is preserved so no DX fullscreen disruption occurs.
    pub fn soft_hide(&self) {
        unsafe {
            // Make fully transparent
            let _ = SetLayeredWindowAttributes(self.hwnd(), COLORREF(0), 0, LWA_ALPHA);
            // Ensure click-through so invisible window doesn't block input
            let ex_style = GetWindowLongW(self.hwnd(), GWL_EXSTYLE);
            SetWindowLongW(self.hwnd(), GWL_EXSTYLE,
                ex_style | (WS_EX_TRANSPARENT.0 as i32));
        }
    }

    /// Restore the overlay to its configured opacity.
    ///
    /// Removes WS_EX_TRANSPARENT if the overlay is not configured as
    /// click-through (so interactive widgets can receive input again).
    /// Never changes TOPMOST Z-order.
    pub fn soft_show(&self) {
        unsafe {
            // Restore configured opacity
            let byte = (self.config_opacity.clamp(0.0, 1.0) * 255.0) as u8;
            let _ = SetLayeredWindowAttributes(self.hwnd(), COLORREF(0), byte, LWA_ALPHA);
            // Restore click-through state
            let ex_style = GetWindowLongW(self.hwnd(), GWL_EXSTYLE);
            let new_style = if self.config_click_through {
                ex_style | (WS_EX_TRANSPARENT.0 as i32)
            } else {
                ex_style & !(WS_EX_TRANSPARENT.0 as i32)
            };
            SetWindowLongW(self.hwnd(), GWL_EXSTYLE, new_style);
        }
    }

    // -----------------------------------------------------------------------
    // Other controls
    // -----------------------------------------------------------------------

    /// Define which screen regions intercept mouse input.
    pub fn set_interactive_regions(&self, regions: Vec<Rect>) -> Result<(), Box<dyn std::error::Error>> {
        *self.interactive_regions.lock().unwrap() = regions.clone();

        if regions.is_empty() {
            return self.set_click_through(true);
        }

        let hwnd = self.hwnd();
        unsafe {
            let combined_region = CreateRectRgn(0, 0, 0, 0);
            for rect in regions {
                let widget_region = CreateRectRgn(
                    rect.x, rect.y,
                    rect.x + rect.width,
                    rect.y + rect.height,
                );
                CombineRgn(
                    Some(combined_region),
                    Some(combined_region),
                    Some(widget_region),
                    RGN_OR,
                );
                let _ = DeleteObject(widget_region.into());
            }
            let _ = SetWindowRgn(hwnd, Some(combined_region), true);
            self.set_click_through(false)?;
        }
        Ok(())
    }

    pub fn get_interactive_regions(&self) -> Vec<Rect> {
        self.interactive_regions.lock().unwrap().clone()
    }

    /// Toggle global click-through mode for the entire window.
    pub fn set_click_through(&self, enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
        let hwnd = self.hwnd();
        unsafe {
            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            let new_style = if enabled {
                ex_style | (WS_EX_TRANSPARENT.0 as i32)
            } else {
                ex_style & !(WS_EX_TRANSPARENT.0 as i32)
            };
            SetWindowLongW(hwnd, GWL_EXSTYLE, new_style);
        }
        Ok(())
    }

    /// Hard show via Tauri (only used outside of focus transitions).
    pub fn show(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.window.show()?;
        Ok(())
    }

    /// Hard hide via Tauri (only used when closing a phase, not for focus).
    pub fn hide(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.window.hide()?;
        Ok(())
    }

    pub fn close(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.window.close()?;
        Ok(())
    }

    /// Set opacity via `SetLayeredWindowAttributes(LWA_ALPHA)`.
    pub fn set_opacity(&self, opacity: f32) -> Result<(), Box<dyn std::error::Error>> {
        let byte = (opacity.clamp(0.0, 1.0) * 255.0) as u8;
        unsafe {
            SetLayeredWindowAttributes(self.hwnd(), COLORREF(0), byte, LWA_ALPHA)?;
        }
        Ok(())
    }

    /// Toggle OBS/Discord/Windows-Game-Bar capture visibility.
    pub fn set_screen_capture_visible(&self, visible: bool) -> Result<(), Box<dyn std::error::Error>> {
        let affinity = if visible { WDA_NONE } else { WDA_EXCLUDEFROMCAPTURE };
        unsafe {
            SetWindowDisplayAffinity(self.hwnd(), affinity)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Apply Win32 extended styles required for overlay behaviour.
///
/// Each call is isolated — failures are logged but never abort window creation.
fn apply_overlay_styles(
    hwnd: HWND,
    screen_capture_visible: bool,
    click_through: bool,
    always_on_top: bool,
) {
    unsafe {
        // WS_EX_LAYERED  → required for SetLayeredWindowAttributes (opacity)
        // WS_EX_TRANSPARENT → click-through (passes input to underlying window)
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        let mut new_style = ex_style | (WS_EX_LAYERED.0 as i32);
        if click_through {
            new_style |= WS_EX_TRANSPARENT.0 as i32;
        }
        SetWindowLongW(hwnd, GWL_EXSTYLE, new_style);

        // Z-order — SWP_NOACTIVATE avoids ERROR_NOT_ENOUGH_MEMORY from
        // Windows needing activation-context kernel structures.
        let z_order = if always_on_top { HWND_TOPMOST } else { HWND_NOTOPMOST };
        if let Err(e) = SetWindowPos(
            hwnd, Some(z_order),
            0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        ) {
            eprintln!("[overlay] SetWindowPos z-order failed (non-fatal): {}", e);
        }

        // WDA_EXCLUDEFROMCAPTURE requires Windows 10 2004+; degrade gracefully.
        let affinity = if screen_capture_visible { WDA_NONE } else { WDA_EXCLUDEFROMCAPTURE };
        if let Err(e) = SetWindowDisplayAffinity(hwnd, affinity) {
            eprintln!("[overlay] SetWindowDisplayAffinity failed (non-fatal): {}", e);
        }
    }
}

pub fn make_webview_url(url: &str) -> tauri::WebviewUrl {
    if url.starts_with("http://") || url.starts_with("https://") {
        tauri::WebviewUrl::External(url.parse().expect("invalid external URL"))
    } else {
        tauri::WebviewUrl::App(url.into())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_creation() {
        let rect = Rect { x: 100, y: 200, width: 300, height: 400 };
        assert_eq!(rect.x, 100);
        assert_eq!(rect.width, 300);
    }

    #[test]
    fn test_webview_url_external() {
        match make_webview_url("https://example.com") {
            tauri::WebviewUrl::External(_) => {}
            _ => panic!("expected External"),
        }
    }

    #[test]
    fn test_webview_url_app() {
        match make_webview_url("overlay.html") {
            tauri::WebviewUrl::App(_) => {}
            _ => panic!("expected App"),
        }
    }
}
