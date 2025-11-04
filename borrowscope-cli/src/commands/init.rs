//! Init command implementation

use crate::cli::{ConfigTemplate, InitArgs};
use crate::config::Config;
use crate::error::{CliError, Result};

const CONFIG_FILE_NAME: &str = ".borrowscope.toml";

pub fn execute(args: InitArgs) -> Result<()> {
    log::info!("Initializing BorrowScope in: {}", args.path.display());

    // Check if directory exists
    if !args.path.exists() {
        return Err(CliError::FileNotFound(args.path.clone()));
    }

    if !args.path.is_dir() {
        return Err(CliError::Other(format!(
            "Not a directory: {}",
            args.path.display()
        )));
    }

    let config_path = args.path.join(CONFIG_FILE_NAME);

    // Check if config already exists
    if config_path.exists() && !args.force {
        return Err(CliError::ConfigError(format!(
            "Configuration file already exists: {}\nUse --force to overwrite",
            config_path.display()
        )));
    }

    // Create configuration based on template
    let config = match args.template {
        ConfigTemplate::Default => Config::default(),
        ConfigTemplate::Minimal => create_minimal_config(),
        ConfigTemplate::Advanced => create_advanced_config(),
    };

    // Save configuration
    config.save(&config_path)?;

    log::info!("Created configuration file: {}", config_path.display());
    println!("✓ Initialized BorrowScope configuration");
    println!("  Config file: {}", config_path.display());
    println!("\nNext steps:");
    println!("  1. Run: borrowscope run");
    println!("  2. Visualize: borrowscope visualize borrowscope.json");

    Ok(())
}

fn create_minimal_config() -> Config {
    Config {
        run: crate::config::RunConfig {
            output: "borrowscope.json".to_string(),
            visualize: false,
            capture: true,
        },
        visualize: crate::config::VisualizeConfig {
            port: 3000,
            browser: true,
            host: "127.0.0.1".to_string(),
        },
        export: crate::config::ExportConfig {
            format: "dot".to_string(),
        },
        tracking: crate::config::TrackingConfig {
            smart_pointers: false,
            async_code: false,
            unsafe_code: false,
        },
    }
}

fn create_advanced_config() -> Config {
    Config {
        run: crate::config::RunConfig {
            output: "borrowscope.json".to_string(),
            visualize: true,
            capture: true,
        },
        visualize: crate::config::VisualizeConfig {
            port: 3000,
            browser: true,
            host: "127.0.0.1".to_string(),
        },
        export: crate::config::ExportConfig {
            format: "html".to_string(),
        },
        tracking: crate::config::TrackingConfig {
            smart_pointers: true,
            async_code: true,
            unsafe_code: true,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_init_nonexistent_directory() {
        let args = InitArgs {
            path: PathBuf::from("/nonexistent/directory"),
            force: false,
            template: ConfigTemplate::Default,
        };

        let result = execute(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_init_creates_config() {
        let temp_dir = TempDir::new().unwrap();

        let args = InitArgs {
            path: temp_dir.path().to_path_buf(),
            force: false,
            template: ConfigTemplate::Default,
        };

        let result = execute(args);
        assert!(result.is_ok());

        let config_path = temp_dir.path().join(CONFIG_FILE_NAME);
        assert!(config_path.exists());
    }

    #[test]
    fn test_init_without_force_fails_if_exists() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(CONFIG_FILE_NAME);

        // Create existing config
        fs::write(&config_path, "# existing config").unwrap();

        let args = InitArgs {
            path: temp_dir.path().to_path_buf(),
            force: false,
            template: ConfigTemplate::Default,
        };

        let result = execute(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_init_with_force_overwrites() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(CONFIG_FILE_NAME);

        // Create existing config
        fs::write(&config_path, "# existing config").unwrap();

        let args = InitArgs {
            path: temp_dir.path().to_path_buf(),
            force: true,
            template: ConfigTemplate::Default,
        };

        let result = execute(args);
        assert!(result.is_ok());

        let contents = fs::read_to_string(&config_path).unwrap();
        assert!(!contents.contains("# existing config"));
    }

    #[test]
    fn test_init_minimal_template() {
        let temp_dir = TempDir::new().unwrap();

        let args = InitArgs {
            path: temp_dir.path().to_path_buf(),
            force: false,
            template: ConfigTemplate::Minimal,
        };

        let result = execute(args);
        assert!(result.is_ok());

        let config_path = temp_dir.path().join(CONFIG_FILE_NAME);
        let contents = fs::read_to_string(&config_path).unwrap();
        let config: Config = toml::from_str(&contents).unwrap();

        assert!(!config.tracking.smart_pointers);
        assert!(!config.tracking.async_code);
    }

    #[test]
    fn test_init_advanced_template() {
        let temp_dir = TempDir::new().unwrap();

        let args = InitArgs {
            path: temp_dir.path().to_path_buf(),
            force: false,
            template: ConfigTemplate::Advanced,
        };

        let result = execute(args);
        assert!(result.is_ok());

        let config_path = temp_dir.path().join(CONFIG_FILE_NAME);
        let contents = fs::read_to_string(&config_path).unwrap();
        let config: Config = toml::from_str(&contents).unwrap();

        assert!(config.tracking.smart_pointers);
        assert!(config.tracking.async_code);
        assert!(config.tracking.unsafe_code);
    }

    #[test]
    fn test_init_default_template() {
        let temp_dir = TempDir::new().unwrap();

        let args = InitArgs {
            path: temp_dir.path().to_path_buf(),
            force: false,
            template: ConfigTemplate::Default,
        };

        execute(args).unwrap();

        let config_path = temp_dir.path().join(CONFIG_FILE_NAME);
        let contents = fs::read_to_string(&config_path).unwrap();
        let config: Config = toml::from_str(&contents).unwrap();

        assert!(config.tracking.smart_pointers);
        assert!(config.tracking.async_code);
        assert!(!config.tracking.unsafe_code);
    }

    #[test]
    fn test_init_creates_valid_toml() {
        let temp_dir = TempDir::new().unwrap();

        let args = InitArgs {
            path: temp_dir.path().to_path_buf(),
            force: false,
            template: ConfigTemplate::Default,
        };

        execute(args).unwrap();

        let config_path = temp_dir.path().join(CONFIG_FILE_NAME);
        let contents = fs::read_to_string(&config_path).unwrap();

        // Should parse without error
        let result = toml::from_str::<Config>(&contents);
        assert!(result.is_ok());
    }

    #[test]
    fn test_init_config_has_all_sections() {
        let temp_dir = TempDir::new().unwrap();

        let args = InitArgs {
            path: temp_dir.path().to_path_buf(),
            force: false,
            template: ConfigTemplate::Default,
        };

        execute(args).unwrap();

        let config_path = temp_dir.path().join(CONFIG_FILE_NAME);
        let contents = fs::read_to_string(&config_path).unwrap();

        assert!(contents.contains("[run]"));
        assert!(contents.contains("[visualize]"));
        assert!(contents.contains("[export]"));
        assert!(contents.contains("[tracking]"));
    }

    #[test]
    fn test_init_preserves_directory_structure() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();

        let args = InitArgs {
            path: temp_dir.path().to_path_buf(),
            force: false,
            template: ConfigTemplate::Default,
        };

        execute(args).unwrap();

        // Subdirectory should still exist
        assert!(subdir.exists());
    }

    #[test]
    fn test_init_multiple_templates() {
        let temp_dir = TempDir::new().unwrap();

        for template in [
            ConfigTemplate::Default,
            ConfigTemplate::Minimal,
            ConfigTemplate::Advanced,
        ] {
            let subdir = temp_dir.path().join(format!("{:?}", template));
            fs::create_dir(&subdir).unwrap();

            let args = InitArgs {
                path: subdir.clone(),
                force: false,
                template,
            };

            let result = execute(args);
            assert!(result.is_ok());

            let config_path = subdir.join(CONFIG_FILE_NAME);
            assert!(config_path.exists());
        }
    }
}
