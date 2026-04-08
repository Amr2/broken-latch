use super::{AppManifest, AppError, AppState, InstalledApp};
use std::path::PathBuf;
use std::fs;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

pub struct AppRegistry {
    #[allow(dead_code)]
    app_handle: tauri::AppHandle,
}

impl AppRegistry {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }

    /// Install an app from a .lolapp file
    pub async fn install_app(&self, lolapp_path: PathBuf) -> Result<String, AppError> {
        log::info!("Installing app from: {:?}", lolapp_path);

        // Extract .lolapp (zip) to temp directory
        let temp_dir = self.extract_lolapp(&lolapp_path)?;

        // Load and validate manifest
        let manifest_path = temp_dir.join("manifest.json");
        let manifest_content = fs::read_to_string(&manifest_path)
            .map_err(|e| AppError::InvalidManifest(format!("Cannot read manifest: {}", e)))?;

        let manifest: AppManifest = serde_json::from_str(&manifest_content)
            .map_err(|e| AppError::InvalidManifest(format!("Invalid JSON: {}", e)))?;

        manifest
            .validate()
            .map_err(|e| AppError::InvalidManifest(e))?;

        // Check if app already installed
        if self.is_app_installed(&manifest.id) {
            return Err(AppError::AlreadyInstalled);
        }

        // Move app files to permanent location
        let install_path = self.get_app_install_path(&manifest.id)?;
        fs::create_dir_all(&install_path)?;
        copy_directory(&temp_dir, &install_path)?;

        // Generate auth token
        let auth_token = Uuid::new_v4().to_string();

        // Save metadata
        let meta = AppMetadata {
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            auth_token: auth_token.clone(),
            enabled: true,
            state: AppState::Installed,
        };
        let meta_path = install_path.join(".broken-latch-meta.json");
        fs::write(&meta_path, serde_json::to_string_pretty(&meta)?)?;

        // Clean up temp directory
        let _ = fs::remove_dir_all(temp_dir);

        log::info!("Installed app: {} v{}", manifest.name, manifest.version);
        Ok(manifest.id)
    }

    /// Uninstall an app
    pub fn uninstall_app(&self, app_id: &str) -> Result<(), AppError> {
        log::info!("Uninstalling app: {}", app_id);

        let install_path = self.get_app_install_path(app_id)?;
        if install_path.exists() {
            fs::remove_dir_all(install_path)?;
        }

        log::info!("Uninstalled app: {}", app_id);
        Ok(())
    }

    /// Get installed app by ID
    pub fn get_app(&self, app_id: &str) -> Result<InstalledApp, AppError> {
        let install_path = self.get_app_install_path(app_id)?;
        if !install_path.exists() {
            return Err(AppError::NotFound);
        }

        // Read metadata
        let meta_path = install_path.join(".broken-latch-meta.json");
        let meta: AppMetadata = serde_json::from_str(
            &fs::read_to_string(&meta_path).map_err(|_| AppError::NotFound)?,
        )?;

        // Read manifest
        let manifest_path = install_path.join("manifest.json");
        let manifest: AppManifest = serde_json::from_str(
            &fs::read_to_string(&manifest_path).map_err(|_| AppError::NotFound)?,
        )?;

        Ok(InstalledApp {
            id: meta.id,
            name: meta.name,
            version: meta.version,
            manifest,
            auth_token: meta.auth_token,
            install_path,
            enabled: meta.enabled,
            state: meta.state,
        })
    }

    /// List all installed apps
    pub fn list_apps(&self) -> Result<Vec<InstalledApp>, AppError> {
        let apps_dir = self.get_apps_base_dir()?;
        if !apps_dir.exists() {
            return Ok(vec![]);
        }

        let mut apps = Vec::new();
        for entry in fs::read_dir(apps_dir)? {
            let entry = entry?;
            if entry.path().is_dir() {
                if let Some(app_id) = entry.file_name().to_str() {
                    if let Ok(app) = self.get_app(app_id) {
                        apps.push(app);
                    }
                }
            }
        }

        Ok(apps)
    }

    /// Check if app is installed
    pub fn is_app_installed(&self, app_id: &str) -> bool {
        self.get_app_install_path(app_id)
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    /// Get app install directory path
    fn get_app_install_path(&self, app_id: &str) -> Result<PathBuf, AppError> {
        Ok(self.get_apps_base_dir()?.join(app_id))
    }

    fn get_apps_base_dir(&self) -> Result<PathBuf, AppError> {
        let app_data = std::env::var("APPDATA")
            .map_err(|_| AppError::PlatformError("Cannot get APPDATA".to_string()))?;
        Ok(PathBuf::from(app_data).join("broken-latch").join("apps"))
    }

    /// Extract .lolapp zip to temp directory
    fn extract_lolapp(&self, lolapp_path: &PathBuf) -> Result<PathBuf, AppError> {
        let file = fs::File::open(lolapp_path)
            .map_err(|e| AppError::InstallFailed(format!("Cannot open file: {}", e)))?;

        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| AppError::InstallFailed(format!("Invalid zip: {}", e)))?;

        let temp_dir = std::env::temp_dir().join(format!("lolapp-{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_dir)?;

        archive
            .extract(&temp_dir)
            .map_err(|e| AppError::InstallFailed(format!("Extract failed: {}", e)))?;

        Ok(temp_dir)
    }
}

/// Internal metadata stored alongside app files
#[derive(Debug, Serialize, Deserialize)]
struct AppMetadata {
    id: String,
    name: String,
    version: String,
    auth_token: String,
    enabled: bool,
    state: AppState,
}

/// Copy directory recursively
fn copy_directory(src: &PathBuf, dst: &PathBuf) -> Result<(), AppError> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            fs::create_dir_all(&dest_path)?;
            copy_directory(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}
