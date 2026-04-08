use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const REGISTRY_URL: &str = "https://apps.broken-latch.gg/registry.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRegistryEntry {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub category: String,
    pub download_url: String,
    pub icon_url: String,
    pub screenshots: Vec<String>,
    pub min_platform_version: String,
    pub file_size: u64,
    pub checksum: String,
    pub rating: f32,
    pub download_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRegistry {
    pub version: String,
    pub apps: Vec<AppRegistryEntry>,
}

pub struct RegistryClient {
    client: reqwest::Client,
}

impl RegistryClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn fetch_registry(&self) -> Result<AppRegistry, UpdateError> {
        log::info!("Fetching app registry from {}", REGISTRY_URL);

        let response = self
            .client
            .get(REGISTRY_URL)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| UpdateError::NetworkError(e.to_string()))?;

        let registry: AppRegistry = response
            .json()
            .await
            .map_err(|e| UpdateError::ParseError(e.to_string()))?;

        log::info!("Fetched {} apps from registry", registry.apps.len());
        Ok(registry)
    }

    pub async fn download_app(
        &self,
        entry: &AppRegistryEntry,
        dest_path: PathBuf,
    ) -> Result<(), UpdateError> {
        log::info!("Downloading app {} from {}", entry.name, entry.download_url);

        let response = self
            .client
            .get(&entry.download_url)
            .send()
            .await
            .map_err(|e| UpdateError::NetworkError(e.to_string()))?;

        let bytes = response
            .bytes()
            .await
            .map_err(|e| UpdateError::NetworkError(e.to_string()))?;

        let hash = calculate_sha256(&bytes);
        if hash != entry.checksum {
            return Err(UpdateError::ChecksumMismatch);
        }

        tokio::fs::write(&dest_path, bytes)
            .await
            .map_err(|e| UpdateError::IoError(e.to_string()))?;

        log::info!("Downloaded app to {:?}", dest_path);
        Ok(())
    }

    pub async fn check_app_update(
        &self,
        app_id: &str,
        current_version: &str,
    ) -> Result<Option<AppRegistryEntry>, UpdateError> {
        let registry = self.fetch_registry().await?;

        let entry = registry
            .apps
            .iter()
            .find(|app| app.id == app_id)
            .ok_or(UpdateError::AppNotFound)?;

        let current = semver::Version::parse(current_version)
            .map_err(|e| UpdateError::ParseError(e.to_string()))?;
        let latest = semver::Version::parse(&entry.version)
            .map_err(|e| UpdateError::ParseError(e.to_string()))?;

        if latest > current {
            Ok(Some(entry.clone()))
        } else {
            Ok(None)
        }
    }
}

pub fn calculate_sha256(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Checksum mismatch")]
    ChecksumMismatch,

    #[error("App not found in registry")]
    AppNotFound,
}
