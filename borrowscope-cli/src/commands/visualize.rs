//! Visualize command implementation

use std::fs;
use std::path::PathBuf;

use crate::cli::VisualizeArgs;
use crate::config::Config;
use crate::error::{CliError, Result};

pub fn execute(args: VisualizeArgs, config: Config) -> Result<()> {
    log::info!("Visualizing: {}", args.file.display());

    // Check if file exists
    if !args.file.exists() {
        return Err(CliError::FileNotFound(args.file));
    }

    // Load and validate tracking data
    let data = load_tracking_data(&args.file)?;
    log::debug!("Loaded tracking data with {} events", data.event_count());

    // Determine port and host
    let port = args.port.unwrap_or(config.visualize.port);
    let host = args.host.unwrap_or(config.visualize.host);

    // Check if port is available
    if !is_port_available(&host, port) {
        return Err(CliError::PortInUse(port));
    }

    log::info!("Starting visualization server on {}:{}", host, port);

    // TODO: Implement actual web server
    // For now, just print the URL
    let url = format!("http://{}:{}", host, port);
    println!("Visualization available at: {}", url);

    // Open browser if requested
    if !args.no_browser && config.visualize.browser {
        open_browser(&url)?;
    }

    Ok(())
}

struct TrackingData {
    events: Vec<serde_json::Value>,
}

impl TrackingData {
    fn event_count(&self) -> usize {
        self.events.len()
    }
}

fn load_tracking_data(path: &PathBuf) -> Result<TrackingData> {
    let contents = fs::read_to_string(path)?;
    let json: serde_json::Value = serde_json::from_str(&contents)?;

    let events = json
        .get("events")
        .and_then(|e| e.as_array())
        .cloned()
        .unwrap_or_default();

    Ok(TrackingData { events })
}

fn is_port_available(host: &str, port: u16) -> bool {
    use std::net::TcpListener;
    TcpListener::bind((host, port)).is_ok()
}

fn open_browser(url: &str) -> Result<()> {
    log::debug!("Opening browser: {}", url);

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .ok();
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .ok();
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(&["/C", "start", url])
            .spawn()
            .ok();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_visualize_nonexistent_file() {
        let args = VisualizeArgs {
            file: PathBuf::from("/nonexistent/file.json"),
            port: None,
            no_browser: true,
            host: None,
        };
        let config = Config::default();

        let result = execute(args, config);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_tracking_data() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("data.json");

        let data = serde_json::json!({
            "version": "0.1.0",
            "events": [
                {"type": "New", "id": 1},
                {"type": "Borrow", "id": 2}
            ],
            "graph": {
                "nodes": [],
                "edges": []
            }
        });

        fs::write(&test_file, serde_json::to_string(&data).unwrap()).unwrap();

        let loaded = load_tracking_data(&test_file).unwrap();
        assert_eq!(loaded.event_count(), 2);
    }

    #[test]
    fn test_is_port_available() {
        // Port 0 should always be available (OS assigns)
        assert!(is_port_available("127.0.0.1", 0));
    }
}
