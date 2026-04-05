use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub version: String,
    pub startup: StartupConfig,
    pub overlay: OverlayConfig,
    pub performance: PerformanceConfig,
    pub developer: DeveloperConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupConfig {
    pub launch_with_windows: bool,
    pub auto_update_check: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    pub default_opacity: f32,
    pub screen_capture_visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub performance_mode: bool,
    pub cpu_usage_limit: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperConfig {
    pub debug_mode: bool,
    pub show_console_logs: bool,
    pub simulated_game_phases: bool,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            startup: StartupConfig {
                launch_with_windows: false,
                auto_update_check: true,
            },
            overlay: OverlayConfig {
                default_opacity: 1.0,
                screen_capture_visible: false,
            },
            performance: PerformanceConfig {
                performance_mode: false,
                cpu_usage_limit: 2.0,
            },
            developer: DeveloperConfig {
                debug_mode: false,
                show_console_logs: false,
                simulated_game_phases: false,
            },
        }
    }
}

pub fn load_config() -> Result<PlatformConfig, Box<dyn std::error::Error>> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        let default_config = PlatformConfig::default();
        save_config(&default_config)?;
        return Ok(default_config);
    }

    let content = std::fs::read_to_string(config_path)?;
    let config: PlatformConfig = toml::from_str(&content)?;
    Ok(config)
}

pub fn save_config(config: &PlatformConfig) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path()?;
    let content = toml::to_string_pretty(config)?;
    std::fs::write(config_path, content)?;
    Ok(())
}

fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let app_data = std::env::var("APPDATA")?;
    let config_dir = PathBuf::from(app_data).join("broken-latch");
    std::fs::create_dir_all(&config_dir)?;
    Ok(config_dir.join("config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = PlatformConfig::default();
        assert_eq!(config.overlay.default_opacity, 1.0);
        assert_eq!(config.startup.launch_with_windows, false);
        assert_eq!(config.performance.cpu_usage_limit, 2.0);
    }

    #[test]
    fn test_config_serialization() {
        let config = PlatformConfig::default();
        let serialized = toml::to_string(&config).unwrap();
        assert!(serialized.contains("version"));
        assert!(serialized.contains("default_opacity"));
    }
}
