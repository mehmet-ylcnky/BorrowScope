//! Init command implementation

use crate::cli::{ConfigTemplate, InitArgs};
use crate::config::Config;
use crate::error::{CliError, Result};
use std::fs;
use std::path::Path;

const CONFIG_FILE_NAME: &str = ".borrowscope.toml";
const GITIGNORE_FILE: &str = ".gitignore";

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

    // Detect project type
    let is_rust_project = detect_rust_project(&args.path);
    if !is_rust_project {
        log::warn!("No Cargo.toml found - this may not be a Rust project");
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

    // Update .gitignore if it exists
    update_gitignore(&args.path)?;

    log::info!("Created configuration file: {}", config_path.display());
    println!("✓ Initialized BorrowScope configuration");
    println!("  Config file: {}", config_path.display());
    if is_rust_project {
        println!("  Project type: Rust");
    }
    println!("\nNext steps:");
    println!("  1. Run: borrowscope run");
    println!("  2. Visualize: borrowscope visualize borrowscope.json");

    Ok(())
}

fn detect_rust_project(path: &Path) -> bool {
    path.join("Cargo.toml").exists()
}

fn update_gitignore(path: &Path) -> Result<()> {
    let gitignore_path = path.join(GITIGNORE_FILE);
    
    if !gitignore_path.exists() {
        return Ok(());
    }

    let contents = fs::read_to_string(&gitignore_path)?;
    
    // Check if borrowscope entries already exist
    if contents.contains("borrowscope.json") && contents.contains(".borrowscope/") {
        return Ok(());
    }

    // Append borrowscope entries
    let mut new_contents = contents.clone();
    if !new_contents.ends_with('\n') {
        new_contents.push('\n');
    }
    
    new_contents.push_str("\n# BorrowScope\n");
    if !contents.contains("borrowscope.json") {
        new_contents.push_str("borrowscope.json\n");
    }
    if !contents.contains(".borrowscope/") {
        new_contents.push_str(".borrowscope/\n");
    }

    fs::write(&gitignore_path, new_contents)?;
    log::info!("Updated .gitignore");
    
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
        ignore: crate::config::IgnoreConfig {
            patterns: vec![],
            directories: vec!["target".to_string()],
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
        ignore: crate::config::IgnoreConfig {
            patterns: vec!["*.test.rs".to_string(), "*_test.rs".to_string()],
            directories: vec!["target".to_string(), "tests".to_string(), "benches".to_string()],
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

    #[test]
    fn test_detect_rust_project_with_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        
        assert!(detect_rust_project(temp_dir.path()));
    }

    #[test]
    fn test_detect_rust_project_without_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        assert!(!detect_rust_project(temp_dir.path()));
    }

    #[test]
    fn test_update_gitignore_creates_entries() {
        let temp_dir = TempDir::new().unwrap();
        let gitignore = temp_dir.path().join(".gitignore");
        fs::write(&gitignore, "*.log\n").unwrap();

        update_gitignore(temp_dir.path()).unwrap();

        let contents = fs::read_to_string(&gitignore).unwrap();
        assert!(contents.contains("borrowscope.json"));
        assert!(contents.contains(".borrowscope/"));
    }

    #[test]
    fn test_update_gitignore_no_duplicates() {
        let temp_dir = TempDir::new().unwrap();
        let gitignore = temp_dir.path().join(".gitignore");
        fs::write(&gitignore, "borrowscope.json\n.borrowscope/\n").unwrap();

        update_gitignore(temp_dir.path()).unwrap();

        let contents = fs::read_to_string(&gitignore).unwrap();
        assert_eq!(contents.matches("borrowscope.json").count(), 1);
        assert_eq!(contents.matches(".borrowscope/").count(), 1);
    }

    #[test]
    fn test_update_gitignore_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let result = update_gitignore(temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_init_with_rust_project() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

        let args = InitArgs {
            path: temp_dir.path().to_path_buf(),
            force: false,
            template: ConfigTemplate::Default,
        };

        let result = execute(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_init_updates_gitignore() {
        let temp_dir = TempDir::new().unwrap();
        let gitignore = temp_dir.path().join(".gitignore");
        fs::write(&gitignore, "*.log\n").unwrap();

        let args = InitArgs {
            path: temp_dir.path().to_path_buf(),
            force: false,
            template: ConfigTemplate::Default,
        };

        execute(args).unwrap();

        let contents = fs::read_to_string(&gitignore).unwrap();
        assert!(contents.contains("borrowscope.json"));
    }

    #[test]
    fn test_init_file_not_directory() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("file.txt");
        fs::write(&file_path, "test").unwrap();

        let args = InitArgs {
            path: file_path,
            force: false,
            template: ConfigTemplate::Default,
        };

        let result = execute(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_init_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        let args = InitArgs {
            path: temp_dir.path().to_path_buf(),
            force: false,
            template: ConfigTemplate::Default,
        };

        let result = execute(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_init_nested_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested = temp_dir.path().join("a").join("b").join("c");
        fs::create_dir_all(&nested).unwrap();

        let args = InitArgs {
            path: nested.clone(),
            force: false,
            template: ConfigTemplate::Default,
        };

        let result = execute(args);
        assert!(result.is_ok());

        let config_path = nested.join(CONFIG_FILE_NAME);
        assert!(config_path.exists());
    }

    #[test]
    fn test_minimal_config_has_minimal_settings() {
        let config = create_minimal_config();
        assert!(!config.tracking.smart_pointers);
        assert!(!config.tracking.async_code);
        assert!(!config.tracking.unsafe_code);
        assert!(!config.run.visualize);
    }

    #[test]
    fn test_advanced_config_has_all_features() {
        let config = create_advanced_config();
        assert!(config.tracking.smart_pointers);
        assert!(config.tracking.async_code);
        assert!(config.tracking.unsafe_code);
        assert!(config.run.visualize);
    }

    #[test]
    fn test_config_templates_differ() {
        let minimal = create_minimal_config();
        let advanced = create_advanced_config();
        
        assert_ne!(minimal.tracking.smart_pointers, advanced.tracking.smart_pointers);
        assert_ne!(minimal.tracking.async_code, advanced.tracking.async_code);
    }

    #[test]
    fn test_init_with_existing_files() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("src.rs"), "fn main() {}").unwrap();

        let args = InitArgs {
            path: temp_dir.path().to_path_buf(),
            force: false,
            template: ConfigTemplate::Default,
        };

        let result = execute(args);
        assert!(result.is_ok());
        assert!(temp_dir.path().join("src.rs").exists());
    }

    #[test]
    fn test_update_gitignore_preserves_content() {
        let temp_dir = TempDir::new().unwrap();
        let gitignore = temp_dir.path().join(".gitignore");
        let original = "*.log\ntarget/\n";
        fs::write(&gitignore, original).unwrap();

        update_gitignore(temp_dir.path()).unwrap();

        let contents = fs::read_to_string(&gitignore).unwrap();
        assert!(contents.contains("*.log"));
        assert!(contents.contains("target/"));
    }

    #[test]
    fn test_update_gitignore_adds_newline() {
        let temp_dir = TempDir::new().unwrap();
        let gitignore = temp_dir.path().join(".gitignore");
        fs::write(&gitignore, "*.log").unwrap(); // No trailing newline

        update_gitignore(temp_dir.path()).unwrap();

        let contents = fs::read_to_string(&gitignore).unwrap();
        assert!(contents.contains("borrowscope.json"));
    }

    #[test]
    fn test_init_config_is_loadable() {
        let temp_dir = TempDir::new().unwrap();

        let args = InitArgs {
            path: temp_dir.path().to_path_buf(),
            force: false,
            template: ConfigTemplate::Default,
        };

        execute(args).unwrap();

        let config_path = temp_dir.path().join(CONFIG_FILE_NAME);
        let contents = fs::read_to_string(&config_path).unwrap();
        let loaded = toml::from_str::<Config>(&contents);
        assert!(loaded.is_ok());
    }

    #[test]
    fn test_init_all_templates_loadable() {
        let temp_dir = TempDir::new().unwrap();

        for template in [ConfigTemplate::Default, ConfigTemplate::Minimal, ConfigTemplate::Advanced] {
            let subdir = temp_dir.path().join(format!("{:?}", template));
            fs::create_dir(&subdir).unwrap();

            let args = InitArgs {
                path: subdir.clone(),
                force: false,
                template,
            };

            execute(args).unwrap();

            let config_path = subdir.join(CONFIG_FILE_NAME);
            let contents = fs::read_to_string(&config_path).unwrap();
            let loaded = toml::from_str::<Config>(&contents);
            assert!(loaded.is_ok());
        }
    }

    #[test]
    fn test_detect_rust_project_case_sensitive() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("cargo.toml"), "[package]").unwrap(); // lowercase
        
        // Should not detect lowercase cargo.toml
        assert!(!detect_rust_project(temp_dir.path()));
    }

    #[test]
    fn test_init_with_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let real_dir = temp_dir.path().join("real");
        fs::create_dir(&real_dir).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let link_dir = temp_dir.path().join("link");
            symlink(&real_dir, &link_dir).unwrap();

            let args = InitArgs {
                path: link_dir.clone(),
                force: false,
                template: ConfigTemplate::Default,
            };

            let result = execute(args);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_update_gitignore_partial_entries() {
        let temp_dir = TempDir::new().unwrap();
        let gitignore = temp_dir.path().join(".gitignore");
        fs::write(&gitignore, "borrowscope.json\n").unwrap(); // Only one entry

        update_gitignore(temp_dir.path()).unwrap();

        let contents = fs::read_to_string(&gitignore).unwrap();
        assert!(contents.contains(".borrowscope/"));
        assert_eq!(contents.matches("borrowscope.json").count(), 1);
    }

    #[test]
    fn test_init_readonly_directory() {
        let temp_dir = TempDir::new().unwrap();
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(temp_dir.path()).unwrap().permissions();
            perms.set_mode(0o444); // Read-only
            fs::set_permissions(temp_dir.path(), perms).unwrap();

            let args = InitArgs {
                path: temp_dir.path().to_path_buf(),
                force: false,
                template: ConfigTemplate::Default,
            };

            let result = execute(args);
            assert!(result.is_err());

            // Restore permissions for cleanup
            let mut perms = fs::metadata(temp_dir.path()).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(temp_dir.path(), perms).unwrap();
        }
    }

    #[test]
    fn test_config_output_filenames() {
        let minimal = create_minimal_config();
        let advanced = create_advanced_config();
        
        assert_eq!(minimal.run.output, "borrowscope.json");
        assert_eq!(advanced.run.output, "borrowscope.json");
    }

    #[test]
    fn test_config_ignore_patterns() {
        let minimal = create_minimal_config();
        let advanced = create_advanced_config();
        
        assert!(minimal.ignore.patterns.is_empty());
        assert!(!advanced.ignore.patterns.is_empty());
    }
}
