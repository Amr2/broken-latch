use tauri::{AppHandle, WebviewWindow};
use windows::Win32::{
    Foundation::*,
    UI::WindowsAndMessaging::*,
    Graphics::Gdi::*,
};
use std::sync::{Arc, Mutex};

/// Manages the transparent fullscreen overlay window.
///
/// # Win32 window properties applied after creation
/// - `WS_EX_LAYERED`          — enables per-pixel alpha & opacity control
/// - `WS_EX_TRANSPARENT`      — default click-through (passes input to game)
/// - `WS_EX_TOPMOST`          — z-order above all other windows
/// - `WDA_EXCLUDEFROMCAPTURE` — hidden from OBS/Discord screen capture
pub struct OverlayWindow {
    window: WebviewWindow,
    /// Raw HWND stored as `isize` so `OverlayWindow` implements `Send + Sync`.
    ///
    /// Win32 HWNDs are process-global handles that can safely cross thread
    /// boundaries. Tauri's `WebviewWindow` owns the actual resource lifetime;
    /// we just keep the handle value for Win32 API calls.
    hwnd_raw: isize,
    interactive_regions: Arc<Mutex<Vec<Rect>>>,
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
    /// Reconstruct the HWND from the stored raw value.
    fn hwnd(&self) -> HWND {
        HWND(self.hwnd_raw as *mut core::ffi::c_void)
    }

    /// Create and configure the transparent fullscreen overlay window.
    ///
    /// NOTE on `always_on_top` / `fullscreen` in the Tauri builder:
    ///   We set these to `false` here and apply the Win32 `WS_EX_TOPMOST` style
    ///   directly via `SetWindowPos(HWND_TOPMOST)` in `apply_overlay_styles`.
    ///   This avoids the Tauri builder applying TOPMOST *before* the window is
    ///   fully created, which can interfere with the Dev Simulator window focus.
    ///   In production the window is also sized to the screen via Win32 after creation.
    pub fn create(app: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        // Get primary monitor dimensions so we can size the overlay correctly
        let (screen_w, screen_h) = unsafe {
            (
                GetSystemMetrics(SM_CXSCREEN),
                GetSystemMetrics(SM_CYSCREEN),
            )
        };

        let window = tauri::WebviewWindowBuilder::new(
            app,
            "overlay",
            tauri::WebviewUrl::App("overlay.html".into()),
        )
        .transparent(true)
        .decorations(false)
        .always_on_top(false)   // Applied via Win32 SetWindowPos below
        .skip_taskbar(true)
        .inner_size(screen_w as f64, screen_h as f64)
        .position(0.0, 0.0)
        .visible(false)
        .build()?;

        // Extract HWND and store it as isize (Send-safe)
        let hwnd       = window.hwnd()?;
        let hwnd_raw   = hwnd.0 as isize;

        Self::apply_overlay_styles(hwnd)?;

        Ok(Self {
            window,
            hwnd_raw,
            interactive_regions: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Apply Win32 extended styles for overlay behaviour.
    ///
    /// We do NOT apply WS_EX_TOPMOST / HWND_TOPMOST here on purpose.
    /// The overlay must NOT cover the Dev Simulator window while it is open.
    /// When deployed in production (over a real game), the integrating code
    /// would call SetWindowPos(HWND_TOPMOST) after detecting the game window.
    fn apply_overlay_styles(hwnd: HWND) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            // WS_EX_LAYERED    — required for SetLayeredWindowAttributes (opacity)
            // WS_EX_TRANSPARENT — click-through by default
            let new_style = ex_style
                | (WS_EX_LAYERED.0 as i32)
                | (WS_EX_TRANSPARENT.0 as i32);
            SetWindowLongW(hwnd, GWL_EXSTYLE, new_style);

            // Hide from OBS / Discord / Windows Game Bar screen capture
            SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE)?;
        }
        Ok(())
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

    /// Define which screen regions intercept mouse input.
    ///
    /// Calls Win32 `SetWindowRgn()` with a combined region.
    /// Pass an empty list to make the entire window click-through.
    pub fn set_interactive_regions(&self, regions: Vec<Rect>) -> Result<(), Box<dyn std::error::Error>> {
        *self.interactive_regions.lock().unwrap() = regions.clone();

        if regions.is_empty() {
            return self.set_click_through(true);
        }

        let hwnd = self.hwnd();
        unsafe {
            // Build a combined region from all widget bounding boxes
            let combined_region = CreateRectRgn(0, 0, 0, 0);

            for rect in regions {
                let widget_region = CreateRectRgn(
                    rect.x,
                    rect.y,
                    rect.x + rect.width,
                    rect.y + rect.height,
                );

                // windows 0.61: CombineRgn/SetWindowRgn take Option<HRGN>
                CombineRgn(
                    Some(combined_region),
                    Some(combined_region),
                    Some(widget_region),
                    RGN_OR,
                );
                let _ = DeleteObject(widget_region.into()); // HRGN → HGDIOBJ
            }

            let _ = SetWindowRgn(hwnd, Some(combined_region), true);

            self.set_click_through(false)?;
        }

        Ok(())
    }

    pub fn show(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.window.show()?;
        Ok(())
    }

    pub fn hide(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.window.hide()?;
        Ok(())
    }

    /// Set overlay opacity via `SetLayeredWindowAttributes(LWA_ALPHA)`.
    pub fn set_opacity(&self, opacity: f32) -> Result<(), Box<dyn std::error::Error>> {
        let opacity_byte = (opacity.clamp(0.0, 1.0) * 255.0) as u8;
        unsafe {
            SetLayeredWindowAttributes(self.hwnd(), COLORREF(0), opacity_byte, LWA_ALPHA)?;
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

    pub fn get_interactive_regions(&self) -> Vec<Rect> {
        self.interactive_regions.lock().unwrap().clone()
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
    fn test_region_manager() {
        let mut manager = super::super::region::RegionManager::new();
        let rect1 = Rect { x: 0, y: 0, width: 100, height: 100 };
        manager.add_region("app1.widget1".to_string(), rect1.clone());
        let regions = manager.get_all_regions();
        assert_eq!(regions.len(), 1);
        manager.remove_region("app1.widget1");
        assert_eq!(manager.get_all_regions().len(), 0);
    }
}
