use tauri::{AppHandle, WebviewWindow};
use windows::Win32::{
    Foundation::*,
    UI::WindowsAndMessaging::*,
    Graphics::Gdi::*,
};
use std::sync::{Arc, Mutex};

/// Overlay window manager
pub struct OverlayWindow {
    window: WebviewWindow,
    // Stored as isize so OverlayWindow is Send + Sync (HWND wraps *mut c_void which is not Send)
    hwnd: isize,
    interactive_regions: Arc<Mutex<Vec<Rect>>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl OverlayWindow {
    /// Create the transparent fullscreen overlay
    pub fn create(app: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        // Create Tauri window with overlay configuration
        let window = tauri::WebviewWindowBuilder::new(
            app,
            "overlay",
            tauri::WebviewUrl::App("overlay.html".into())
        )
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .fullscreen(true)
        .visible(false) // Start hidden, show when game detected
        .build()?;

        // Get the Windows HWND and store as raw isize
        let hwnd_raw = window.hwnd()?;
        let hwnd = hwnd_raw.0 as isize;

        // Apply Windows-specific styles (non-fatal — some APIs may be unavailable in dev mode)
        if let Err(e) = Self::apply_overlay_styles(hwnd) {
            eprintln!("Warning: overlay styles partially applied: {}", e);
        }

        Ok(Self {
            window,
            hwnd,
            interactive_regions: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Apply Windows extended styles for overlay behavior
    fn apply_overlay_styles(hwnd: isize) -> Result<(), Box<dyn std::error::Error>> {
        let hwnd = HWND(hwnd as *mut _);
        unsafe {
            // Get current extended styles
            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);

            // Add layered and transparent flags
            let new_style = ex_style
                | (WS_EX_LAYERED.0 as i32)
                | (WS_EX_TRANSPARENT.0 as i32)
                | (WS_EX_TOPMOST.0 as i32);

            SetWindowLongW(hwnd, GWL_EXSTYLE, new_style);

            // Set window to exclude from screen capture by default
            // Apps can override this via manifest
            SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE)?;

            // Force window to top of z-order
            SetWindowPos(
                hwnd,
                Some(HWND_TOPMOST),
                0, 0, 0, 0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            )?;
        }

        Ok(())
    }

    /// Set the entire window to click-through mode
    pub fn set_click_through(&self, enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
        let hwnd = HWND(self.hwnd as *mut _);
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

    /// Set specific regions as interactive (only these capture input)
    /// All other areas remain click-through
    pub fn set_interactive_regions(&self, regions: Vec<Rect>) -> Result<(), Box<dyn std::error::Error>> {
        *self.interactive_regions.lock().unwrap() = regions.clone();

        if regions.is_empty() {
            // No interactive regions - make entire window click-through
            return self.set_click_through(true);
        }

        let hwnd = HWND(self.hwnd as *mut _);
        unsafe {
            // Create a combined region from all widget bounding boxes
            let combined_region = CreateRectRgn(0, 0, 0, 0);

            for rect in regions {
                let widget_region = CreateRectRgn(
                    rect.x,
                    rect.y,
                    rect.x + rect.width,
                    rect.y + rect.height,
                );

                CombineRgn(Some(combined_region), Some(combined_region), Some(widget_region), RGN_OR);
                let _ = DeleteObject(widget_region.into());
            }

            // Apply the region to the window
            let _ = SetWindowRgn(hwnd, Some(combined_region), true);

            // Ensure window is NOT click-through (interactive regions work)
            self.set_click_through(false)?;
        }

        Ok(())
    }

    /// Show the overlay window
    pub fn show(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.window.show()?;
        Ok(())
    }

    /// Hide the overlay window
    pub fn hide(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.window.hide()?;
        Ok(())
    }

    /// Set window opacity (0.0 = invisible, 1.0 = opaque)
    pub fn set_opacity(&self, opacity: f32) -> Result<(), Box<dyn std::error::Error>> {
        let hwnd = HWND(self.hwnd as *mut _);
        let opacity_byte = (opacity.clamp(0.0, 1.0) * 255.0) as u8;

        unsafe {
            SetLayeredWindowAttributes(
                hwnd,
                COLORREF(0),
                opacity_byte,
                LWA_ALPHA,
            )?;
        }

        Ok(())
    }

    /// Set screen capture visibility for this window
    pub fn set_screen_capture_visible(&self, visible: bool) -> Result<(), Box<dyn std::error::Error>> {
        let hwnd = HWND(self.hwnd as *mut _);
        unsafe {
            let affinity = if visible {
                WDA_NONE
            } else {
                WDA_EXCLUDEFROMCAPTURE
            };

            SetWindowDisplayAffinity(hwnd, affinity)?;
        }

        Ok(())
    }

    /// Get current interactive regions
    pub fn get_interactive_regions(&self) -> Vec<Rect> {
        self.interactive_regions.lock().unwrap().clone()
    }
}

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
        assert_eq!(regions[0].x, 0);

        manager.remove_region("app1.widget1");
        assert_eq!(manager.get_all_regions().len(), 0);
    }
}
