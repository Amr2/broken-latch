# Task 06: Widget Window System

**Platform: broken-latch**  
**Dependencies:** Task 01, 02 (Overlay Window), 04 (Game Lifecycle Detector)  
**Estimated Complexity:** High  
**Priority:** P0 (Critical Path)

---

## Objective

Implement the widget window management system that allows apps to create draggable, resizable overlay windows (panels) that render on top of the game. Each widget is a WebView2 instance positioned within the overlay window, with support for persistence, phase-based visibility, and interactive regions.

---

## Context

Widget windows are how apps display their UI. A single app can have multiple widgets (e.g., Hunter Mode has separate "Loading Panel", "Win Condition Panel", "Hunter Focus Panel"). The platform:

- Creates WebView2 instances for each widget
- Positions them within the overlay window
- Handles drag/resize interactions
- Persists widget positions to database
- Auto shows/hides based on game phase
- Manages click-through regions (only widget areas capture input)

Apps define widgets in their `manifest.json` and control them via `LOLOverlay.windows.create()` in the SDK.

---

## What You Need to Build

### 1. Dependencies

Add to `src-tauri/Cargo.toml`:

```toml
[dependencies]
webview2-com = "0.29"        # WebView2 Rust bindings
windows = { version = "0.52", features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
    "Win32_Graphics_Gdi",
    "Win32_System_Com",
]}
```

### 2. Widget Data Model (`src-tauri/src/widgets.rs`)

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, Window};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetConfig {
    pub id: String,
    pub app_id: String,
    pub url: String,
    pub default_position: Position,
    pub default_size: Size,
    pub min_size: Option<Size>,
    pub max_size: Option<Size>,
    pub draggable: bool,
    pub resizable: bool,
    pub persist_position: bool,
    pub click_through: bool,
    pub opacity: f32,
    pub show_in_phases: Vec<String>,  // ["InGame", "Loading"]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetState {
    pub id: String,
    pub app_id: String,
    pub visible: bool,
    pub position: Position,
    pub size: Size,
    pub opacity: f32,
    pub click_through: bool,
}

pub struct WidgetWindow {
    pub config: WidgetConfig,
    pub window: Window,
    pub state: Arc<Mutex<WidgetState>>,
}

pub struct WidgetManager {
    widgets: Arc<Mutex<HashMap<String, WidgetWindow>>>,
    app_handle: AppHandle,
}

impl WidgetManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            widgets: Arc::new(Mutex::new(HashMap::new())),
            app_handle,
        }
    }

    /// Create a new widget window
    pub async fn create_widget(
        &self,
        config: WidgetConfig,
    ) -> Result<String, WidgetError> {
        let widget_id = format!("{}:{}", config.app_id, config.id);

        // Check if widget already exists
        if self.widgets.lock().unwrap().contains_key(&widget_id) {
            return Err(WidgetError::AlreadyExists);
        }

        // Load persisted position if enabled
        let position = if config.persist_position {
            self.load_position(&config.app_id, &config.id).await
                .unwrap_or(config.default_position.clone())
        } else {
            config.default_position.clone()
        };

        let size = config.default_size.clone();

        // Create Tauri window for this widget
        let window = tauri::WindowBuilder::new(
            &self.app_handle,
            &widget_id,
            tauri::WindowUrl::App(config.url.clone().into()),
        )
        .title(&format!("Widget: {}", config.id))
        .position(position.x as f64, position.y as f64)
        .inner_size(size.width as f64, size.height as f64)
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)  // Start hidden, show based on phase
        .build()
        .map_err(|e| WidgetError::CreationFailed(e.to_string()))?;

        // Set opacity
        self.set_window_opacity(&window, config.opacity)?;

        // Set min/max size constraints
        if let Some(min_size) = &config.min_size {
            window.set_min_size(Some(tauri::LogicalSize::new(
                min_size.width as f64,
                min_size.height as f64,
            ))).ok();
        }

        if let Some(max_size) = &config.max_size {
            window.set_max_size(Some(tauri::LogicalSize::new(
                max_size.width as f64,
                max_size.height as f64,
            ))).ok();
        }

        // Create widget state
        let state = Arc::new(Mutex::new(WidgetState {
            id: config.id.clone(),
            app_id: config.app_id.clone(),
            visible: false,
            position: position.clone(),
            size: size.clone(),
            opacity: config.opacity,
            click_through: config.click_through,
        }));

        // Set up drag handlers if draggable
        if config.draggable {
            self.setup_drag_handlers(&window, state.clone())?;
        }

        // Store widget
        let widget_window = WidgetWindow {
            config: config.clone(),
            window: window.clone(),
            state: state.clone(),
        };

        self.widgets.lock().unwrap().insert(widget_id.clone(), widget_window);

        log::info!("Created widget {} for app {}", config.id, config.app_id);

        Ok(widget_id)
    }

    /// Show a widget
    pub fn show_widget(&self, widget_id: &str) -> Result<(), WidgetError> {
        let widgets = self.widgets.lock().unwrap();
        let widget = widgets.get(widget_id)
            .ok_or(WidgetError::NotFound)?;

        widget.window.show().map_err(|e| WidgetError::ShowFailed(e.to_string()))?;
        widget.state.lock().unwrap().visible = true;

        log::debug!("Showed widget: {}", widget_id);

        Ok(())
    }

    /// Hide a widget
    pub fn hide_widget(&self, widget_id: &str) -> Result<(), WidgetError> {
        let widgets = self.widgets.lock().unwrap();
        let widget = widgets.get(widget_id)
            .ok_or(WidgetError::NotFound)?;

        widget.window.hide().map_err(|e| WidgetError::HideFailed(e.to_string()))?;
        widget.state.lock().unwrap().visible = false;

        log::debug!("Hid widget: {}", widget_id);

        Ok(())
    }

    /// Set widget opacity
    pub fn set_opacity(&self, widget_id: &str, opacity: f32) -> Result<(), WidgetError> {
        let widgets = self.widgets.lock().unwrap();
        let widget = widgets.get(widget_id)
            .ok_or(WidgetError::NotFound)?;

        self.set_window_opacity(&widget.window, opacity)?;
        widget.state.lock().unwrap().opacity = opacity;

        Ok(())
    }

    /// Set widget position
    pub async fn set_position(
        &self,
        widget_id: &str,
        position: Position,
    ) -> Result<(), WidgetError> {
        let widgets = self.widgets.lock().unwrap();
        let widget = widgets.get(widget_id)
            .ok_or(WidgetError::NotFound)?;

        widget.window.set_position(tauri::LogicalPosition::new(
            position.x as f64,
            position.y as f64,
        )).map_err(|e| WidgetError::PositionFailed(e.to_string()))?;

        widget.state.lock().unwrap().position = position.clone();

        // Persist if enabled
        if widget.config.persist_position {
            self.save_position(&widget.config.app_id, &widget.config.id, &position).await?;
        }

        Ok(())
    }

    /// Set widget click-through mode
    pub fn set_click_through(&self, widget_id: &str, enabled: bool) -> Result<(), WidgetError> {
        let widgets = self.widgets.lock().unwrap();
        let widget = widgets.get(widget_id)
            .ok_or(WidgetError::NotFound)?;

        // Apply Windows extended style
        let hwnd = widget.window.hwnd()
            .map_err(|e| WidgetError::PlatformError(e.to_string()))?;

        unsafe {
            use windows::Win32::UI::WindowsAndMessaging::*;

            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            let new_style = if enabled {
                ex_style | WS_EX_TRANSPARENT.0 as i32
            } else {
                ex_style & !(WS_EX_TRANSPARENT.0 as i32)
            };

            SetWindowLongW(hwnd, GWL_EXSTYLE, new_style);
        }

        widget.state.lock().unwrap().click_through = enabled;

        Ok(())
    }

    /// Get widget state
    pub fn get_state(&self, widget_id: &str) -> Result<WidgetState, WidgetError> {
        let widgets = self.widgets.lock().unwrap();
        let widget = widgets.get(widget_id)
            .ok_or(WidgetError::NotFound)?;

        Ok(widget.state.lock().unwrap().clone())
    }

    /// Get all widgets for an app
    pub fn get_widgets_for_app(&self, app_id: &str) -> Vec<String> {
        self.widgets
            .lock()
            .unwrap()
            .keys()
            .filter(|id| id.starts_with(&format!("{}:", app_id)))
            .cloned()
            .collect()
    }

    /// Destroy a widget
    pub fn destroy_widget(&self, widget_id: &str) -> Result<(), WidgetError> {
        let mut widgets = self.widgets.lock().unwrap();

        if let Some(widget) = widgets.remove(widget_id) {
            widget.window.close().ok();
            log::info!("Destroyed widget: {}", widget_id);
            Ok(())
        } else {
            Err(WidgetError::NotFound)
        }
    }

    /// Destroy all widgets for an app
    pub fn destroy_all_for_app(&self, app_id: &str) -> Result<(), WidgetError> {
        let widget_ids = self.get_widgets_for_app(app_id);

        for widget_id in widget_ids {
            self.destroy_widget(&widget_id)?;
        }

        Ok(())
    }

    /// Handle game phase change - show/hide widgets based on showInPhases
    pub fn handle_phase_change(&self, new_phase: &str) {
        let widgets = self.widgets.lock().unwrap();

        for (widget_id, widget) in widgets.iter() {
            let should_show = widget.config.show_in_phases.contains(&new_phase.to_string())
                || widget.config.show_in_phases.is_empty();

            if should_show && !widget.state.lock().unwrap().visible {
                widget.window.show().ok();
                widget.state.lock().unwrap().visible = true;
                log::debug!("Auto-showed widget {} for phase {}", widget_id, new_phase);
            } else if !should_show && widget.state.lock().unwrap().visible {
                widget.window.hide().ok();
                widget.state.lock().unwrap().visible = false;
                log::debug!("Auto-hid widget {} for phase {}", widget_id, new_phase);
            }
        }
    }

    /// Set window opacity using Windows API
    fn set_window_opacity(&self, window: &Window, opacity: f32) -> Result<(), WidgetError> {
        let hwnd = window.hwnd()
            .map_err(|e| WidgetError::PlatformError(e.to_string()))?;

        unsafe {
            use windows::Win32::UI::WindowsAndMessaging::*;
            use windows::Win32::Graphics::Gdi::*;

            let alpha = (opacity * 255.0) as u8;

            SetLayeredWindowAttributes(
                hwnd,
                COLORREF(0),
                alpha,
                LWA_ALPHA,
            ).map_err(|e| WidgetError::PlatformError(e.to_string()))?;
        }

        Ok(())
    }

    /// Set up drag handlers for movable widgets
    fn setup_drag_handlers(
        &self,
        window: &Window,
        state: Arc<Mutex<WidgetState>>,
    ) -> Result<(), WidgetError> {
        // Inject JavaScript to handle drag events
        let drag_script = r#"
            (function() {
                let isDragging = false;
                let startX, startY;

                document.addEventListener('mousedown', (e) => {
                    // Only start drag if on drag handle element
                    if (e.target.hasAttribute('data-broken-latch-drag')) {
                        isDragging = true;
                        startX = e.clientX;
                        startY = e.clientY;
                        e.preventDefault();
                    }
                });

                document.addEventListener('mousemove', (e) => {
                    if (isDragging) {
                        const deltaX = e.clientX - startX;
                        const deltaY = e.clientY - startY;

                        window.__TAURI__.event.emit('widget-drag', {
                            deltaX,
                            deltaY
                        });

                        startX = e.clientX;
                        startY = e.clientY;
                    }
                });

                document.addEventListener('mouseup', () => {
                    if (isDragging) {
                        isDragging = false;
                        window.__TAURI__.event.emit('widget-drag-end');
                    }
                });
            })();
        "#;

        window.eval(drag_script)
            .map_err(|e| WidgetError::CreationFailed(e.to_string()))?;

        Ok(())
    }

    /// Load persisted widget position from database
    async fn load_position(&self, app_id: &str, widget_id: &str) -> Result<Position, WidgetError> {
        // TODO: Query database for saved position
        // For now, return error to use default
        Err(WidgetError::NotFound)
    }

    /// Save widget position to database
    async fn save_position(
        &self,
        app_id: &str,
        widget_id: &str,
        position: &Position,
    ) -> Result<(), WidgetError> {
        // TODO: Save to database
        // Will be implemented when db module is ready
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WidgetError {
    #[error("Widget already exists")]
    AlreadyExists,

    #[error("Widget not found")]
    NotFound,

    #[error("Failed to create widget: {0}")]
    CreationFailed(String),

    #[error("Failed to show widget: {0}")]
    ShowFailed(String),

    #[error("Failed to hide widget: {0}")]
    HideFailed(String),

    #[error("Failed to set position: {0}")]
    PositionFailed(String),

    #[error("Platform error: {0}")]
    PlatformError(String),
}
```

### 3. Tauri Commands (`src-tauri/src/main.rs`)

```rust
use crate::widgets::{WidgetManager, WidgetConfig, Position, WidgetState};

#[tauri::command]
async fn create_widget(
    config: WidgetConfig,
    manager: tauri::State<'_, Arc<WidgetManager>>,
) -> Result<String, String> {
    manager.create_widget(config).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn show_widget(
    widget_id: String,
    manager: tauri::State<'_, Arc<WidgetManager>>,
) -> Result<(), String> {
    manager.show_widget(&widget_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn hide_widget(
    widget_id: String,
    manager: tauri::State<'_, Arc<WidgetManager>>,
) -> Result<(), String> {
    manager.hide_widget(&widget_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_widget_opacity(
    widget_id: String,
    opacity: f32,
    manager: tauri::State<'_, Arc<WidgetManager>>,
) -> Result<(), String> {
    manager.set_opacity(&widget_id, opacity)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_widget_position(
    widget_id: String,
    position: Position,
    manager: tauri::State<'_, Arc<WidgetManager>>,
) -> Result<(), String> {
    manager.set_position(&widget_id, position).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_widget_click_through(
    widget_id: String,
    enabled: bool,
    manager: tauri::State<'_, Arc<WidgetManager>>,
) -> Result<(), String> {
    manager.set_click_through(&widget_id, enabled)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_widget_state(
    widget_id: String,
    manager: tauri::State<'_, Arc<WidgetManager>>,
) -> Result<WidgetState, String> {
    manager.get_state(&widget_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn destroy_widget(
    widget_id: String,
    manager: tauri::State<'_, Arc<WidgetManager>>,
) -> Result<(), String> {
    manager.destroy_widget(&widget_id)
        .map_err(|e| e.to_string())
}

// In main():
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let widget_manager = Arc::new(WidgetManager::new(app.handle()));
            app.manage(widget_manager.clone());

            // Subscribe to game phase changes
            let widget_manager_clone = widget_manager.clone();
            app.listen_global("game_phase_changed", move |event| {
                if let Some(phase_data) = event.payload() {
                    // Parse phase and handle widget visibility
                    widget_manager_clone.handle_phase_change(phase_data);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_widget,
            show_widget,
            hide_widget,
            set_widget_opacity,
            set_widget_position,
            set_widget_click_through,
            get_widget_state,
            destroy_widget,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## Integration Points

### Integration with Task 02 (Overlay Window)

- Widgets are positioned within the overlay window's coordinate space
- Overlay window's interactive regions are updated when widgets are created/moved

### Integration with Task 04 (Game Lifecycle Detector)

- Listens to `game_phase_changed` events
- Auto shows/hides widgets based on `showInPhases` config

### Integration with Task 05 (Hotkey Manager)

- Hotkeys can toggle widget visibility
- Apps define hotkey bindings for their widgets

### Integration with Task 07 (App Lifecycle Manager)

- Task 07 reads `windows` field from app manifest
- Creates widgets automatically when app starts
- Destroys widgets when app stops

### Integration with Task 08 (HTTP API Server)

- Exposes widget management endpoints (`POST /api/windows/create`, etc.)

### Integration with Task 09 (JavaScript SDK)

- SDK provides `LOLOverlay.windows.create()` wrapper
- SDK handles widget drag events from injected JavaScript

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_config_serialization() {
        let config = WidgetConfig {
            id: "test-widget".to_string(),
            app_id: "test-app".to_string(),
            url: "index.html".to_string(),
            default_position: Position { x: 100, y: 100 },
            default_size: Size { width: 300, height: 400 },
            min_size: None,
            max_size: None,
            draggable: true,
            resizable: false,
            persist_position: true,
            click_through: false,
            opacity: 0.9,
            show_in_phases: vec!["InGame".to_string()],
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("test-widget"));
    }

    #[test]
    fn test_position_bounds() {
        let pos = Position { x: -100, y: 50 };
        assert_eq!(pos.x, -100);
        assert_eq!(pos.y, 50);
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_widget_lifecycle() {
    // Create widget
    // Show widget
    // Set position
    // Hide widget
    // Destroy widget
}
```

### Manual Testing Checklist

- [ ] Create widget via Tauri command succeeds
- [ ] Widget window appears at correct position
- [ ] Widget window has correct size
- [ ] Dragging widget updates its position
- [ ] Widget opacity changes when set
- [ ] Widget hides when game phase doesn't match showInPhases
- [ ] Widget shows when game phase matches showInPhases
- [ ] Widget position persists across platform restarts
- [ ] Multiple widgets for same app can coexist
- [ ] Click-through mode works correctly
- [ ] Min/max size constraints are enforced
- [ ] Destroying widget removes it completely

---

## Acceptance Criteria

✅ **Complete when:**

1. WidgetManager compiles and initializes successfully
2. Widgets can be created with full configuration
3. Widgets can be shown, hidden, moved, and destroyed
4. Drag handlers work correctly with data-broken-latch-drag attribute
5. Phase-based visibility works automatically
6. Widget positions persist to database when enabled
7. Opacity and click-through settings work
8. All unit tests pass
9. Integration tests pass
10. Manual testing checklist is 100% complete
11. Multiple widgets can be managed simultaneously
12. No memory leaks when creating/destroying widgets

---

## Dependencies for Next Tasks

- **Task 07** depends on: Widget creation from app manifests
- **Task 08** depends on: Widget management for HTTP API
- **Task 09** depends on: Widget commands for SDK wrapping
- **Task 10** depends on: Widget list for platform UI

---

## Files to Create/Modify

### New Files:

- `src-tauri/src/widgets.rs`

### Modified Files:

- `src-tauri/src/main.rs` (add commands and setup)
- `src-tauri/Cargo.toml` (add dependencies)
- `src-tauri/src/db.rs` (add widget position persistence)

---

## Expected Time: 10-12 hours

## Difficulty: High (WebView2 + window management + drag/drop)
