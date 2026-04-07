use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub update_url: Option<String>,
    pub entry_point: String,
    pub permissions: Vec<AppPermission>,
    pub windows: Vec<WindowManifest>,
    #[serde(default)]
    pub hotkeys: Vec<HotkeyManifest>,
    #[serde(default)]
    pub webhooks: Vec<WebhookManifest>,
    pub backend: Option<BackendManifest>,
}

impl AppManifest {
    /// Basic validation of manifest fields
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() || !self.id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return Err("id must be lowercase alphanumeric with hyphens".into());
        }
        if self.name.is_empty() || self.name.len() > 100 {
            return Err("name must be 1-100 characters".into());
        }
        if self.version.is_empty() {
            return Err("version is required".into());
        }
        if self.entry_point.is_empty() {
            return Err("entry_point is required".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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
    #[serde(default = "default_true")]
    pub draggable: bool,
    #[serde(default)]
    pub resizable: bool,
    #[serde(default)]
    pub persist_position: bool,
    #[serde(default)]
    pub click_through: bool,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    #[serde(default)]
    pub show_in_phases: Vec<String>,
}

fn default_true() -> bool { true }
fn default_opacity() -> f32 { 1.0 }

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
    pub process: String,
    pub port: u16,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledApp {
    pub id: String,
    pub name: String,
    pub version: String,
    pub manifest: AppManifest,
    pub auth_token: String,
    pub install_path: std::path::PathBuf,
    pub enabled: bool,
    pub state: AppState,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("App not found")]
    NotFound,
    #[error("App already installed")]
    AlreadyInstalled,
    #[error("App already running")]
    AlreadyRunning,
    #[error("App not running")]
    NotRunning,
    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),
    #[error("Install failed: {0}")]
    InstallFailed(String),
    #[error("Backend start failed: {0}")]
    BackendStartFailed(String),
    #[error("Platform error: {0}")]
    PlatformError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Widget error: {0}")]
    WidgetError(#[from] crate::widgets::WidgetError),
    #[error("Hotkey error: {0}")]
    HotkeyError(#[from] crate::hotkey::HotkeyError),
}

pub mod registry;
pub mod loader;
pub mod sandbox;
pub mod updater;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_validation_valid() {
        let manifest = AppManifest {
            id: "hunter-mode".to_string(),
            name: "Hunter Mode".to_string(),
            version: "1.0.0".to_string(),
            description: "A test app".to_string(),
            author: "Test".to_string(),
            update_url: None,
            entry_point: "index.html".to_string(),
            permissions: vec![AppPermission::GameSession],
            windows: vec![],
            hotkeys: vec![],
            webhooks: vec![],
            backend: None,
        };
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_manifest_validation_invalid_id() {
        let manifest = AppManifest {
            id: "Invalid ID!".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            description: "".to_string(),
            author: "".to_string(),
            update_url: None,
            entry_point: "index.html".to_string(),
            permissions: vec![],
            windows: vec![],
            hotkeys: vec![],
            webhooks: vec![],
            backend: None,
        };
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_app_permission_serialization() {
        let perm = AppPermission::GameSession;
        let json = serde_json::to_string(&perm).unwrap();
        assert_eq!(json, r#""game.session""#);
    }

    #[test]
    fn test_app_state_serialization() {
        let state = AppState::Running;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, r#""Running""#);
    }
}
