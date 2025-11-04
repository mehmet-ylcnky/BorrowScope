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
    use std::fs;
    use tempfile::TempDir;

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

    #[test]
    fn test_run_config_defaults() {
        let config = RunConfig::default();
        assert_eq!(config.output, "borrowscope.json");
        assert!(!config.visualize);
        assert!(config.capture);
    }

    #[test]
    fn test_visualize_config_defaults() {
        let config = VisualizeConfig::default();
        assert_eq!(config.port, 3000);
        assert!(config.browser);
        assert_eq!(config.host, "127.0.0.1");
    }

    #[test]
    fn test_export_config_defaults() {
        let config = ExportConfig::default();
        assert_eq!(config.format, "dot");
    }

    #[test]
    fn test_tracking_config_defaults() {
        let config = TrackingConfig::default();
        assert!(config.smart_pointers);
        assert!(config.async_code);
        assert!(!config.unsafe_code);
    }

    #[test]
    fn test_config_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join(".borrowscope.toml");

        let toml_content = r#"
[run]
output = "custom.json"
visualize = true
capture = false

[visualize]
port = 8080
browser = false
host = "0.0.0.0"

[export]
format = "json"

[tracking]
smart_pointers = false
async_code = false
unsafe_code = true
"#;

        fs::write(&config_file, toml_content).unwrap();

        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.run.output, "custom.json");
        assert!(config.run.visualize);
        assert!(!config.run.capture);
        assert_eq!(config.visualize.port, 8080);
        assert!(!config.visualize.browser);
        assert_eq!(config.visualize.host, "0.0.0.0");
        assert_eq!(config.export.format, "json");
        assert!(!config.tracking.smart_pointers);
        assert!(!config.tracking.async_code);
        assert!(config.tracking.unsafe_code);
    }

    #[test]
    fn test_config_partial_override() {
        let toml_content = r#"
[run]
output = "custom.json"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.run.output, "custom.json");
        // Other fields should have defaults
        assert_eq!(config.visualize.port, 3000);
        assert!(config.tracking.smart_pointers);
    }

    #[test]
    fn test_config_all_sections() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();

        assert!(toml_str.contains("[run]"));
        assert!(toml_str.contains("[visualize]"));
        assert!(toml_str.contains("[export]"));
        assert!(toml_str.contains("[tracking]"));
    }

    #[test]
    fn test_config_clone() {
        let config = Config::default();
        let cloned = config.clone();

        assert_eq!(config.run.output, cloned.run.output);
        assert_eq!(config.visualize.port, cloned.visualize.port);
        assert_eq!(config.export.format, cloned.export.format);
        assert_eq!(
            config.tracking.smart_pointers,
            cloned.tracking.smart_pointers
        );
    }

    #[test]
    fn test_config_custom_values() {
        let mut config = Config::default();
        config.run.output = "test.json".to_string();
        config.visualize.port = 9000;
        config.export.format = "html".to_string();
        config.tracking.unsafe_code = true;

        assert_eq!(config.run.output, "test.json");
        assert_eq!(config.visualize.port, 9000);
        assert_eq!(config.export.format, "html");
        assert!(config.tracking.unsafe_code);
    }

    #[test]
    fn test_config_roundtrip() {
        let original = Config::default();
        let toml_str = toml::to_string(&original).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        let toml_str2 = toml::to_string(&deserialized).unwrap();

        assert_eq!(toml_str, toml_str2);
    }

    #[test]
    fn test_invalid_config() {
        let invalid_toml = r#"
[run]
output = 123  # Should be string
"#;

        let result = toml::from_str::<Config>(invalid_toml);
        assert!(result.is_err());
    }
}
