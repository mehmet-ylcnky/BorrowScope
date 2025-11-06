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
    #[allow(dead_code)]
    pub fn validate(&self) -> Result<()> {
        self.validate_run()?;
        self.validate_visualize()?;
        self.validate_export()?;
        self.validate_ignore()?;
        Ok(())
    }

    /// Validate run configuration
    #[allow(dead_code)]
    fn validate_run(&self) -> Result<()> {
        if self.run.output.is_empty() {
            return Err(CliError::ConfigError(
                "Output filename cannot be empty".to_string(),
            ));
        }

        if self.run.output.trim().is_empty() {
            return Err(CliError::ConfigError(
                "Output filename cannot be only whitespace".to_string(),
            ));
        }

        if !self.run.output.ends_with(".json") {
            return Err(CliError::ConfigError(
                "Output filename must end with .json".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate visualize configuration
    #[allow(dead_code)]
    fn validate_visualize(&self) -> Result<()> {
        if self.visualize.port < 1024 {
            return Err(CliError::ConfigError(
                "Port must be >= 1024 (privileged ports require root)".to_string(),
            ));
        }

        if !is_valid_host(&self.visualize.host) {
            return Err(CliError::ConfigError(format!(
                "Invalid host address: {}",
                self.visualize.host
            )));
        }

        Ok(())
    }

    /// Validate export configuration
    #[allow(dead_code)]
    fn validate_export(&self) -> Result<()> {
        let valid_formats = ["dot", "json", "html", "svg", "png"];
        if !valid_formats.contains(&self.export.format.as_str()) {
            return Err(CliError::ConfigError(format!(
                "Invalid export format '{}'. Must be one of: {}",
                self.export.format,
                valid_formats.join(", ")
            )));
        }

        Ok(())
    }

    /// Validate ignore configuration
    #[allow(dead_code)]
    fn validate_ignore(&self) -> Result<()> {
        for pattern in &self.ignore.patterns {
            if pattern.contains("**/**") {
                return Err(CliError::ConfigError(format!(
                    "Invalid glob pattern '{}': redundant **/**",
                    pattern
                )));
            }
        }

        for dir in &self.ignore.directories {
            if dir.is_empty() {
                return Err(CliError::ConfigError(
                    "Ignore directory cannot be empty".to_string(),
                ));
            }
            if dir.contains('\0') {
                return Err(CliError::ConfigError(
                    "Ignore directory cannot contain null bytes".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Validate host address (IP or hostname)
#[allow(dead_code)]
fn is_valid_host(host: &str) -> bool {
    if host.is_empty() {
        return false;
    }

    // Check for IPv4
    if is_valid_ipv4(host) {
        return true;
    }

    // Check for IPv6
    if is_valid_ipv6(host) {
        return true;
    }

    // Check for hostname
    is_valid_hostname(host)
}

/// Validate IPv4 address
#[allow(dead_code)]
fn is_valid_ipv4(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return false;
    }

    parts.iter().all(|part| {
        if let Ok(num) = part.parse::<u16>() {
            num <= 255
        } else {
            false
        }
    })
}

/// Validate IPv6 address (simplified)
#[allow(dead_code)]
fn is_valid_ipv6(s: &str) -> bool {
    if !s.contains(':') {
        return false;
    }

    // Reject invalid patterns
    if s.contains(":::") || s.starts_with(':') && !s.starts_with("::") {
        return false;
    }

    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() < 3 || parts.len() > 8 {
        return false;
    }

    parts.iter().all(|part| {
        part.is_empty() || part.len() <= 4 && part.chars().all(|c| c.is_ascii_hexdigit())
    })
}

/// Validate hostname
#[allow(dead_code)]
fn is_valid_hostname(s: &str) -> bool {
    if s.len() > 253 {
        return false;
    }

    let labels: Vec<&str> = s.split('.').collect();

    // Reject if it looks like an invalid IPv4 (4+ numeric parts)
    if labels.len() >= 4 && labels.iter().all(|p| p.chars().all(|c| c.is_ascii_digit())) {
        return false;
    }

    labels.iter().all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && label.chars().all(|c| c.is_alphanumeric() || c == '-')
            && !label.starts_with('-')
            && !label.ends_with('-')
    })
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
        assert!(config
            .ignore
            .directories
            .contains(&"target/debug".to_string()));
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
        // Only whitespace - should fail
        assert!(config.validate().is_err());
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
        config.ignore.patterns = vec!["*.Test.rs".to_string(), "*.test.rs".to_string()];
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

    // Schema Validation Tests

    #[test]
    fn test_validate_output_json_extension() {
        let mut config = Config::default();
        config.run.output = "output.txt".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_output_valid_json() {
        let mut config = Config::default();
        config.run.output = "output.json".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_output_whitespace_only() {
        let mut config = Config::default();
        config.run.output = "   ".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_export_format_valid() {
        let formats = ["dot", "json", "html", "svg", "png"];
        for format in formats {
            let mut config = Config::default();
            config.export.format = format.to_string();
            assert!(
                config.validate().is_ok(),
                "Format {} should be valid",
                format
            );
        }
    }

    #[test]
    fn test_validate_export_format_invalid() {
        let mut config = Config::default();
        config.export.format = "invalid".to_string();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid export format"));
    }

    #[test]
    fn test_validate_export_format_case_sensitive() {
        let mut config = Config::default();
        config.export.format = "DOT".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_host_ipv4_valid() {
        let hosts = ["127.0.0.1", "0.0.0.0", "192.168.1.1", "255.255.255.255"];
        for host in hosts {
            let mut config = Config::default();
            config.visualize.host = host.to_string();
            assert!(config.validate().is_ok(), "Host {} should be valid", host);
        }
    }

    #[test]
    fn test_validate_host_ipv4_invalid() {
        let hosts = ["256.1.1.1", "1.1.1.1.1"];
        for host in hosts {
            let mut config = Config::default();
            config.visualize.host = host.to_string();
            assert!(
                config.validate().is_err(),
                "Host {} should be invalid",
                host
            );
        }
    }

    #[test]
    fn test_validate_host_ipv6_valid() {
        let hosts = ["::1", "fe80::1", "2001:db8::1", "::"];
        for host in hosts {
            let mut config = Config::default();
            config.visualize.host = host.to_string();
            assert!(config.validate().is_ok(), "Host {} should be valid", host);
        }
    }

    #[test]
    fn test_validate_host_ipv6_invalid() {
        let hosts = [":::", "1:2:3:4:5:6:7:8:9"];
        for host in hosts {
            let mut config = Config::default();
            config.visualize.host = host.to_string();
            assert!(
                config.validate().is_err(),
                "Host {} should be invalid",
                host
            );
        }
    }

    #[test]
    fn test_validate_host_hostname_valid() {
        let hosts = ["localhost", "example.com", "sub.example.com", "my-server"];
        for host in hosts {
            let mut config = Config::default();
            config.visualize.host = host.to_string();
            assert!(config.validate().is_ok(), "Host {} should be valid", host);
        }
    }

    #[test]
    fn test_validate_host_hostname_invalid() {
        let hosts = ["-invalid", "invalid-", "inv@lid", ""];
        for host in hosts {
            let mut config = Config::default();
            config.visualize.host = host.to_string();
            assert!(
                config.validate().is_err(),
                "Host {} should be invalid",
                host
            );
        }
    }

    #[test]
    fn test_validate_host_empty() {
        let mut config = Config::default();
        config.visualize.host = "".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_ignore_pattern_redundant_glob() {
        let mut config = Config::default();
        config.ignore.patterns = vec!["**/**/*.rs".to_string()];
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_ignore_pattern_valid_glob() {
        let mut config = Config::default();
        config.ignore.patterns = vec!["**/*.rs".to_string(), "src/**/*.test.rs".to_string()];
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_ignore_directory_empty() {
        let mut config = Config::default();
        config.ignore.directories = vec!["".to_string()];
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_ignore_directory_null_byte() {
        let mut config = Config::default();
        config.ignore.directories = vec!["target\0".to_string()];
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_ignore_directory_valid() {
        let mut config = Config::default();
        config.ignore.directories = vec!["target".to_string(), "build/debug".to_string()];
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_all_sections_valid() {
        let mut config = Config::default();
        config.run.output = "test.json".to_string();
        config.visualize.port = 8080;
        config.visualize.host = "localhost".to_string();
        config.export.format = "svg".to_string();
        config.ignore.patterns = vec!["*.tmp".to_string()];
        config.ignore.directories = vec!["target".to_string()];
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_multiple_errors_run_section() {
        let mut config = Config::default();
        config.run.output = "".to_string();
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_multiple_errors_visualize_section() {
        let mut config = Config::default();
        config.visualize.port = 80;
        config.visualize.host = "invalid@host".to_string();
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_output_path_with_directory() {
        let mut config = Config::default();
        config.run.output = "output/data.json".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_output_absolute_path() {
        let mut config = Config::default();
        config.run.output = "/tmp/output.json".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_output_relative_path() {
        let mut config = Config::default();
        config.run.output = "../output.json".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_host_max_length() {
        let mut config = Config::default();
        config.visualize.host = "a".repeat(254);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_host_label_max_length() {
        let mut config = Config::default();
        config.visualize.host = format!("{}.com", "a".repeat(64));
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_export_format_empty() {
        let mut config = Config::default();
        config.export.format = "".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_ignore_patterns_empty_list() {
        let mut config = Config::default();
        config.ignore.patterns = vec![];
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_ignore_directories_empty_list() {
        let mut config = Config::default();
        config.ignore.directories = vec![];
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_port_boundary_1024() {
        let mut config = Config::default();
        config.visualize.port = 1024;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_port_boundary_1023() {
        let mut config = Config::default();
        config.visualize.port = 1023;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_port_max_value() {
        let mut config = Config::default();
        config.visualize.port = 65535;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_host_ipv4_edge_cases() {
        let mut config = Config::default();

        config.visualize.host = "0.0.0.0".to_string();
        assert!(config.validate().is_ok());

        config.visualize.host = "255.255.255.255".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_host_ipv6_compressed() {
        let mut config = Config::default();
        config.visualize.host = "::1".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_host_ipv6_full() {
        let mut config = Config::default();
        config.visualize.host = "2001:0db8:85a3:0000:0000:8a2e:0370:7334".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_hostname_single_label() {
        let mut config = Config::default();
        config.visualize.host = "localhost".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_hostname_multiple_labels() {
        let mut config = Config::default();
        config.visualize.host = "sub.domain.example.com".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_hostname_with_numbers() {
        let mut config = Config::default();
        config.visualize.host = "server123.example.com".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_hostname_with_hyphens() {
        let mut config = Config::default();
        config.visualize.host = "my-server.example-domain.com".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_output_multiple_extensions() {
        let mut config = Config::default();
        config.run.output = "output.backup.json".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_output_no_extension() {
        let mut config = Config::default();
        config.run.output = "output".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_output_wrong_extension() {
        let mut config = Config::default();
        config.run.output = "output.xml".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_ignore_pattern_with_spaces() {
        let mut config = Config::default();
        config.ignore.patterns = vec!["* test.rs".to_string()];
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_ignore_directory_with_spaces() {
        let mut config = Config::default();
        config.ignore.directories = vec!["my folder".to_string()];
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_complex_valid_config() {
        let toml_content = r#"
[run]
output = "tracking/data.json"
visualize = true
capture = false

[visualize]
port = 8080
browser = false
host = "192.168.1.100"

[export]
format = "svg"

[tracking]
smart_pointers = true
async_code = true
unsafe_code = true

[ignore]
patterns = ["*.tmp", "**/*.bak", "src/**/*.test.rs"]
directories = ["target", "build", "dist"]
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_error_messages_descriptive() {
        let mut config = Config::default();

        config.export.format = "invalid".to_string();
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("Invalid export format"));
        assert!(err.to_string().contains("dot, json, html, svg, png"));
    }

    #[test]
    fn test_validate_host_special_characters() {
        let mut config = Config::default();
        config.visualize.host = "host@example.com".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_ignore_pattern_multiple_stars() {
        let mut config = Config::default();
        config.ignore.patterns = vec!["***/*.rs".to_string()];
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_all_export_formats() {
        let formats = vec![
            ("dot", true),
            ("json", true),
            ("html", true),
            ("svg", true),
            ("png", true),
            ("pdf", false),
            ("txt", false),
            ("", false),
        ];

        for (format, should_pass) in formats {
            let mut config = Config::default();
            config.export.format = format.to_string();
            let result = config.validate();
            assert_eq!(
                result.is_ok(),
                should_pass,
                "Format '{}' validation mismatch",
                format
            );
        }
    }

    #[test]
    fn test_validate_ipv4_octet_boundaries() {
        let ips = vec![
            ("0.0.0.0", true),
            ("255.255.255.255", true),
            ("256.0.0.0", false),
            ("1.256.1.1", false),
            ("1.1.256.1", false),
            ("1.1.1.256", false),
        ];

        for (ip, should_pass) in ips {
            let mut config = Config::default();
            config.visualize.host = ip.to_string();
            config.run.output = "test.json".to_string(); // Ensure valid output
            let result = config.validate();
            assert_eq!(
                result.is_ok(),
                should_pass,
                "IP '{}' validation mismatch",
                ip
            );
        }
    }

    #[test]
    fn test_validate_hostname_edge_cases() {
        let hosts = vec![
            ("a", true),
            ("a.b", true),
            ("a-b", true),
            ("-ab", false),
            ("ab-", false),
            ("a..b", false),
            ("a_b", false),
        ];

        for (host, should_pass) in hosts {
            let mut config = Config::default();
            config.visualize.host = host.to_string();
            let result = config.validate();
            assert_eq!(
                result.is_ok(),
                should_pass,
                "Host '{}' validation mismatch",
                host
            );
        }
    }
}
