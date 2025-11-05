//! Web server for visualization

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Server state
#[derive(Clone)]
pub struct ServerState {
    pub data_file: PathBuf,
    pub shutdown_tx: broadcast::Sender<()>,
}

/// Start the web server
pub async fn start_server(
    host: String,
    port: u16,
    data_file: PathBuf,
) -> anyhow::Result<(SocketAddr, broadcast::Receiver<()>)> {
    let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
    
    let state = ServerState {
        data_file,
        shutdown_tx: shutdown_tx.clone(),
    };

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/data", get(data_handler))
        .route("/api/health", get(health_handler))
        .route("/api/shutdown", get(shutdown_handler))
        .with_state(Arc::new(state));

    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let actual_addr = listener.local_addr()?;
    
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("Server failed");
    });

    Ok((actual_addr, shutdown_rx))
}

/// Index page handler
async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

/// Data API handler
async fn data_handler(State(state): State<Arc<ServerState>>) -> Response {
    match std::fs::read_to_string(&state.data_file) {
        Ok(content) => {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(json) => Json(json).into_response(),
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Invalid JSON").into_response(),
            }
        }
        Err(_) => (StatusCode::NOT_FOUND, "Data file not found").into_response(),
    }
}

/// Health check handler
async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Shutdown handler
async fn shutdown_handler(State(state): State<Arc<ServerState>>) -> &'static str {
    let _ = state.shutdown_tx.send(());
    "Shutting down..."
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_start_server() {
        let temp_dir = TempDir::new().unwrap();
        let data_file = temp_dir.path().join("data.json");
        fs::write(&data_file, r#"{"test": "data"}"#).unwrap();

        let result = start_server("127.0.0.1".to_string(), 0, data_file).await;
        assert!(result.is_ok());
        
        let (addr, _rx) = result.unwrap();
        assert!(addr.port() > 0);
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let temp_dir = TempDir::new().unwrap();
        let data_file = temp_dir.path().join("data.json");
        fs::write(&data_file, r#"{"test": "data"}"#).unwrap();

        let (addr, _rx) = start_server("127.0.0.1".to_string(), 0, data_file).await.unwrap();
        
        let client = reqwest::Client::new();
        let url = format!("http://{}/api/health", addr);
        let response = client.get(&url).send().await.unwrap();
        
        assert_eq!(response.status(), 200);
        let json: serde_json::Value = response.json().await.unwrap();
        assert_eq!(json["status"], "ok");
    }

    #[tokio::test]
    async fn test_data_endpoint() {
        let temp_dir = TempDir::new().unwrap();
        let data_file = temp_dir.path().join("data.json");
        fs::write(&data_file, r#"{"test": "data"}"#).unwrap();

        let (addr, _rx) = start_server("127.0.0.1".to_string(), 0, data_file).await.unwrap();
        
        let client = reqwest::Client::new();
        let url = format!("http://{}/api/data", addr);
        let response = client.get(&url).send().await.unwrap();
        
        assert_eq!(response.status(), 200);
        let json: serde_json::Value = response.json().await.unwrap();
        assert_eq!(json["test"], "data");
    }

    #[tokio::test]
    async fn test_data_endpoint_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let data_file = temp_dir.path().join("nonexistent.json");

        let (addr, _rx) = start_server("127.0.0.1".to_string(), 0, data_file).await.unwrap();
        
        let client = reqwest::Client::new();
        let url = format!("http://{}/api/data", addr);
        let response = client.get(&url).send().await.unwrap();
        
        assert_eq!(response.status(), 404);
    }

    #[tokio::test]
    async fn test_shutdown_endpoint() {
        let temp_dir = TempDir::new().unwrap();
        let data_file = temp_dir.path().join("data.json");
        fs::write(&data_file, r#"{"test": "data"}"#).unwrap();

        let (addr, mut rx) = start_server("127.0.0.1".to_string(), 0, data_file).await.unwrap();
        
        let client = reqwest::Client::new();
        let url = format!("http://{}/api/shutdown", addr);
        let response = client.get(&url).send().await.unwrap();
        
        assert_eq!(response.status(), 200);
        
        // Should receive shutdown signal
        tokio::select! {
            _ = rx.recv() => {},
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                panic!("Shutdown signal not received");
            }
        }
    }

    #[tokio::test]
    async fn test_index_endpoint() {
        let temp_dir = TempDir::new().unwrap();
        let data_file = temp_dir.path().join("data.json");
        fs::write(&data_file, r#"{"test": "data"}"#).unwrap();

        let (addr, _rx) = start_server("127.0.0.1".to_string(), 0, data_file).await.unwrap();
        
        let client = reqwest::Client::new();
        let url = format!("http://{}/", addr);
        let response = client.get(&url).send().await.unwrap();
        
        assert_eq!(response.status(), 200);
        let text = response.text().await.unwrap();
        assert!(text.contains("html") || text.contains("HTML"));
    }

    #[tokio::test]
    async fn test_server_on_random_port() {
        let temp_dir = TempDir::new().unwrap();
        let data_file = temp_dir.path().join("data.json");
        fs::write(&data_file, r#"{}"#).unwrap();

        let (addr, _rx) = start_server("127.0.0.1".to_string(), 0, data_file).await.unwrap();
        assert!(addr.port() > 0);
        assert!(addr.port() <= 65535);
    }

    #[tokio::test]
    async fn test_server_on_specific_port() {
        let temp_dir = TempDir::new().unwrap();
        let data_file = temp_dir.path().join("data.json");
        fs::write(&data_file, r#"{}"#).unwrap();

        // Use a high port to avoid conflicts
        let port = 50000 + (std::process::id() % 10000) as u16;
        let result = start_server("127.0.0.1".to_string(), port, data_file).await;
        
        if let Ok((addr, _rx)) = result {
            assert_eq!(addr.port(), port);
        }
    }

    #[tokio::test]
    async fn test_concurrent_requests() {
        let temp_dir = TempDir::new().unwrap();
        let data_file = temp_dir.path().join("data.json");
        fs::write(&data_file, r#"{"test": "data"}"#).unwrap();

        let (addr, _rx) = start_server("127.0.0.1".to_string(), 0, data_file).await.unwrap();
        
        let client = reqwest::Client::new();
        let url = format!("http://{}/api/health", addr);
        
        let mut handles = vec![];
        for _ in 0..10 {
            let client = client.clone();
            let url = url.clone();
            handles.push(tokio::spawn(async move {
                client.get(&url).send().await.unwrap().status()
            }));
        }
        
        for handle in handles {
            let status = handle.await.unwrap();
            assert_eq!(status, 200);
        }
    }

    #[tokio::test]
    async fn test_invalid_json_data() {
        let temp_dir = TempDir::new().unwrap();
        let data_file = temp_dir.path().join("data.json");
        fs::write(&data_file, "invalid json {{{").unwrap();

        let (addr, _rx) = start_server("127.0.0.1".to_string(), 0, data_file).await.unwrap();
        
        let client = reqwest::Client::new();
        let url = format!("http://{}/api/data", addr);
        let response = client.get(&url).send().await.unwrap();
        
        assert_eq!(response.status(), 500);
    }

    #[tokio::test]
    async fn test_large_data_file() {
        let temp_dir = TempDir::new().unwrap();
        let data_file = temp_dir.path().join("data.json");
        
        let events: Vec<_> = (0..1000)
            .map(|i| serde_json::json!({"id": i}))
            .collect();
        
        let large_data = serde_json::json!({
            "events": events
        });
        fs::write(&data_file, serde_json::to_string(&large_data).unwrap()).unwrap();

        let (addr, _rx) = start_server("127.0.0.1".to_string(), 0, data_file).await.unwrap();
        
        let client = reqwest::Client::new();
        let url = format!("http://{}/api/data", addr);
        let response = client.get(&url).send().await.unwrap();
        
        assert_eq!(response.status(), 200);
        let json: serde_json::Value = response.json().await.unwrap();
        assert_eq!(json["events"].as_array().unwrap().len(), 1000);
    }

    #[tokio::test]
    async fn test_server_bind_localhost() {
        let temp_dir = TempDir::new().unwrap();
        let data_file = temp_dir.path().join("data.json");
        fs::write(&data_file, r#"{}"#).unwrap();

        let (addr, _rx) = start_server("127.0.0.1".to_string(), 0, data_file).await.unwrap();
        assert!(addr.ip().is_loopback());
    }

    #[tokio::test]
    async fn test_multiple_shutdown_calls() {
        let temp_dir = TempDir::new().unwrap();
        let data_file = temp_dir.path().join("data.json");
        fs::write(&data_file, r#"{}"#).unwrap();

        let (addr, mut rx) = start_server("127.0.0.1".to_string(), 0, data_file).await.unwrap();
        
        let client = reqwest::Client::new();
        let url = format!("http://{}/api/shutdown", addr);
        
        // Call shutdown multiple times
        let _ = client.get(&url).send().await;
        let _ = client.get(&url).send().await;
        
        // Should still receive signal
        let _ = rx.recv().await;
    }
}
