# Task 02: Core Overlay Window System

**Platform: broken-latch**  
**Dependencies:** Task 01 (Project Setup)  
**Estimated Complexity:** High  
**Priority:** P0 (Critical Path)

---

## Objective

Implement the transparent, fullscreen overlay window that sits on top of League of Legends using Windows API. This window is the canvas where all app widgets will render. It must be click-through by default, always-on-top, and support selective interactive regions for app widgets.

---

## Context

The overlay window is broken-latch's core rendering surface. It's a single fullscreen transparent window that:

- Covers the entire primary monitor
- Is always above the game window (HWND_TOPMOST)
- Passes all input through to the game by default (WS_EX_TRANSPARENT)
- Can selectively capture input in specific widget bounding boxes
- Is hidden from screen capture (optional, per-app setting)

This task implements the Windows API calls needed to create and manage this special window.

---

## What You Need to Build

### 1. Windows API Dependencies

Add to `src-tauri/Cargo.toml`:

```toml
[dependencies]
windows = { version = "0.52", features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
    "Win32_Graphics_Gdi",
    "Win32_System_LibraryLoader",
]}
```

### 2. Overlay Window Module (`src-tauri/src/overlay/window.rs`)

**Core functionality:**

```rust
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
            let mut combined_region = CreateRectRgn(0, 0, 0, 0);

            for rect in regions {
                let widget_region = CreateRectRgn(
                    rect.x,
                    rect.y,
                    rect.x + rect.width,
                    rect.y + rect.height,
                );

                CombineRgn(combined_region, combined_region, widget_region, RGN_OR);
                DeleteObject(widget_region);
            }

            // Apply the region to the window
            SetWindowRgn(self.hwnd, combined_region, true);

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
```

### 3. Region Tracking Module (`src-tauri/src/overlay/region.rs`)

Tracks which widgets are interactive and manages region updates:

```rust
use super::window::Rect;
use std::collections::HashMap;

/// Manages interactive regions for app widgets
pub struct RegionManager {
    /// Map of app_id.widget_id -> Rect
    regions: HashMap<String, Rect>,
}

impl RegionManager {
    pub fn new() -> Self {
        Self {
            regions: HashMap::new(),
        }
    }

    /// Register a widget as interactive
    pub fn add_region(&mut self, widget_key: String, rect: Rect) {
        self.regions.insert(widget_key, rect);
    }

    /// Remove a widget's interactive region
    pub fn remove_region(&mut self, widget_key: &str) {
        self.regions.remove(widget_key);
    }

    /// Get all current interactive regions as a Vec
    pub fn get_all_regions(&self) -> Vec<Rect> {
        self.regions.values().cloned().collect()
    }

    /// Update a widget's region (e.g., after drag/resize)
    pub fn update_region(&mut self, widget_key: String, rect: Rect) {
        self.regions.insert(widget_key, rect);
    }

    /// Clear all regions
    pub fn clear(&mut self) {
        self.regions.clear();
    }
}
```

### 4. Module Exports (`src-tauri/src/overlay/mod.rs`)

```rust
pub mod window;
pub mod region;

pub use window::{OverlayWindow, Rect};
pub use region::RegionManager;
```

### 5. App State Integration (`src-tauri/src/main.rs`)

Create global state to hold overlay window:

```rust
use std::sync::Mutex;
use tauri::{Manager, State};

mod overlay;
use overlay::{OverlayWindow, RegionManager};

// ... other mod declarations from Task 01

struct AppState {
    overlay: Mutex<Option<OverlayWindow>>,
    region_manager: Mutex<RegionManager>,
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize overlay window
            let overlay = OverlayWindow::create(app.handle())?;

            // Store in app state
            app.manage(AppState {
                overlay: Mutex::new(Some(overlay)),
                region_manager: Mutex::new(RegionManager::new()),
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 6. Tauri Commands for Frontend Control

Add these commands to expose overlay control to the frontend/apps:

```rust
#[tauri::command]
async fn show_overlay(state: State<'_, AppState>) -> Result<(), String> {
    let overlay = state.overlay.lock().unwrap();
    if let Some(ref window) = *overlay {
        window.show().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn hide_overlay(state: State<'_, AppState>) -> Result<(), String> {
    let overlay = state.overlay.lock().unwrap();
    if let Some(ref window) = *overlay {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn set_overlay_opacity(opacity: f32, state: State<'_, AppState>) -> Result<(), String> {
    let overlay = state.overlay.lock().unwrap();
    if let Some(ref window) = *overlay {
        window.set_opacity(opacity).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn update_interactive_regions(
    regions: Vec<overlay::Rect>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let overlay = state.overlay.lock().unwrap();
    if let Some(ref window) = *overlay {
        window.set_interactive_regions(regions).map_err(|e| e.to_string())?;
    }
    Ok(())
}

// Register commands in main():
fn main() {
    tauri::Builder::default()
        .setup(/* ... */)
        .invoke_handler(tauri::generate_handler![
            show_overlay,
            hide_overlay,
            set_overlay_opacity,
            update_interactive_regions,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 7. Overlay HTML (`src/overlay.html`)

Create a minimal HTML file that will be loaded into the overlay window:

```html
<!DOCTYPE html>
<html>
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>broken-latch Overlay</title>
    <style>
      * {
        margin: 0;
        padding: 0;
        box-sizing: border-box;
      }

      body {
        width: 100vw;
        height: 100vh;
        overflow: hidden;
        background: transparent;
        pointer-events: none; /* Default to click-through */
      }

      #overlay-root {
        width: 100%;
        height: 100%;
        position: relative;
      }

      /* App widget containers will be injected here */
      .widget-container {
        position: absolute;
        pointer-events: auto; /* Widgets can capture input */
      }
    </style>
  </head>
  <body>
    <div id="overlay-root">
      <!-- App widgets will be dynamically inserted here by Task 06 -->
    </div>
  </body>
</html>
```

---

## Integration Points

### From Task 01:

- Uses `PlatformConfig.overlay.default_opacity` for initial opacity
- Uses `PlatformConfig.overlay.screen_capture_visible` for capture affinity

### For Task 04 (Game Detection):

- Game detector will call `show_overlay()` when League launches
- Will call `hide_overlay()` when game closes

### For Task 06 (Widget System):

- Widget manager will call `update_interactive_regions()` when widgets are shown/hidden/moved
- Each widget reports its bounding box via `Rect { x, y, width, height }`

### For Task 07 (App Lifecycle):

- App loader will inject app WebView2 instances into `#overlay-root`
- Each app gets its own container div with absolute positioning

---

## Testing Requirements

### Unit Tests (`src-tauri/src/overlay/window.rs`)

```rust
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
```

### Integration Tests (`src-tauri/tests/overlay_test.rs`)

```rust
use broken_latch::overlay::{OverlayWindow, Rect};

#[test]
fn test_overlay_window_lifecycle() {
    // This requires a Tauri app instance - run in dev mode
    // Test: create overlay, show, hide, set opacity
}
```

### Manual Testing Checklist

- [ ] Launch platform in dev mode: `npm run tauri dev`
- [ ] Verify overlay window is created but hidden initially
- [ ] Call `show_overlay()` from devtools console
- [ ] Verify fullscreen transparent window appears
- [ ] Click through the overlay - clicks pass to desktop beneath
- [ ] Call `update_interactive_regions([{x:100,y:100,width:200,height:200}])`
- [ ] Verify clicking inside region does NOT pass through
- [ ] Verify clicking outside region still passes through
- [ ] Call `set_overlay_opacity(0.5)` - verify overlay becomes semi-transparent
- [ ] Call `hide_overlay()` - verify overlay disappears
- [ ] Verify no memory leaks after 10+ show/hide cycles

---

## Acceptance Criteria

✅ **Complete when:**

1. Overlay window can be created and shown programmatically
2. Window is always on top (above other windows including games)
3. Window is fully transparent (can see desktop through it)
4. Window is click-through by default
5. Interactive regions work: input captured only in defined rectangles
6. Opacity can be controlled from 0.0 to 1.0
7. Screen capture visibility can be toggled
8. All unit tests pass
9. Manual testing checklist is 100% complete
10. No performance impact: CPU usage <0.5% when idle with overlay shown

---

## Performance Requirements

- Window creation: <100ms
- Show/hide transition: <16ms (1 frame at 60fps)
- Region update: <16ms
- RAM overhead: <5MB for overlay window

---

## Dependencies for Next Tasks

- **Task 03** (DirectX Hook) will composite content onto this window
- **Task 04** (Game Detection) will show/hide this window based on game state
- **Task 06** (Widget System) will inject widgets and update interactive regions
- **Task 07** (App Lifecycle) will render app WebView2 instances into this window

---

## Files to Create/Modify

### New Files:

- `src-tauri/src/overlay/window.rs`
- `src-tauri/src/overlay/region.rs`
- `src-tauri/src/overlay/mod.rs`
- `src/overlay.html`
- `src-tauri/tests/overlay_test.rs`

### Modified Files:

- `src-tauri/src/main.rs` (add state, commands)
- `src-tauri/Cargo.toml` (add windows crate)

---

## Expected Time: 6-8 hours

## Difficulty: High (Windows API expertise required)
