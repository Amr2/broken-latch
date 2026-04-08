use super::{AppManifest, AppError, AppState, InstalledApp};
use crate::widgets::{WidgetManager, WidgetConfig};
use crate::hotkey::HotkeyManager;
use std::sync::{Arc, Mutex};
use std::process::{Command, Child};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct AppLoader {
    widget_manager: Arc<WidgetManager>,
    hotkey_manager: Arc<HotkeyManager>,
    running_apps: Mutex<HashMap<String, RunningApp>>,
}

struct RunningApp {
    #[allow(dead_code)]
    manifest: AppManifest,
    #[allow(dead_code)]
    state: AppState,
    backend_process: Option<Child>,
    widget_ids: Vec<String>,
}

impl AppLoader {
    pub fn new(
        widget_manager: Arc<WidgetManager>,
        hotkey_manager: Arc<HotkeyManager>,
    ) -> Self {
        Self {
            widget_manager,
            hotkey_manager,
            running_apps: Mutex::new(HashMap::new()),
        }
    }

    /// Start an app (load and run)
    pub fn start_app(&self, app: &InstalledApp) -> Result<(), AppError> {
        log::info!("Starting app: {}", app.id);

        {
            let running_apps = self.running_apps.lock().unwrap();
            if running_apps.contains_key(&app.id) {
                return Err(AppError::AlreadyRunning);
            }
        }

        // Start backend process if defined
        let backend_process = if let Some(ref backend) = app.manifest.backend {
            Some(self.start_backend_process(&app.install_path, backend)?)
        } else {
            None
        };

        // Create widgets from manifest
        let mut widget_ids = Vec::new();
        for window_manifest in &app.manifest.windows {
            let widget_config = WidgetConfig {
                id: window_manifest.id.clone(),
                app_id: app.id.clone(),
                url: format!(
                    "{}/{}",
                    app.install_path.display(),
                    window_manifest.url
                ),
                default_position: window_manifest.default_position.clone(),
                default_size: window_manifest.default_size.clone(),
                min_size: window_manifest.min_size.clone(),
                max_size: window_manifest.max_size.clone(),
                draggable: window_manifest.draggable,
                resizable: window_manifest.resizable,
                persist_position: window_manifest.persist_position,
                click_through: window_manifest.click_through,
                opacity: window_manifest.opacity,
                show_in_phases: window_manifest.show_in_phases.clone(),
            };

            let widget_id = self.widget_manager.create_widget(widget_config)?;
            widget_ids.push(widget_id);
        }

        // Register hotkeys from manifest
        for hotkey_manifest in &app.manifest.hotkeys {
            self.hotkey_manager.register(
                &app.id,
                &hotkey_manifest.id,
                &hotkey_manifest.default_keys,
            )?;
        }

        let mut running_apps = self.running_apps.lock().unwrap();
        running_apps.insert(
            app.id.clone(),
            RunningApp {
                manifest: app.manifest.clone(),
                state: AppState::Running,
                backend_process,
                widget_ids,
            },
        );

        log::info!("Started app: {}", app.id);
        Ok(())
    }

    /// Stop an app
    pub fn stop_app(&self, app_id: &str) -> Result<(), AppError> {
        log::info!("Stopping app: {}", app_id);

        let mut running_apps = self.running_apps.lock().unwrap();
        let mut running_app = running_apps.remove(app_id).ok_or(AppError::NotRunning)?;

        // Destroy widgets
        for widget_id in &running_app.widget_ids {
            let _ = self.widget_manager.destroy_widget(widget_id);
        }

        // Unregister hotkeys
        let _ = self.hotkey_manager.unregister_all_for_app(app_id);

        // Stop backend process
        if let Some(ref mut process) = running_app.backend_process {
            let _ = process.kill();
        }

        log::info!("Stopped app: {}", app_id);
        Ok(())
    }

    /// Suspend an app (hide widgets)
    pub fn suspend_app(&self, app_id: &str) -> Result<(), AppError> {
        let running_apps = self.running_apps.lock().unwrap();
        let running_app = running_apps.get(app_id).ok_or(AppError::NotRunning)?;

        for widget_id in &running_app.widget_ids {
            let _ = self.widget_manager.hide_widget(widget_id);
        }

        Ok(())
    }

    /// Resume an app (widgets auto-show based on phase)
    pub fn resume_app(&self, _app_id: &str) -> Result<(), AppError> {
        // Widgets will auto-show based on game phase via the listener
        Ok(())
    }

    /// Check if app is running
    pub fn is_running(&self, app_id: &str) -> bool {
        self.running_apps.lock().unwrap().contains_key(app_id)
    }

    /// Start backend process for an app
    fn start_backend_process(
        &self,
        install_path: &PathBuf,
        backend: &super::BackendManifest,
    ) -> Result<Child, AppError> {
        let backend_path = install_path.join(&backend.process);

        log::info!("Starting backend process: {:?}", backend_path);

        let process = Command::new(&backend_path)
            .current_dir(install_path)
            .spawn()
            .map_err(|e| AppError::BackendStartFailed(e.to_string()))?;

        Ok(process)
    }
}
