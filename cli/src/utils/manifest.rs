use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    pub entry_point: String,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub windows: Vec<WindowConfig>,
    #[serde(default)]
    pub hotkeys: Vec<HotkeyConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub id: String,
    pub default_keys: String,
    pub description: String,
}

impl AppManifest {
    /// Validate manifest fields, returns list of error strings
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.id.is_empty()
            || !self
                .id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            errors.push(
                "id must be lowercase alphanumeric with hyphens (e.g. my-app)".to_string(),
            );
        }

        if self.name.is_empty() || self.name.len() > 100 {
            errors.push("name must be 1–100 characters".to_string());
        }

        if self.version.is_empty() {
            errors.push("version is required".to_string());
        } else if semver::Version::parse(&self.version).is_err() {
            errors.push(format!("version '{}' is not valid semver", self.version));
        }

        if self.entry_point.is_empty() {
            errors.push("entry_point is required".to_string());
        }

        errors
    }
}

fn semver_parse(v: &str) -> Result<(), ()> {
    // Simple semver check: X.Y.Z
    let parts: Vec<&str> = v.split('.').collect();
    if parts.len() != 3 {
        return Err(());
    }
    for part in parts {
        if part.parse::<u64>().is_err() {
            return Err(());
        }
    }
    Ok(())
}

mod semver {
    pub struct Version;
    impl Version {
        pub fn parse(v: &str) -> Result<(), ()> {
            super::semver_parse(v)
        }
    }
}
