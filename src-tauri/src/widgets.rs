use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, WebviewWindow};

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
    pub show_in_phases: Vec<String>,
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
    pub window: WebviewWindow,
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
    pub fn create_widget(
        &self,
        config: WidgetConfig,
    ) -> Result<String, WidgetError> {
        let widget_id = format!("{}:{}", config.app_id, config.id);

        if self.widgets.lock().unwrap().contains_key(&widget_id) {
            return Err(WidgetError::AlreadyExists);
        }

        let position = config.default_position.clone();
        let size = config.default_size.clone();

        // Create Tauri webview window (Tauri 2.x API)
        let window = tauri::WebviewWindowBuilder::new(
            &self.app_handle,
            &widget_id,
            tauri::WebviewUrl::App(config.url.clone().into()),
        )
        .title(&format!("Widget: {}", config.id))
        .position(position.x as f64, position.y as f64)
        .inner_size(size.width as f64, size.height as f64)
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .build()
        .map_err(|e| WidgetError::CreationFailed(e.to_string()))?;

        // Set min/max size constraints
        if let Some(ref min_size) = config.min_size {
            let _ = window.set_min_size(Some(tauri::LogicalSize::new(
                min_size.width as f64,
                min_size.height as f64,
            )));
        }

        if let Some(ref max_size) = config.max_size {
            let _ = window.set_max_size(Some(tauri::LogicalSize::new(
                max_size.width as f64,
                max_size.height as f64,
            )));
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

        // Inject drag handler script if draggable
        if config.draggable {
            let _ = Self::inject_drag_script(&window);
        }

        let widget_window = WidgetWindow {
            config,
            window,
            state,
        };

        self.widgets
            .lock()
            .unwrap()
            .insert(widget_id.clone(), widget_window);

        log::info!("Created widget {}", widget_id);

        Ok(widget_id)
    }

    /// Show a widget
    pub fn show_widget(&self, widget_id: &str) -> Result<(), WidgetError> {
        let widgets = self.widgets.lock().unwrap();
        let widget = widgets.get(widget_id).ok_or(WidgetError::NotFound)?;

        widget
            .window
            .show()
            .map_err(|e| WidgetError::ShowFailed(e.to_string()))?;
        widget.state.lock().unwrap().visible = true;

        Ok(())
    }

    /// Hide a widget
    pub fn hide_widget(&self, widget_id: &str) -> Result<(), WidgetError> {
        let widgets = self.widgets.lock().unwrap();
        let widget = widgets.get(widget_id).ok_or(WidgetError::NotFound)?;

        widget
            .window
            .hide()
            .map_err(|e| WidgetError::HideFailed(e.to_string()))?;
        widget.state.lock().unwrap().visible = false;

        Ok(())
    }

    /// Set widget opacity using SetLayeredWindowAttributes
    pub fn set_widget_opacity(&self, widget_id: &str, opacity: f32) -> Result<(), WidgetError> {
        let widgets = self.widgets.lock().unwrap();
        let widget = widgets.get(widget_id).ok_or(WidgetError::NotFound)?;

        set_window_opacity_raw(&widget.window, opacity)?;
        widget.state.lock().unwrap().opacity = opacity;

        Ok(())
    }

    /// Set widget position
    pub fn set_widget_position(
        &self,
        widget_id: &str,
        position: Position,
    ) -> Result<(), WidgetError> {
        let widgets = self.widgets.lock().unwrap();
        let widget = widgets.get(widget_id).ok_or(WidgetError::NotFound)?;

        widget
            .window
            .set_position(tauri::LogicalPosition::new(
                position.x as f64,
                position.y as f64,
            ))
            .map_err(|e| WidgetError::PositionFailed(e.to_string()))?;

        widget.state.lock().unwrap().position = position;

        Ok(())
    }

    /// Set widget click-through mode
    pub fn set_click_through(&self, widget_id: &str, enabled: bool) -> Result<(), WidgetError> {
        let widgets = self.widgets.lock().unwrap();
        let widget = widgets.get(widget_id).ok_or(WidgetError::NotFound)?;

        let hwnd_raw = widget
            .window
            .hwnd()
            .map_err(|e| WidgetError::PlatformError(e.to_string()))?;

        unsafe {
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::WindowsAndMessaging::*;

            let hwnd = HWND(hwnd_raw.0 as *mut _);
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
        let widget = widgets.get(widget_id).ok_or(WidgetError::NotFound)?;
        let state = widget.state.lock().unwrap().clone();
        Ok(state)
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
            let _ = widget.window.close();
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
            let should_show = widget.config.show_in_phases.is_empty()
                || widget
                    .config
                    .show_in_phases
                    .contains(&new_phase.to_string());

            let currently_visible = widget.state.lock().unwrap().visible;

            if should_show && !currently_visible {
                let _ = widget.window.show();
                widget.state.lock().unwrap().visible = true;
                log::debug!("Auto-showed widget {} for phase {}", widget_id, new_phase);
            } else if !should_show && currently_visible {
                let _ = widget.window.hide();
                widget.state.lock().unwrap().visible = false;
                log::debug!("Auto-hid widget {} for phase {}", widget_id, new_phase);
            }
        }
    }

    /// Inject drag handler JavaScript into widget window
    fn inject_drag_script(window: &WebviewWindow) -> Result<(), WidgetError> {
        let drag_script = r#"
            (function() {
                let isDragging = false;
                let startX, startY;

                document.addEventListener('mousedown', (e) => {
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
                        if (window.__TAURI__) {
                            window.__TAURI__.event.emit('widget-drag', {
                                deltaX,
                                deltaY
                            });
                        }
                        startX = e.clientX;
                        startY = e.clientY;
                    }
                });

                document.addEventListener('mouseup', () => {
                    if (isDragging) {
                        isDragging = false;
                        if (window.__TAURI__) {
                            window.__TAURI__.event.emit('widget-drag-end');
                        }
                    }
                });
            })();
        "#;

        window
            .eval(drag_script)
            .map_err(|e| WidgetError::CreationFailed(e.to_string()))
    }
}

/// Set window opacity via Win32 SetLayeredWindowAttributes
fn set_window_opacity(window: &WebviewWindow, opacity: f32) -> Result<(), WidgetError> {
    set_window_opacity_raw(window, opacity)
}

fn set_window_opacity_raw(window: &WebviewWindow, opacity: f32) -> Result<(), WidgetError> {
    let hwnd_raw = window
        .hwnd()
        .map_err(|e| WidgetError::PlatformError(e.to_string()))?;

    unsafe {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::*;

        let hwnd = HWND(hwnd_raw.0 as *mut _);

        // Ensure the window has layered style
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        if ex_style & WS_EX_LAYERED.0 as i32 == 0 {
            SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style | WS_EX_LAYERED.0 as i32);
        }

        let alpha = (opacity.clamp(0.0, 1.0) * 255.0) as u8;
        SetLayeredWindowAttributes(hwnd, windows::Win32::Foundation::COLORREF(0), alpha, LWA_ALPHA)
            .map_err(|e| WidgetError::PlatformError(e.to_string()))?;
    }

    Ok(())
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
            default_size: Size {
                width: 300,
                height: 400,
            },
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
        assert!(json.contains("InGame"));
    }

    #[test]
    fn test_position_bounds() {
        let pos = Position { x: -100, y: 50 };
        assert_eq!(pos.x, -100);
        assert_eq!(pos.y, 50);
    }

    #[test]
    fn test_widget_state_serialization() {
        let state = WidgetState {
            id: "panel".to_string(),
            app_id: "hunter".to_string(),
            visible: true,
            position: Position { x: 200, y: 300 },
            size: Size {
                width: 400,
                height: 500,
            },
            opacity: 0.85,
            click_through: false,
        };

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: WidgetState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "panel");
        assert_eq!(deserialized.opacity, 0.85);
    }
}
