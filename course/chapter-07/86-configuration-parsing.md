# Section 86: Configuration File Parsing

**src/config.rs (Enhanced):**

```rust
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub run: RunConfig,
    pub visualize: VisualizeConfig,
    pub export: ExportConfig,
    pub tracking: TrackingConfig,
    pub ignore: IgnoreConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunConfig {
    pub output: String,
    pub visualize: bool,
    pub capture: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VisualizeConfig {
    pub port: u16,
    pub browser: bool,
    pub host: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportConfig {
    pub format: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackingConfig {
    pub smart_pointers: bool,
    pub async_code: bool,
    pub unsafe_code: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgnoreConfig {
    pub patterns: Vec<String>,
    pub directories: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            run: RunConfig {
                output: "borrowscope.json".into(),
                visualize: false,
                capture: true,
            },
            visualize: VisualizeConfig {
                port: 3000,
                browser: true,
                host: "127.0.0.1".into(),
            },
            export: ExportConfig {
                format: "dot".into(),
            },
            tracking: TrackingConfig {
                smart_pointers: true,
                async_code: true,
                unsafe_code: false,
            },
            ignore: IgnoreConfig {
                patterns: vec!["*.test.rs".into(), "*_test.rs".into()],
                directories: vec!["target".into(), "tests".into()],
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        if let Ok(config) = Self::load_from(".borrowscope.toml") {
            return Ok(config);
        }
        
        if let Some(config_dir) = directories::ProjectDirs::from("", "", "borrowscope") {
            let user_config = config_dir.config_dir().join("config.toml");
            if let Ok(config) = Self::load_from(&user_config) {
                return Ok(config);
            }
        }
        
        Ok(Self::default())
    }
    
    fn load_from<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
    
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    pub fn validate(&self) -> Result<()> {
        if self.visualize.port < 1024 {
            anyhow::bail!("Port must be >= 1024");
        }
        Ok(())
    }
}
```

**Progress:** 11/13
