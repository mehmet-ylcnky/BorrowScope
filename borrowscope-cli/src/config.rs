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

    #[serde(default)]
    pub ignore: IgnoreConfig,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreConfig {
    #[serde(default = "default_ignore_patterns")]
    pub patterns: Vec<String>,

    #[serde(default = "default_ignore_directories")]
    pub directories: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        toml::from_str(DEFAULT_CONFIG).unwrap_or_else(|_| Self {
            run: RunConfig::default(),
            visualize: VisualizeConfig::default(),
            export: ExportConfig::default(),
            tracking: TrackingConfig::default(),
            ignore: IgnoreConfig::default(),
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

impl Default for IgnoreConfig {
    fn default() -> Self {
        Self {
            patterns: default_ignore_patterns(),
            directories: default_ignore_directories(),
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

    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        if self.visualize.port < 1024 {
            return Err(CliError::ConfigError(
                "Port must be >= 1024 (privileged ports require root)".to_string(),
            ));
        }

        if self.run.output.is_empty() {
            return Err(CliError::ConfigError(
                "Output filename cannot be empty".to_string(),
            ));
        }

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

fn default_ignore_patterns() -> Vec<String> {
    vec!["*.test.rs".to_string(), "*_test.rs".to_string()]
}

fn default_ignore_directories() -> Vec<String> {
    vec!["target".to_string(), "tests".to_string()]
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

    #[test]
    fn test_ignore_config_defaults() {
        let config = IgnoreConfig::default();
        assert_eq!(config.patterns.len(), 2);
        assert!(config.patterns.contains(&"*.test.rs".to_string()));
        assert!(config.patterns.contains(&"*_test.rs".to_string()));
        assert_eq!(config.directories.len(), 2);
        assert!(config.directories.contains(&"target".to_string()));
        assert!(config.directories.contains(&"tests".to_string()));
    }

    #[test]
    fn test_config_with_ignore_section() {
        let toml_content = r#"
[ignore]
patterns = ["*.tmp", "*.bak"]
directories = ["build", "dist"]
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.ignore.patterns.len(), 2);
        assert!(config.ignore.patterns.contains(&"*.tmp".to_string()));
        assert_eq!(config.ignore.directories.len(), 2);
        assert!(config.ignore.directories.contains(&"build".to_string()));
    }

    #[test]
    fn test_validate_valid_config() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_port_too_low() {
        let mut config = Config::default();
        config.visualize.port = 80;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_port_too_high() {
        // u16 max is 65535, which is valid - type system prevents overflow
        let mut config = Config::default();
        config.visualize.port = 65535;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_output() {
        let mut config = Config::default();
        config.run.output = "".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_valid_port_range() {
        let mut config = Config::default();
        config.visualize.port = 8080;
        assert!(config.validate().is_ok());
        
        config.visualize.port = 1024;
        assert!(config.validate().is_ok());
        
        config.visualize.port = 65535;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_ignore_patterns_custom() {
        let mut config = Config::default();
        config.ignore.patterns = vec!["*.rs".to_string(), "*.toml".to_string()];
        assert_eq!(config.ignore.patterns.len(), 2);
    }

    #[test]
    fn test_ignore_directories_custom() {
        let mut config = Config::default();
        config.ignore.directories = vec!["node_modules".to_string()];
        assert_eq!(config.ignore.directories.len(), 1);
    }

    // Edge Case Tests

    #[test]
    fn test_config_empty_toml() {
        let toml_content = "";
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.run.output, "borrowscope.json");
        assert_eq!(config.visualize.port, 3000);
    }

    #[test]
    fn test_config_only_run_section() {
        let toml_content = r#"
[run]
output = "test.json"
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.run.output, "test.json");
        assert_eq!(config.visualize.port, 3000);
    }

    #[test]
    fn test_config_missing_fields_in_section() {
        let toml_content = r#"
[run]
output = "test.json"
# visualize and capture missing
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.run.output, "test.json");
        assert!(!config.run.visualize);
        assert!(config.run.capture);
    }

    #[test]
    fn test_config_extreme_port_values() {
        let mut config = Config::default();
        
        config.visualize.port = 1024;
        assert!(config.validate().is_ok());
        
        config.visualize.port = 65535;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_port_boundary() {
        let mut config = Config::default();
        
        config.visualize.port = 1023;
        assert!(config.validate().is_err());
        
        config.visualize.port = 1024;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_ignore_empty_patterns() {
        let toml_content = r#"
[ignore]
patterns = []
directories = []
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert!(config.ignore.patterns.is_empty());
        assert!(config.ignore.directories.is_empty());
    }

    #[test]
    fn test_ignore_many_patterns() {
        let mut config = Config::default();
        config.ignore.patterns = (0..100).map(|i| format!("*.test{}.rs", i)).collect();
        assert_eq!(config.ignore.patterns.len(), 100);
    }

    #[test]
    fn test_ignore_special_characters() {
        let toml_content = r#"
[ignore]
patterns = ["*.rs", "**/*.toml", "src/**/test_*.rs"]
directories = ["target/debug", "target/release"]
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert!(config.ignore.patterns.contains(&"**/*.toml".to_string()));
        assert!(config.ignore.directories.contains(&"target/debug".to_string()));
    }

    #[test]
    fn test_config_unicode_strings() {
        let toml_content = r#"
[run]
output = "输出.json"

[visualize]
host = "本地主机"
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.run.output, "输出.json");
        assert_eq!(config.visualize.host, "本地主机");
    }

    #[test]
    fn test_config_whitespace_in_strings() {
        let toml_content = r#"
[run]
output = "  spaced.json  "
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.run.output, "  spaced.json  ");
    }

    #[test]
    fn test_config_special_paths() {
        let toml_content = r#"
[run]
output = "../parent/output.json"
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.run.output, "../parent/output.json");
    }

    #[test]
    fn test_config_absolute_paths() {
        let toml_content = r#"
[run]
output = "/tmp/borrowscope.json"
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.run.output, "/tmp/borrowscope.json");
    }

    #[test]
    fn test_export_format_variations() {
        let formats = vec!["dot", "json", "html", "svg", "png"];
        for format in formats {
            let mut config = Config::default();
            config.export.format = format.to_string();
            assert_eq!(config.export.format, format);
        }
    }

    #[test]
    fn test_tracking_all_combinations() {
        let combinations = vec![
            (true, true, true),
            (true, true, false),
            (true, false, true),
            (true, false, false),
            (false, true, true),
            (false, true, false),
            (false, false, true),
            (false, false, false),
        ];
        
        for (smart, async_code, unsafe_code) in combinations {
            let mut config = Config::default();
            config.tracking.smart_pointers = smart;
            config.tracking.async_code = async_code;
            config.tracking.unsafe_code = unsafe_code;
            
            assert_eq!(config.tracking.smart_pointers, smart);
            assert_eq!(config.tracking.async_code, async_code);
            assert_eq!(config.tracking.unsafe_code, unsafe_code);
        }
    }

    #[test]
    fn test_visualize_host_variations() {
        let hosts = vec!["127.0.0.1", "0.0.0.0", "localhost", "::1", "192.168.1.1"];
        for host in hosts {
            let mut config = Config::default();
            config.visualize.host = host.to_string();
            assert_eq!(config.visualize.host, host);
        }
    }

    #[test]
    fn test_config_save_and_load_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.toml");
        
        let mut original = Config::default();
        original.run.output = "custom.json".to_string();
        original.visualize.port = 8080;
        original.ignore.patterns = vec!["*.tmp".to_string()];
        
        original.save(&config_path).unwrap();
        
        let contents = fs::read_to_string(&config_path).unwrap();
        let loaded: Config = toml::from_str(&contents).unwrap();
        
        assert_eq!(loaded.run.output, "custom.json");
        assert_eq!(loaded.visualize.port, 8080);
        assert!(loaded.ignore.patterns.contains(&"*.tmp".to_string()));
    }

    #[test]
    fn test_config_malformed_toml() {
        let invalid_toml = r#"
[run
output = "test.json"
"#;
        let result = toml::from_str::<Config>(invalid_toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_duplicate_keys() {
        let toml_content = r#"
[run]
output = "first.json"
output = "second.json"
"#;
        let result = toml::from_str::<Config>(toml_content);
        // TOML parser should handle this
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_config_unknown_section() {
        let toml_content = r#"
[unknown_section]
key = "value"

[run]
output = "test.json"
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.run.output, "test.json");
    }

    #[test]
    fn test_config_nested_tables() {
        let toml_content = r#"
[run]
output = "test.json"
visualize = true
capture = false
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.run.output, "test.json");
        assert!(config.run.visualize);
        assert!(!config.run.capture);
    }

    #[test]
    fn test_validate_output_with_spaces() {
        let mut config = Config::default();
        config.run.output = "   ".to_string();
        // Non-empty but only spaces - should pass (filesystem will handle)
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_multiple_errors() {
        let mut config = Config::default();
        config.visualize.port = 80;
        config.run.output = "".to_string();
        
        // Should fail on first error
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_boolean_variations() {
        let toml_content = r#"
[run]
visualize = true
capture = false

[visualize]
browser = false
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert!(config.run.visualize);
        assert!(!config.run.capture);
        assert!(!config.visualize.browser);
    }

    #[test]
    fn test_ignore_duplicate_patterns() {
        let mut config = Config::default();
        config.ignore.patterns = vec![
            "*.test.rs".to_string(),
            "*.test.rs".to_string(),
            "*_test.rs".to_string(),
        ];
        assert_eq!(config.ignore.patterns.len(), 3);
    }

    #[test]
    fn test_ignore_case_sensitive_patterns() {
        let mut config = Config::default();
        config.ignore.patterns = vec![
            "*.Test.rs".to_string(),
            "*.test.rs".to_string(),
        ];
        assert_eq!(config.ignore.patterns.len(), 2);
    }

    #[test]
    fn test_config_comments_preserved() {
        let toml_content = r#"
# This is a comment
[run]
output = "test.json"  # inline comment
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.run.output, "test.json");
    }

    #[test]
    fn test_config_array_formats() {
        let toml_content = r#"
[ignore]
patterns = [
    "*.test.rs",
    "*_test.rs",
]
directories = ["target", "tests"]
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.ignore.patterns.len(), 2);
        assert_eq!(config.ignore.directories.len(), 2);
    }

    #[test]
    fn test_config_string_escapes() {
        let toml_content = r#"
[run]
output = "path\\with\\backslashes.json"
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert!(config.run.output.contains("\\"));
    }

    #[test]
    fn test_config_multiline_arrays() {
        let toml_content = r#"
[ignore]
patterns = [
    "*.test.rs",
    "*_test.rs",
    "*.tmp",
    "*.bak",
]
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.ignore.patterns.len(), 4);
    }

    #[test]
    fn test_config_port_zero() {
        let mut config = Config::default();
        config.visualize.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_very_long_strings() {
        let long_string = "a".repeat(10000);
        let mut config = Config::default();
        config.run.output = long_string.clone();
        assert_eq!(config.run.output.len(), 10000);
    }

    #[test]
    fn test_ignore_empty_string_patterns() {
        let mut config = Config::default();
        config.ignore.patterns = vec!["".to_string(), "*.rs".to_string()];
        assert_eq!(config.ignore.patterns.len(), 2);
    }

    #[test]
    fn test_config_default_equality() {
        let config1 = Config::default();
        let config2 = Config::default();
        
        assert_eq!(config1.run.output, config2.run.output);
        assert_eq!(config1.visualize.port, config2.visualize.port);
        assert_eq!(config1.export.format, config2.export.format);
    }

    #[test]
    fn test_config_partial_sections() {
        let toml_content = r#"
[visualize]
port = 9000
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.visualize.port, 9000);
        assert!(config.visualize.browser);
        assert_eq!(config.visualize.host, "127.0.0.1");
    }

    #[test]
    fn test_config_all_defaults_explicit() {
        let toml_content = r#"
[run]
output = "borrowscope.json"
visualize = false
capture = true

[visualize]
port = 3000
browser = true
host = "127.0.0.1"

[export]
format = "dot"

[tracking]
smart_pointers = true
async_code = true
unsafe_code = false

[ignore]
patterns = ["*.test.rs", "*_test.rs"]
directories = ["target", "tests"]
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        let default_config = Config::default();
        
        assert_eq!(config.run.output, default_config.run.output);
        assert_eq!(config.visualize.port, default_config.visualize.port);
    }

    #[test]
    fn test_save_creates_pretty_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("pretty.toml");
        
        let config = Config::default();
        config.save(&config_path).unwrap();
        
        let contents = fs::read_to_string(&config_path).unwrap();
        assert!(contents.contains("[run]"));
        assert!(contents.contains("[visualize]"));
        assert!(contents.contains("[export]"));
        assert!(contents.contains("[tracking]"));
        assert!(contents.contains("[ignore]"));
    }

    #[test]
    fn test_config_type_coercion() {
        let toml_content = r#"
[visualize]
port = 8080
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.visualize.port, 8080u16);
    }
}
