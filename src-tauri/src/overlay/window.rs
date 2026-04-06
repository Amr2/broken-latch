use tauri::{AppHandle, Manager, Window};
use windows::Win32::{
    Foundation::*,
    UI::WindowsAndMessaging::*,
    Graphics::Gdi::*,
};
use std::sync::{Arc, Mutex};

/// Overlay window manager
pub struct OverlayWindow {
    window: Window,
    hwnd: HWND,
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
        let window = tauri::WindowBuilder::new(
            app,
            "overlay",
            tauri::WindowUrl::App("overlay.html".into())
        )
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .fullscreen(true)
        .visible(false) // Start hidden, show when game detected
        .build()?;

        // Get the Windows HWND
        let hwnd = window.hwnd()?;

        // Apply Windows-specific styles
        Self::apply_overlay_styles(hwnd)?;

        Ok(Self {
            window,
            hwnd,
            interactive_regions: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Apply Windows extended styles for overlay behavior
    fn apply_overlay_styles(hwnd: HWND) -> Result<(), Box<dyn std::error::Error>> {
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
                HWND_TOPMOST,
                0, 0, 0, 0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            )?;
        }

        Ok(())
    }

    /// Set the entire window to click-through mode
    pub fn set_click_through(&self, enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let ex_style = GetWindowLongW(self.hwnd, GWL_EXSTYLE);

            let new_style = if enabled {
                ex_style | (WS_EX_TRANSPARENT.0 as i32)
            } else {
                ex_style & !(WS_EX_TRANSPARENT.0 as i32)
            };

            SetWindowLongW(self.hwnd, GWL_EXSTYLE, new_style);
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

                CombineRgn(combined_region, combined_region, widget_region, RGN_OR);
                let _ = DeleteObject(widget_region);
            }

            // Apply the region to the window
            let _ = SetWindowRgn(self.hwnd, combined_region, true);

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
        let opacity_byte = (opacity.clamp(0.0, 1.0) * 255.0) as u8;

        unsafe {
            SetLayeredWindowAttributes(
                self.hwnd,
                COLORREF(0),
                opacity_byte,
                LWA_ALPHA,
            )?;
        }

        Ok(())
    }

    /// Set screen capture visibility for this window
    pub fn set_screen_capture_visible(&self, visible: bool) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let affinity = if visible {
                WDA_NONE
            } else {
                WDA_EXCLUDEFROMCAPTURE
            };

            SetWindowDisplayAffinity(self.hwnd, affinity)?;
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
