//! Configuration management

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::error::{CliError, Result};

const CONFIG_FILE_NAME: &str = ".borrowscope.toml";
const DEFAULT_CONFIG: &str = include_str!("../config/default.toml");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub run: RunConfig,

    #[serde(default)]
    pub visualize: VisualizeConfig,

    #[serde(default)]
    pub export: ExportConfig,

    #[serde(default)]
    pub tracking: TrackingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunConfig {
    #[serde(default = "default_output")]
    pub output: String,

    #[serde(default)]
    pub visualize: bool,

    #[serde(default = "default_true")]
    pub capture: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizeConfig {
    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_true")]
    pub browser: bool,

    #[serde(default = "default_host")]
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    #[serde(default = "default_export_format")]
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingConfig {
    #[serde(default = "default_true")]
    pub smart_pointers: bool,

    #[serde(default = "default_true")]
    pub async_code: bool,

    #[serde(default)]
    pub unsafe_code: bool,
}

impl Default for Config {
    fn default() -> Self {
        toml::from_str(DEFAULT_CONFIG).unwrap_or_else(|_| Self {
            run: RunConfig::default(),
            visualize: VisualizeConfig::default(),
            export: ExportConfig::default(),
            tracking: TrackingConfig::default(),
        })
    }
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            output: default_output(),
            visualize: false,
            capture: true,
        }
    }
}

impl Default for VisualizeConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            browser: true,
            host: default_host(),
        }
    }
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            format: default_export_format(),
        }
    }
}

impl Default for TrackingConfig {
    fn default() -> Self {
        Self {
            smart_pointers: true,
            async_code: true,
            unsafe_code: false,
        }
    }
}

impl Config {
    /// Load configuration from file or use defaults
    pub fn load() -> Result<Self> {
        // Try project config first
        if let Ok(config) = Self::load_from_path(CONFIG_FILE_NAME) {
            return Ok(config);
        }

        // Try user config
        if let Some(user_config) = Self::user_config_path() {
            if let Ok(config) = Self::load_from_path(&user_config) {
                return Ok(config);
            }
        }

        // Use defaults
        Ok(Self::default())
    }

    fn load_from_path(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let contents = fs::read_to_string(&path).map_err(|_| {
            CliError::ConfigError(format!("Could not read config file: {}", path.display()))
        })?;

        toml::from_str(&contents)
            .map_err(|e| CliError::ConfigError(format!("Invalid config file: {}", e)))
    }

    fn user_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("borrowscope").join("config.toml"))
    }

    /// Save configuration to file
    pub fn save(&self, path: impl Into<PathBuf>) -> Result<()> {
        let path = path.into();
        let contents = toml::to_string_pretty(self)
            .map_err(|e| CliError::ConfigError(format!("Failed to serialize config: {}", e)))?;

        fs::write(&path, contents)?;
        Ok(())
    }
}

fn default_output() -> String {
    "borrowscope.json".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_export_format() -> String {
    "dot".to_string()
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.run.output, "borrowscope.json");
        assert_eq!(config.visualize.port, 3000);
        assert!(config.tracking.smart_pointers);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.run.output, deserialized.run.output);
    }
}
