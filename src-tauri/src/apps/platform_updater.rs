use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const PLATFORM_UPDATE_URL: &str = "https://releases.broken-latch.gg/latest.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformRelease {
    pub version: String,
    pub download_url: String,
    pub checksum: String,
    pub release_notes: String,
}

pub struct PlatformUpdater {
    client: reqwest::Client,
}

impl PlatformUpdater {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn check_for_update(
        &self,
    ) -> Result<Option<PlatformRelease>, super::updater::UpdateError> {
        use super::updater::UpdateError;

        log::info!("Checking for platform updates...");

        let response = self
            .client
            .get(PLATFORM_UPDATE_URL)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| UpdateError::NetworkError(e.to_string()))?;

        let release: PlatformRelease = response
            .json()
            .await
            .map_err(|e| UpdateError::ParseError(e.to_string()))?;

        let current = semver::Version::parse(env!("CARGO_PKG_VERSION"))
            .map_err(|e| UpdateError::ParseError(e.to_string()))?;
        let latest = semver::Version::parse(&release.version)
            .map_err(|e| UpdateError::ParseError(e.to_string()))?;

        if latest > current {
            log::info!("Platform update available: {} -> {}", current, latest);
            Ok(Some(release))
        } else {
            log::info!("Platform is up to date ({})", current);
            Ok(None)
        }
    }

    pub async fn download_update(
        &self,
        release: &PlatformRelease,
    ) -> Result<PathBuf, super::updater::UpdateError> {
        use super::updater::{calculate_sha256, UpdateError};

        log::info!("Downloading platform update v{}", release.version);

        let response = self
            .client
            .get(&release.download_url)
            .send()
            .await
            .map_err(|e| UpdateError::NetworkError(e.to_string()))?;

        let bytes = response
            .bytes()
            .await
            .map_err(|e| UpdateError::NetworkError(e.to_string()))?;

        let hash = calculate_sha256(&bytes);
        if hash != release.checksum {
            return Err(UpdateError::ChecksumMismatch);
        }

        let temp_path = std::env::temp_dir()
            .join(format!("broken-latch-update-{}.exe", release.version));

        tokio::fs::write(&temp_path, bytes)
            .await
            .map_err(|e| UpdateError::IoError(e.to_string()))?;

        log::info!("Update downloaded to {:?}", temp_path);
        Ok(temp_path)
    }

    pub fn apply_update(
        &self,
        installer_path: PathBuf,
    ) -> Result<(), super::updater::UpdateError> {
        use super::updater::UpdateError;

        log::info!("Launching update installer: {:?}", installer_path);

        std::process::Command::new(installer_path)
            .spawn()
            .map_err(|e| UpdateError::IoError(e.to_string()))?;

        std::process::exit(0);
    }
}
