//! Visualize command implementation

use std::fs;
use std::path::PathBuf;

use crate::cli::VisualizeArgs;
use crate::config::Config;
use crate::error::{CliError, Result};
use crate::server;
use crate::progress::spinner;

pub fn execute(args: VisualizeArgs, config: Config) -> Result<()> {
    log::info!("Visualizing: {}", args.file.display());

    // Check if file exists
    if !args.file.exists() {
        return Err(CliError::FileNotFound(args.file.clone()));
    }

    // Load and validate tracking data
    let data = load_tracking_data(&args.file)?;
    log::debug!("Loaded tracking data with {} events", data.event_count());

    // Determine port and host
    let port = args.port.unwrap_or(config.visualize.port);
    let host = args.host.clone().unwrap_or(config.visualize.host);

    // Check if port is available
    if port > 0 && !is_port_available(&host, port) {
        return Err(CliError::PortInUse(port));
    }

    // For testing, just validate and return
    #[cfg(test)]
    {
        println!("Visualization available at: http://{}:{}", host, port);
        return Ok(());
    }

    // Start web server (only in non-test builds)
    #[cfg(not(test))]
    {
        let sp = spinner("Starting visualization server");
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| CliError::Other(format!("Failed to create runtime: {}", e)))?;
        
        let (addr, mut shutdown_rx) = runtime.block_on(async {
            server::start_server(host.clone(), port, args.file.clone()).await
        }).map_err(|e| CliError::Other(format!("Failed to start server: {}", e)))?;
        
        sp.finish_with_message("✓ Server started");

        let url = format!("http://{}", addr);
        println!("\n🌐 Visualization server running at: {}", url);
        println!("   Press Ctrl+C to stop or visit /api/shutdown\n");

        // Open browser if requested
        if !args.no_browser && config.visualize.browser {
            open_browser(&url)?;
        }

        // Wait for shutdown signal
        runtime.block_on(async {
            let _ = shutdown_rx.recv().await;
        });

        println!("\n✓ Server stopped");
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
        std::process::Command::new("xdg-open").arg(url).spawn().ok();
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn().ok();
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
        assert!(matches!(result.unwrap_err(), CliError::FileNotFound(_)));
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
    fn test_load_empty_events() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("data.json");

        let data = serde_json::json!({
            "version": "0.1.0",
            "events": [],
            "graph": {"nodes": [], "edges": []}
        });

        fs::write(&test_file, serde_json::to_string(&data).unwrap()).unwrap();

        let loaded = load_tracking_data(&test_file).unwrap();
        assert_eq!(loaded.event_count(), 0);
    }

    #[test]
    fn test_load_missing_events() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("data.json");

        let data = serde_json::json!({
            "version": "0.1.0",
            "graph": {"nodes": [], "edges": []}
        });

        fs::write(&test_file, serde_json::to_string(&data).unwrap()).unwrap();

        let loaded = load_tracking_data(&test_file).unwrap();
        assert_eq!(loaded.event_count(), 0);
    }

    #[test]
    fn test_is_port_available() {
        // Port 0 should always be available (OS assigns)
        assert!(is_port_available("127.0.0.1", 0));
    }

    #[test]
    fn test_custom_port() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("data.json");

        let data = serde_json::json!({
            "version": "0.1.0",
            "events": [],
            "graph": {"nodes": [], "edges": []}
        });

        fs::write(&test_file, serde_json::to_string(&data).unwrap()).unwrap();

        let args = VisualizeArgs {
            file: test_file,
            port: Some(8080),
            no_browser: true,
            host: None,
        };
        let config = Config::default();

        let result = execute(args, config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_custom_host() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("data.json");

        let data = serde_json::json!({
            "version": "0.1.0",
            "events": [],
            "graph": {"nodes": [], "edges": []}
        });

        fs::write(&test_file, serde_json::to_string(&data).unwrap()).unwrap();

        let args = VisualizeArgs {
            file: test_file,
            port: None,
            no_browser: true,
            host: Some("0.0.0.0".to_string()),
        };
        let config = Config::default();

        let result = execute(args, config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("data.json");
        fs::write(&test_file, "invalid json").unwrap();

        let result = load_tracking_data(&test_file);
        assert!(result.is_err());
    }

    #[test]
    fn test_large_event_count() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("data.json");

        let events: Vec<_> = (0..1000)
            .map(|i| serde_json::json!({"type": "New", "id": i}))
            .collect();

        let data = serde_json::json!({
            "version": "0.1.0",
            "events": events,
            "graph": {"nodes": [], "edges": []}
        });

        fs::write(&test_file, serde_json::to_string(&data).unwrap()).unwrap();

        let loaded = load_tracking_data(&test_file).unwrap();
        assert_eq!(loaded.event_count(), 1000);
    }

    #[test]
    fn test_no_browser_flag() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("data.json");

        let data = serde_json::json!({
            "version": "0.1.0",
            "events": [],
            "graph": {"nodes": [], "edges": []}
        });

        fs::write(&test_file, serde_json::to_string(&data).unwrap()).unwrap();

        let args = VisualizeArgs {
            file: test_file,
            port: None,
            no_browser: true,
            host: None,
        };
        let config = Config::default();

        // Should not attempt to open browser
        let result = execute(args, config);
        assert!(result.is_ok());
    }
}
