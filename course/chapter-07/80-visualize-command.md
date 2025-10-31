# Section 80: Visualize Command Implementation

## Learning Objectives

By the end of this section, you will:
- Implement a web server for visualization
- Serve static HTML/CSS/JavaScript files
- Handle HTTP requests with Tokio
- Integrate with browser auto-open
- Implement hot-reload for development
- Handle WebSocket connections for live updates

## Prerequisites

- Section 79 (Run Command Implementation)
- Understanding of HTTP protocol
- Familiarity with async Rust and Tokio
- Basic knowledge of web servers

---

## Command Purpose

The `visualize` command starts a local web server that:
1. Loads tracking data from JSON file
2. Serves an interactive web UI
3. Opens the browser automatically
4. Provides real-time updates via WebSocket
5. Handles multiple concurrent connections

---

## Architecture

```
┌─────────────┐
│   Browser   │
└──────┬──────┘
       │ HTTP/WebSocket
┌──────▼──────────────┐
│   Web Server        │
│   (Tokio/Hyper)     │
└──────┬──────────────┘
       │
┌──────▼──────────────┐
│  Tracking Data      │
│  (JSON)             │
└─────────────────────┘
```

---

## Complete Implementation

**src/commands/visualize.rs:**

```rust
use crate::{cli::VisualizeArgs, error::CliError, output::Output};
use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn execute(args: VisualizeArgs) -> Result<()> {
    let output = Output::new(false, false);
    
    // Validate file exists
    if !args.file.exists() {
        return Err(CliError::FileNotFound(args.file.display().to_string()).into());
    }
    
    // Load tracking data
    output.info("Loading tracking data...");
    let data = load_tracking_data(&args.file)?;
    
    output.success(&format!(
        "Loaded {} nodes, {} edges", 
        data.nodes.len(), 
        data.edges.len()
    ));
    
    // Validate data
    validate_tracking_data(&data)?;
    
    // Start web server
    output.info(&format!("Starting server at http://{}:{}", args.host, args.port));
    
    let addr = format!("{}:{}", args.host, args.port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            return Err(CliError::PortInUse(args.port).into());
        }
        Err(e) => return Err(e.into()),
    };
    
    // Open browser
    if !args.no_browser {
        output.info("Opening browser...");
        if let Err(e) = open_browser(&args) {
            output.warning(&format!("Failed to open browser: {}", e));
            output.info(&format!("Please open http://{}:{} manually", args.host, args.port));
        }
    }
    
    output.success("Visualization ready!");
    output.info("Press Ctrl+C to stop the server");
    
    // Serve requests
    serve(listener, Arc::new(data)).await?;
    
    Ok(())
}

fn load_tracking_data(path: &Path) -> Result<borrowscope_graph::GraphExport> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read tracking data file")?;
    
    let data: borrowscope_graph::GraphExport = serde_json::from_str(&content)
        .context("Failed to parse tracking data (invalid JSON)")?;
    
    Ok(data)
}

fn validate_tracking_data(data: &borrowscope_graph::GraphExport) -> Result<()> {
    if data.nodes.is_empty() {
        anyhow::bail!("Tracking data contains no nodes");
    }
    
    // Validate node IDs are unique
    let mut seen_ids = std::collections::HashSet::new();
    for node in &data.nodes {
        if !seen_ids.insert(node.id) {
            anyhow::bail!("Duplicate node ID: {}", node.id);
        }
    }
    
    // Validate edges reference valid nodes
    for edge in &data.edges {
        if !seen_ids.contains(&edge.from_id) {
            anyhow::bail!("Edge references non-existent node: {}", edge.from_id);
        }
        if !seen_ids.contains(&edge.to_id) {
            anyhow::bail!("Edge references non-existent node: {}", edge.to_id);
        }
    }
    
    Ok(())
}

async fn serve(
    listener: TcpListener,
    data: Arc<borrowscope_graph::GraphExport>
) -> Result<()> {
    loop {
        let (mut stream, addr) = listener.accept().await?;
        let data = Arc::clone(&data);
        
        tokio::spawn(async move {
            if let Err(e) = handle_connection(&mut stream, data).await {
                eprintln!("Error handling connection from {}: {}", addr, e);
            }
        });
    }
}

async fn handle_connection(
    stream: &mut tokio::net::TcpStream,
    data: Arc<borrowscope_graph::GraphExport>
) -> Result<()> {
    let mut buffer = [0; 4096];
    let n = stream.read(&mut buffer).await?;
    
    if n == 0 {
        return Ok(());
    }
    
    let request = String::from_utf8_lossy(&buffer[..n]);
    let request_line = request.lines().next().unwrap_or("");
    
    // Parse HTTP request
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return send_response(stream, 400, "Bad Request", "text/plain", b"Bad Request").await;
    }
    
    let method = parts[0];
    let path = parts[1];
    
    match (method, path) {
        ("GET", "/") => {
            serve_index(stream).await?;
        }
        ("GET", "/api/data") => {
            serve_data(stream, &data).await?;
        }
        ("GET", "/api/stats") => {
            serve_stats(stream, &data).await?;
        }
        ("GET", path) if path.starts_with("/static/") => {
            serve_static(stream, path).await?;
        }
        _ => {
            send_response(stream, 404, "Not Found", "text/plain", b"Not Found").await?;
        }
    }
    
    Ok(())
}

async fn serve_index(stream: &mut tokio::net::TcpStream) -> Result<()> {
    let html = generate_html();
    send_response(stream, 200, "OK", "text/html", html.as_bytes()).await
}

async fn serve_data(
    stream: &mut tokio::net::TcpStream,
    data: &borrowscope_graph::GraphExport
) -> Result<()> {
    let json = serde_json::to_string(data)?;
    send_response(stream, 200, "OK", "application/json", json.as_bytes()).await
}

async fn serve_stats(
    stream: &mut tokio::net::TcpStream,
    data: &borrowscope_graph::GraphExport
) -> Result<()> {
    let stats = compute_stats(data);
    let json = serde_json::to_string(&stats)?;
    send_response(stream, 200, "OK", "application/json", json.as_bytes()).await
}

async fn serve_static(
    stream: &mut tokio::net::TcpStream,
    path: &str
) -> Result<()> {
    // In production, serve actual static files
    // For now, return 404
    send_response(stream, 404, "Not Found", "text/plain", b"Not Found").await
}

async fn send_response(
    stream: &mut tokio::net::TcpStream,
    status_code: u16,
    status_text: &str,
    content_type: &str,
    body: &[u8]
) -> Result<()> {
    let response = format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: {}\r\n\
         Content-Length: {}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Connection: close\r\n\
         \r\n",
        status_code,
        status_text,
        content_type,
        body.len()
    );
    
    stream.write_all(response.as_bytes()).await?;
    stream.write_all(body).await?;
    stream.flush().await?;
    
    Ok(())
}

fn generate_html() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BorrowScope Visualization</title>
    <script src="https://unpkg.com/cytoscape@3.26.0/dist/cytoscape.min.js"></script>
    <script src="https://unpkg.com/cytoscape-dagre@2.5.0/cytoscape-dagre.js"></script>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #1e1e1e;
            color: #d4d4d4;
            overflow: hidden;
        }
        
        #container {
            display: flex;
            height: 100vh;
        }
        
        #sidebar {
            width: 300px;
            background: #252526;
            border-right: 1px solid #3e3e42;
            padding: 20px;
            overflow-y: auto;
        }
        
        #cy {
            flex: 1;
            background: #1e1e1e;
        }
        
        h1 {
            font-size: 24px;
            margin-bottom: 20px;
            color: #569cd6;
        }
        
        .stats {
            margin-bottom: 30px;
        }
        
        .stat-item {
            display: flex;
            justify-content: space-between;
            padding: 8px 0;
            border-bottom: 1px solid #3e3e42;
        }
        
        .stat-label {
            color: #9cdcfe;
        }
        
        .stat-value {
            color: #ce9178;
            font-weight: bold;
        }
        
        .controls {
            margin-top: 20px;
        }
        
        button {
            width: 100%;
            padding: 10px;
            margin-bottom: 10px;
            background: #0e639c;
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
        }
        
        button:hover {
            background: #1177bb;
        }
        
        .legend {
            margin-top: 30px;
        }
        
        .legend-item {
            display: flex;
            align-items: center;
            margin-bottom: 10px;
        }
        
        .legend-color {
            width: 20px;
            height: 20px;
            border-radius: 4px;
            margin-right: 10px;
        }
        
        .loading {
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            font-size: 18px;
            color: #569cd6;
        }
    </style>
</head>
<body>
    <div id="container">
        <div id="sidebar">
            <h1>BorrowScope</h1>
            
            <div class="stats">
                <h2 style="font-size: 18px; margin-bottom: 10px;">Statistics</h2>
                <div class="stat-item">
                    <span class="stat-label">Total Variables:</span>
                    <span class="stat-value" id="stat-nodes">0</span>
                </div>
                <div class="stat-item">
                    <span class="stat-label">Total Borrows:</span>
                    <span class="stat-value" id="stat-edges">0</span>
                </div>
                <div class="stat-item">
                    <span class="stat-label">Alive Variables:</span>
                    <span class="stat-value" id="stat-alive">0</span>
                </div>
                <div class="stat-item">
                    <span class="stat-label">Dropped Variables:</span>
                    <span class="stat-value" id="stat-dropped">0</span>
                </div>
            </div>
            
            <div class="controls">
                <h2 style="font-size: 18px; margin-bottom: 10px;">Controls</h2>
                <button onclick="resetZoom()">Reset Zoom</button>
                <button onclick="fitGraph()">Fit to Screen</button>
                <button onclick="exportPNG()">Export as PNG</button>
            </div>
            
            <div class="legend">
                <h2 style="font-size: 18px; margin-bottom: 10px;">Legend</h2>
                <div class="legend-item">
                    <div class="legend-color" style="background: #3498db;"></div>
                    <span>Owned Variable</span>
                </div>
                <div class="legend-item">
                    <div class="legend-color" style="background: #2ecc71;"></div>
                    <span>Immutable Borrow</span>
                </div>
                <div class="legend-item">
                    <div class="legend-color" style="background: #e74c3c;"></div>
                    <span>Mutable Borrow</span>
                </div>
                <div class="legend-item">
                    <div class="legend-color" style="background: #95a5a6;"></div>
                    <span>Dropped</span>
                </div>
            </div>
        </div>
        
        <div id="cy"></div>
        <div class="loading" id="loading">Loading graph data...</div>
    </div>
    
    <script>
        let cy;
        
        async function loadData() {
            try {
                const response = await fetch('/api/data');
                const data = await response.json();
                
                document.getElementById('loading').style.display = 'none';
                
                initializeGraph(data);
                updateStats(data);
            } catch (error) {
                console.error('Failed to load data:', error);
                document.getElementById('loading').textContent = 'Error loading data';
            }
        }
        
        function initializeGraph(data) {
            const elements = {
                nodes: data.nodes.map(node => ({
                    data: {
                        id: String(node.id),
                        label: node.name,
                        type: node.type_name,
                        alive: node.dropped_at === null
                    }
                })),
                edges: data.edges.map((edge, i) => ({
                    data: {
                        id: `e${i}`,
                        source: String(edge.from_id),
                        target: String(edge.to_id),
                        relationship: edge.relationship
                    }
                }))
            };
            
            cy = cytoscape({
                container: document.getElementById('cy'),
                elements: elements,
                style: [
                    {
                        selector: 'node',
                        style: {
                            'label': 'data(label)',
                            'text-valign': 'center',
                            'text-halign': 'center',
                            'background-color': '#3498db',
                            'color': '#fff',
                            'font-size': '12px',
                            'width': '60px',
                            'height': '60px',
                            'border-width': '2px',
                            'border-color': '#2980b9'
                        }
                    },
                    {
                        selector: 'node[!alive]',
                        style: {
                            'background-color': '#95a5a6',
                            'border-color': '#7f8c8d',
                            'opacity': 0.6
                        }
                    },
                    {
                        selector: 'edge',
                        style: {
                            'width': 2,
                            'line-color': '#95a5a6',
                            'target-arrow-color': '#95a5a6',
                            'target-arrow-shape': 'triangle',
                            'curve-style': 'bezier'
                        }
                    }
                ],
                layout: {
                    name: 'dagre',
                    rankDir: 'LR',
                    nodeSep: 50,
                    rankSep: 100
                }
            });
            
            // Add click handler
            cy.on('tap', 'node', function(evt) {
                const node = evt.target;
                console.log('Clicked node:', node.data());
            });
        }
        
        function updateStats(data) {
            document.getElementById('stat-nodes').textContent = data.nodes.length;
            document.getElementById('stat-edges').textContent = data.edges.length;
            
            const alive = data.nodes.filter(n => n.dropped_at === null).length;
            document.getElementById('stat-alive').textContent = alive;
            document.getElementById('stat-dropped').textContent = data.nodes.length - alive;
        }
        
        function resetZoom() {
            if (cy) cy.zoom(1);
        }
        
        function fitGraph() {
            if (cy) cy.fit();
        }
        
        function exportPNG() {
            if (cy) {
                const png = cy.png({ full: true });
                const link = document.createElement('a');
                link.download = 'borrowscope-graph.png';
                link.href = png;
                link.click();
            }
        }
        
        // Load data on page load
        loadData();
    </script>
</body>
</html>"#.to_string()
}

fn compute_stats(data: &borrowscope_graph::GraphExport) -> serde_json::Value {
    let alive = data.nodes.iter().filter(|n| n.dropped_at.is_none()).count();
    let dropped = data.nodes.len() - alive;
    
    serde_json::json!({
        "total_nodes": data.nodes.len(),
        "total_edges": data.edges.len(),
        "alive_nodes": alive,
        "dropped_nodes": dropped,
    })
}

fn open_browser(args: &VisualizeArgs) -> Result<()> {
    let url = format!("http://{}:{}", args.host, args.port);
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .context("Failed to open browser")?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .context("Failed to open browser")?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(&["/C", "start", &url])
            .spawn()
            .context("Failed to open browser")?;
    }
    
    Ok(())
}
```

---

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_load_tracking_data() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("data.json");
        
        let data = r#"{
            "nodes": [
                {"id": 1, "name": "x", "type_name": "i32", "created_at": 1000, "dropped_at": null, "scope_depth": 0}
            ],
            "edges": []
        }"#;
        
        fs::write(&file, data).unwrap();
        
        let result = load_tracking_data(&file);
        assert!(result.is_ok());
        
        let loaded = result.unwrap();
        assert_eq!(loaded.nodes.len(), 1);
        assert_eq!(loaded.nodes[0].name, "x");
    }

    #[test]
    fn test_validate_tracking_data() {
        let data = borrowscope_graph::GraphExport {
            nodes: vec![
                borrowscope_graph::NodeExport {
                    id: 1,
                    name: "x".into(),
                    type_name: "i32".into(),
                    created_at: 1000,
                    dropped_at: None,
                    scope_depth: 0,
                }
            ],
            edges: vec![],
        };
        
        assert!(validate_tracking_data(&data).is_ok());
    }

    #[test]
    fn test_validate_empty_data() {
        let data = borrowscope_graph::GraphExport {
            nodes: vec![],
            edges: vec![],
        };
        
        assert!(validate_tracking_data(&data).is_err());
    }

    #[test]
    fn test_validate_duplicate_ids() {
        let data = borrowscope_graph::GraphExport {
            nodes: vec![
                borrowscope_graph::NodeExport {
                    id: 1,
                    name: "x".into(),
                    type_name: "i32".into(),
                    created_at: 1000,
                    dropped_at: None,
                    scope_depth: 0,
                },
                borrowscope_graph::NodeExport {
                    id: 1,  // Duplicate!
                    name: "y".into(),
                    type_name: "i32".into(),
                    created_at: 1100,
                    dropped_at: None,
                    scope_depth: 0,
                }
            ],
            edges: vec![],
        };
        
        assert!(validate_tracking_data(&data).is_err());
    }

    #[test]
    fn test_validate_invalid_edge() {
        let data = borrowscope_graph::GraphExport {
            nodes: vec![
                borrowscope_graph::NodeExport {
                    id: 1,
                    name: "x".into(),
                    type_name: "i32".into(),
                    created_at: 1000,
                    dropped_at: None,
                    scope_depth: 0,
                }
            ],
            edges: vec![
                borrowscope_graph::EdgeExport {
                    from_id: 1,
                    to_id: 999,  // Non-existent node
                    relationship: borrowscope_graph::Relationship::BorrowsImmut { at: 1050 },
                }
            ],
        };
        
        assert!(validate_tracking_data(&data).is_err());
    }

    #[test]
    fn test_compute_stats() {
        let data = borrowscope_graph::GraphExport {
            nodes: vec![
                borrowscope_graph::NodeExport {
                    id: 1,
                    name: "x".into(),
                    type_name: "i32".into(),
                    created_at: 1000,
                    dropped_at: None,
                    scope_depth: 0,
                },
                borrowscope_graph::NodeExport {
                    id: 2,
                    name: "y".into(),
                    type_name: "i32".into(),
                    created_at: 1100,
                    dropped_at: Some(1200),
                    scope_depth: 0,
                }
            ],
            edges: vec![],
        };
        
        let stats = compute_stats(&data);
        assert_eq!(stats["total_nodes"], 2);
        assert_eq!(stats["alive_nodes"], 1);
        assert_eq!(stats["dropped_nodes"], 1);
    }
}
```

---

## Usage Examples

```bash
# Basic usage
borrowscope visualize borrowscope.json

# Custom port
borrowscope visualize data.json --port 8080

# Don't open browser
borrowscope visualize data.json --no-browser

# Custom host (for remote access)
borrowscope visualize data.json --host 0.0.0.0 --port 3000

# Then access from another machine
# http://your-ip:3000
```

---

## Advanced Features

### WebSocket Support

For live updates, add WebSocket support:

```rust
use tokio_tungstenite::{accept_async, tungstenite::Message};

async fn handle_websocket(
    stream: tokio::net::TcpStream,
    data: Arc<borrowscope_graph::GraphExport>
) -> Result<()> {
    let ws_stream = accept_async(stream).await?;
    let (mut write, mut read) = ws_stream.split();
    
    // Send initial data
    let json = serde_json::to_string(&*data)?;
    write.send(Message::Text(json)).await?;
    
    // Handle incoming messages
    while let Some(msg) = read.next().await {
        match msg? {
            Message::Text(text) => {
                // Handle commands from client
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
    
    Ok(())
}
```

### Hot Reload

Watch for file changes and reload:

```rust
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

pub async fn execute_with_watch(args: VisualizeArgs) -> Result<()> {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(1))?;
    
    watcher.watch(&args.file, RecursiveMode::NonRecursive)?;
    
    // Start server in background
    let server_handle = tokio::spawn(async move {
        // ... server code
    });
    
    // Watch for file changes
    loop {
        match rx.recv() {
            Ok(event) => {
                println!("File changed, reloading...");
                // Reload data and notify clients
            }
            Err(e) => {
                eprintln!("Watch error: {}", e);
                break;
            }
        }
    }
    
    server_handle.await?
}
```

---

## Key Takeaways

✅ **Web server** - Tokio-based HTTP server  
✅ **Static HTML** - Embedded visualization UI  
✅ **Cytoscape.js** - Interactive graph rendering  
✅ **Browser integration** - Auto-open on all platforms  
✅ **Data validation** - Ensure tracking data integrity  
✅ **Statistics** - Real-time graph metrics  
✅ **Error handling** - Port conflicts, invalid data  

---

**Previous:** [79-run-command-implementation.md](./79-run-command-implementation.md)  
**Next:** [81-export-command.md](./81-export-command.md)

**Progress:** 5/13 ⬛⬛⬛⬛⬛⬜⬜⬜⬜⬜⬜⬜⬜
