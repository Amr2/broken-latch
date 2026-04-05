# Task 07: App Lifecycle Manager

**Platform: broken-latch**  
**Dependencies:** Task 01, 04, 05, 06  
**Estimated Complexity:** Very High  
**Priority:** P0 (Critical Path)

---

## Objective

Implement the complete app lifecycle management system that handles app installation, loading, sandboxing, state transitions, and runtime management. This is the orchestrator that brings together all platform components and manages the lifecycle of third-party apps running on broken-latch.

---

## Context

The app lifecycle manager is responsible for:

- Installing `.lolapp` packages (zip files with manifest)
- Validating app manifests and permissions
- Loading apps on platform startup
- Managing app state transitions (Installed → Enabled → Loading → Running → Suspended → Stopped)
- Creating app widgets from manifest definitions
- Registering app hotkeys from manifest
- Subscribing to webhooks from manifest
- Isolating apps in WebView2 sandboxes
- Providing apps with authentication tokens
- Cleaning up when apps are uninstalled

This is one of the most complex components as it ties together almost every other system.

---

## What You Need to Build

### 1. Dependencies

Add to `src-tauri/Cargo.toml`:

```toml
[dependencies]
zip = "0.6"                          # Extract .lolapp files
uuid = { version = "1.0", features = ["v4", "serde"] }
regex = "1.10"
validator = { version = "0.16", features = ["derive"] }
```

### 2. App Manifest Model (`src-tauri/src/apps/mod.rs`)

```rust
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AppManifest {
    #[validate(regex = "^[a-z0-9-]+$")]
    pub id: String,

    #[validate(length(min = 1, max = 100))]
    pub name: String,

    #[validate(regex = r"^\d+\.\d+\.\d+$")]
    pub version: String,

    #[validate(length(max = 500))]
    pub description: String,

    pub author: String,

    #[validate(url)]
    pub update_url: Option<String>,

    pub entry_point: String,  // e.g. "index.html"

    pub permissions: Vec<AppPermission>,

    pub windows: Vec<WindowManifest>,

    pub hotkeys: Vec<HotkeyManifest>,

    pub webhooks: Vec<WebhookManifest>,

    pub backend: Option<BackendManifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AppPermission {
    #[serde(rename = "game.session")]
    GameSession,

    #[serde(rename = "windows.create")]
    WindowsCreate,

    #[serde(rename = "hotkeys.register")]
    HotkeysRegister,

    #[serde(rename = "storage")]
    Storage,

    #[serde(rename = "notify")]
    Notify,

    #[serde(rename = "messaging")]
    Messaging,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowManifest {
    pub id: String,
    pub url: String,
    pub default_position: crate::widgets::Position,
    pub default_size: crate::widgets::Size,
    pub min_size: Option<crate::widgets::Size>,
    pub max_size: Option<crate::widgets::Size>,
    pub draggable: bool,
    pub resizable: bool,
    pub persist_position: bool,
    pub click_through: bool,
    pub opacity: f32,
    pub show_in_phases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyManifest {
    pub id: String,
    pub default_keys: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookManifest {
    pub event: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendManifest {
    pub process: String,  // e.g. "backend.exe" or "node server.js"
    pub port: u16,
}

pub mod registry;
pub mod loader;
pub mod sandbox;
pub mod updater;
```

### 3. App Registry (`src-tauri/src/apps/registry.rs`)

```rust
use super::AppManifest;
use std::path::PathBuf;
use std::fs;
use uuid::Uuid;

pub struct AppRegistry {
    app_handle: tauri::AppHandle,
}

impl AppRegistry {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }

    /// Install an app from a .lolapp file
    pub async fn install_app(
        &self,
        lolapp_path: PathBuf,
    ) -> Result<String, AppError> {
        log::info!("Installing app from: {:?}", lolapp_path);

        // Extract .lolapp (zip) to temp directory
        let temp_dir = self.extract_lolapp(&lolapp_path)?;

        // Load and validate manifest
        let manifest_path = temp_dir.join("manifest.json");
        let manifest_content = fs::read_to_string(&manifest_path)
            .map_err(|e| AppError::InvalidManifest(format!("Cannot read manifest: {}", e)))?;

        let manifest: AppManifest = serde_json::from_str(&manifest_content)
            .map_err(|e| AppError::InvalidManifest(format!("Invalid JSON: {}", e)))?;

        // Validate manifest
        manifest.validate()
            .map_err(|e| AppError::InvalidManifest(format!("Validation failed: {}", e)))?;

        // Check if app already installed
        if self.is_app_installed(&manifest.id).await? {
            return Err(AppError::AlreadyInstalled);
        }

        // Move app files to permanent location
        let install_path = self.get_app_install_path(&manifest.id)?;
        fs::create_dir_all(&install_path)?;

        self.copy_directory(&temp_dir, &install_path)?;

        // Generate auth token
        let auth_token = Uuid::new_v4().to_string();

        // Save to database
        self.save_app_to_db(&manifest, &install_path, &auth_token).await?;

        // Clean up temp directory
        fs::remove_dir_all(temp_dir).ok();

        log::info!("Installed app: {} v{}", manifest.name, manifest.version);

        Ok(manifest.id)
    }

    /// Uninstall an app
    pub async fn uninstall_app(&self, app_id: &str) -> Result<(), AppError> {
        log::info!("Uninstalling app: {}", app_id);

        // Remove from database
        self.remove_app_from_db(app_id).await?;

        // Delete app files
        let install_path = self.get_app_install_path(app_id)?;
        fs::remove_dir_all(install_path)?;

        log::info!("Uninstalled app: {}", app_id);

        Ok(())
    }

    /// Get installed app by ID
    pub async fn get_app(&self, app_id: &str) -> Result<InstalledApp, AppError> {
        // Query database for app
        // TODO: Implement database query
        Err(AppError::NotFound)
    }

    /// List all installed apps
    pub async fn list_apps(&self) -> Result<Vec<InstalledApp>, AppError> {
        // Query database for all apps
        // TODO: Implement database query
        Ok(vec![])
    }

    /// Check if app is installed
    pub async fn is_app_installed(&self, app_id: &str) -> Result<bool, AppError> {
        // Query database
        // TODO: Implement
        Ok(false)
    }

    /// Get app install directory path
    fn get_app_install_path(&self, app_id: &str) -> Result<PathBuf, AppError> {
        let app_data = std::env::var("APPDATA")
            .map_err(|_| AppError::PlatformError("Cannot get APPDATA".to_string()))?;

        let path = PathBuf::from(app_data)
            .join("broken-latch")
            .join("apps")
            .join(app_id);

        Ok(path)
    }

    /// Extract .lolapp zip to temp directory
    fn extract_lolapp(&self, lolapp_path: &PathBuf) -> Result<PathBuf, AppError> {
        let file = fs::File::open(lolapp_path)
            .map_err(|e| AppError::InstallFailed(format!("Cannot open file: {}", e)))?;

        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| AppError::InstallFailed(format!("Invalid zip: {}", e)))?;

        let temp_dir = std::env::temp_dir().join(format!("lolapp-{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir)?;

        archive.extract(&temp_dir)
            .map_err(|e| AppError::InstallFailed(format!("Extract failed: {}", e)))?;

        Ok(temp_dir)
    }

    /// Copy directory recursively
    fn copy_directory(&self, src: &PathBuf, dst: &PathBuf) -> Result<(), AppError> {
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let dest_path = dst.join(entry.file_name());

            if path.is_dir() {
                fs::create_dir_all(&dest_path)?;
                self.copy_directory(&path, &dest_path)?;
            } else {
                fs::copy(&path, &dest_path)?;
            }
        }

        Ok(())
    }

    /// Save app to database
    async fn save_app_to_db(
        &self,
        manifest: &AppManifest,
        install_path: &PathBuf,
        auth_token: &str,
    ) -> Result<(), AppError> {
        let manifest_json = serde_json::to_string(manifest)?;
        let now = chrono::Utc::now().timestamp();

        // TODO: Execute database insert
        // INSERT INTO apps (id, name, version, description, author, manifest_json,
        //                   install_path, auth_token, installed_at, updated_at)
        // VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)

        Ok(())
    }

    /// Remove app from database
    async fn remove_app_from_db(&self, app_id: &str) -> Result<(), AppError> {
        // TODO: Execute database delete
        // DELETE FROM apps WHERE id = ?

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledApp {
    pub id: String,
    pub name: String,
    pub version: String,
    pub manifest: AppManifest,
    pub auth_token: String,
    pub install_path: PathBuf,
    pub enabled: bool,
    pub state: AppState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppState {
    Installed,
    Enabled,
    Loading,
    Running,
    Suspended,
    Stopped,
    Error,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("App not found")]
    NotFound,

    #[error("App already installed")]
    AlreadyInstalled,

    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    #[error("Install failed: {0}")]
    InstallFailed(String),

    #[error("Platform error: {0}")]
    PlatformError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}
```

### 4. App Loader (`src-tauri/src/apps/loader.rs`)

```rust
use super::{AppManifest, InstalledApp, AppState};
use crate::widgets::{WidgetManager, WidgetConfig};
use crate::hotkey::HotkeyManager;
use std::sync::Arc;
use std::process::{Command, Child};
use std::collections::HashMap;

pub struct AppLoader {
    app_handle: tauri::AppHandle,
    widget_manager: Arc<WidgetManager>,
    hotkey_manager: Arc<HotkeyManager>,
    running_apps: Arc<Mutex<HashMap<String, RunningApp>>>,
}

struct RunningApp {
    manifest: AppManifest,
    state: AppState,
    backend_process: Option<Child>,
    widget_ids: Vec<String>,
}

impl AppLoader {
    pub fn new(
        app_handle: tauri::AppHandle,
        widget_manager: Arc<WidgetManager>,
        hotkey_manager: Arc<HotkeyManager>,
    ) -> Self {
        Self {
            app_handle,
            widget_manager,
            hotkey_manager,
            running_apps: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start an app (load and run)
    pub async fn start_app(&self, app: &InstalledApp) -> Result<(), AppError> {
        log::info!("Starting app: {}", app.id);

        let mut running_apps = self.running_apps.lock().unwrap();

        if running_apps.contains_key(&app.id) {
            return Err(AppError::AlreadyRunning);
        }

        // Update state to Loading
        self.update_app_state(&app.id, AppState::Loading).await?;

        // Start backend process if defined
        let backend_process = if let Some(backend) = &app.manifest.backend {
            Some(self.start_backend_process(&app.id, backend)?)
        } else {
            None
        };

        // Create widgets from manifest
        let mut widget_ids = Vec::new();
        for window_manifest in &app.manifest.windows {
            let widget_config = WidgetConfig {
                id: window_manifest.id.clone(),
                app_id: app.id.clone(),
                url: format!("{}/{}", app.install_path.display(), window_manifest.url),
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

            let widget_id = self.widget_manager.create_widget(widget_config).await?;
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

        // Register webhooks
        for webhook in &app.manifest.webhooks {
            self.register_webhook(&app.id, &webhook.event, &webhook.url).await?;
        }

        // Update state to Running
        self.update_app_state(&app.id, AppState::Running).await?;

        running_apps.insert(app.id.clone(), RunningApp {
            manifest: app.manifest.clone(),
            state: AppState::Running,
            backend_process,
            widget_ids,
        });

        log::info!("Started app: {}", app.id);

        Ok(())
    }

    /// Stop an app
    pub async fn stop_app(&self, app_id: &str) -> Result<(), AppError> {
        log::info!("Stopping app: {}", app_id);

        let mut running_apps = self.running_apps.lock().unwrap();

        let running_app = running_apps.remove(app_id)
            .ok_or(AppError::NotRunning)?;

        // Destroy widgets
        for widget_id in running_app.widget_ids {
            self.widget_manager.destroy_widget(&widget_id).ok();
        }

        // Unregister hotkeys
        self.hotkey_manager.unregister_all_for_app(app_id).ok();

        // Unregister webhooks
        self.unregister_all_webhooks(app_id).await?;

        // Stop backend process
        if let Some(mut process) = running_app.backend_process {
            process.kill().ok();
        }

        // Update state
        self.update_app_state(app_id, AppState::Stopped).await?;

        log::info!("Stopped app: {}", app_id);

        Ok(())
    }

    /// Suspend an app (game not running)
    pub async fn suspend_app(&self, app_id: &str) -> Result<(), AppError> {
        log::info!("Suspending app: {}", app_id);

        let running_apps = self.running_apps.lock().unwrap();
        let running_app = running_apps.get(app_id)
            .ok_or(AppError::NotRunning)?;

        // Hide all widgets
        for widget_id in &running_app.widget_ids {
            self.widget_manager.hide_widget(widget_id).ok();
        }

        self.update_app_state(app_id, AppState::Suspended).await?;

        Ok(())
    }

    /// Resume an app (game started again)
    pub async fn resume_app(&self, app_id: &str) -> Result<(), AppError> {
        log::info!("Resuming app: {}", app_id);

        self.update_app_state(app_id, AppState::Running).await?;

        // Widgets will auto-show based on game phase

        Ok(())
    }

    /// Start backend process for an app
    fn start_backend_process(
        &self,
        app_id: &str,
        backend: &super::BackendManifest,
    ) -> Result<Child, AppError> {
        let install_path = self.get_app_install_path(app_id)?;
        let backend_path = install_path.join(&backend.process);

        log::info!("Starting backend process: {:?}", backend_path);

        let process = Command::new(backend_path)
            .current_dir(install_path)
            .spawn()
            .map_err(|e| AppError::BackendStartFailed(e.to_string()))?;

        // Wait a moment for backend to start
        std::thread::sleep(std::time::Duration::from_millis(500));

        Ok(process)
    }

    /// Register webhook for an app
    async fn register_webhook(
        &self,
        app_id: &str,
        event: &str,
        url: &str,
    ) -> Result<(), AppError> {
        // TODO: Save to webhooks table
        // INSERT INTO webhooks (app_id, event, url, created_at)
        Ok(())
    }

    /// Unregister all webhooks for an app
    async fn unregister_all_webhooks(&self, app_id: &str) -> Result<(), AppError> {
        // TODO: Delete from webhooks table
        // DELETE FROM webhooks WHERE app_id = ?
        Ok(())
    }

    /// Update app state in database
    async fn update_app_state(&self, app_id: &str, state: AppState) -> Result<(), AppError> {
        // TODO: Update apps table
        // UPDATE apps SET state = ? WHERE id = ?
        Ok(())
    }

    fn get_app_install_path(&self, app_id: &str) -> Result<PathBuf, AppError> {
        let app_data = std::env::var("APPDATA")
            .map_err(|_| AppError::PlatformError("Cannot get APPDATA".to_string()))?;

        Ok(PathBuf::from(app_data).join("broken-latch").join("apps").join(app_id))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("App already running")]
    AlreadyRunning,

    #[error("App not running")]
    NotRunning,

    #[error("Backend start failed: {0}")]
    BackendStartFailed(String),

    // ... other errors
}
```

### 5. Permission Sandbox (`src-tauri/src/apps/sandbox.rs`)

```rust
use super::AppPermission;
use std::collections::HashSet;

pub struct PermissionSandbox {
    app_permissions: std::collections::HashMap<String, HashSet<AppPermission>>,
}

impl PermissionSandbox {
    pub fn new() -> Self {
        Self {
            app_permissions: std::collections::HashMap::new(),
        }
    }

    /// Load app permissions
    pub fn load_permissions(&mut self, app_id: &str, permissions: Vec<AppPermission>) {
        self.app_permissions.insert(
            app_id.to_string(),
            permissions.into_iter().collect(),
        );
    }

    /// Check if app has permission
    pub fn has_permission(&self, app_id: &str, permission: &AppPermission) -> bool {
        self.app_permissions
            .get(app_id)
            .map(|perms| perms.contains(permission))
            .unwrap_or(false)
    }

    /// Enforce permission check (returns error if denied)
    pub fn require_permission(
        &self,
        app_id: &str,
        permission: &AppPermission,
    ) -> Result<(), PermissionError> {
        if self.has_permission(app_id, permission) {
            Ok(())
        } else {
            Err(PermissionError::Denied(format!(
                "App {} does not have permission: {:?}",
                app_id, permission
            )))
        }
    }

    /// Clear permissions for an app
    pub fn clear_permissions(&mut self, app_id: &str) {
        self.app_permissions.remove(app_id);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PermissionError {
    #[error("Permission denied: {0}")]
    Denied(String),
}
```

### 6. Tauri Commands

```rust
#[tauri::command]
async fn install_app(
    lolapp_path: String,
    registry: tauri::State<'_, Arc<AppRegistry>>,
) -> Result<String, String> {
    registry.install_app(PathBuf::from(lolapp_path)).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn uninstall_app(
    app_id: String,
    registry: tauri::State<'_, Arc<AppRegistry>>,
) -> Result<(), String> {
    registry.uninstall_app(&app_id).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_app(
    app_id: String,
    registry: tauri::State<'_, Arc<AppRegistry>>,
    loader: tauri::State<'_, Arc<AppLoader>>,
) -> Result<(), String> {
    let app = registry.get_app(&app_id).await
        .map_err(|e| e.to_string())?;

    loader.start_app(&app).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_app(
    app_id: String,
    loader: tauri::State<'_, Arc<AppLoader>>,
) -> Result<(), String> {
    loader.stop_app(&app_id).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_installed_apps(
    registry: tauri::State<'_, Arc<AppRegistry>>,
) -> Result<Vec<InstalledApp>, String> {
    registry.list_apps().await
        .map_err(|e| e.to_string())
}
```

---

## Integration Points

### Uses Task 04 (Game Lifecycle):

- Suspends apps when game is NotRunning
- Resumes apps when game starts

### Uses Task 05 (Hotkey Manager):

- Registers hotkeys from app manifest

### Uses Task 06 (Widget Manager):

- Creates widgets from app manifest

### Used by Task 08 (HTTP API):

- Provides app management endpoints

### Used by Task 09 (SDK):

- Validates app permissions for SDK calls

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_validation() {
        // Test valid manifest parses
        // Test invalid manifest fails
    }

    #[test]
    fn test_permission_sandbox() {
        let mut sandbox = PermissionSandbox::new();
        sandbox.load_permissions("app1", vec![AppPermission::Storage]);

        assert!(sandbox.has_permission("app1", &AppPermission::Storage));
        assert!(!sandbox.has_permission("app1", &AppPermission::GameSession));
    }
}
```

### Manual Testing Checklist

- [ ] Install .lolapp file succeeds
- [ ] App appears in installed apps list
- [ ] Starting app creates widgets
- [ ] Starting app registers hotkeys
- [ ] Backend process starts if defined
- [ ] Stopping app cleans up all resources
- [ ] Uninstalling app removes files
- [ ] Permission checks work correctly
- [ ] App state transitions correctly

---

## Acceptance Criteria

✅ **Complete when:**

1. Apps can be installed from .lolapp files
2. Manifest validation works correctly
3. Apps can be started and stopped
4. Widgets are created from manifest
5. Hotkeys are registered from manifest
6. Backend processes start correctly
7. Permission sandbox enforces restrictions
8. All unit tests pass
9. Manual testing checklist is 100% complete
10. App state persists across restarts

---

## Files to Create/Modify

### New Files:

- `src-tauri/src/apps/mod.rs`
- `src-tauri/src/apps/registry.rs`
- `src-tauri/src/apps/loader.rs`
- `src-tauri/src/apps/sandbox.rs`

### Modified Files:

- `src-tauri/src/main.rs`
- `src-tauri/Cargo.toml`

---

## Expected Time: 14-16 hours

## Difficulty: Very High (Complex orchestration + file I/O + process management)
