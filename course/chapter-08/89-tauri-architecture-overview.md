# Section 89: Tauri Architecture Overview

## Learning Objectives

By the end of this section, you will:
- Understand Tauri's architecture and design philosophy
- Learn how Rust backend communicates with web frontend
- Understand the security model and sandboxing
- Compare Tauri with Electron and other frameworks
- Plan the BorrowScope UI architecture
- Understand IPC (Inter-Process Communication) patterns

## Prerequisites

- Completed Chapter 7 (CLI Development)
- Basic understanding of web technologies (HTML/CSS/JavaScript)
- Familiarity with async Rust
- Understanding of desktop application concepts

---

## What is Tauri?

**Tauri** is a framework for building desktop applications using web technologies for the frontend and Rust for the backend.

### Key Features

1. **Small Bundle Size** - Uses system webview instead of bundling Chromium
2. **Security First** - Sandboxed frontend, controlled backend access
3. **Cross-Platform** - Windows, macOS, Linux from single codebase
4. **Performance** - Rust backend for heavy computation
5. **Modern Web** - Use any frontend framework (React, Vue, Svelte, vanilla JS)

### Tauri vs Electron

| Feature | Tauri | Electron |
|---------|-------|----------|
| Bundle Size | ~3-5 MB | ~50-100 MB |
| Memory Usage | ~50-100 MB | ~200-500 MB |
| Backend | Rust | Node.js |
| Webview | System | Bundled Chromium |
| Security | Sandboxed by default | Requires configuration |
| Startup Time | Fast | Slower |

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri Application                     │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌────────────────────┐         ┌──────────────────┐   │
│  │   Web Frontend     │         │   Rust Backend   │   │
│  │                    │         │                  │   │
│  │  HTML/CSS/JS       │◄───────►│  Tauri Core      │   │
│  │  Cytoscape.js      │   IPC   │  Commands        │   │
│  │  D3.js             │         │  State           │   │
│  │  UI Components     │         │  File System     │   │
│  └────────────────────┘         └──────────────────┘   │
│           │                              │              │
│           │                              │              │
│  ┌────────▼──────────┐         ┌────────▼──────────┐   │
│  │   System Webview  │         │  OS Integration   │   │
│  │   (WebKit/Edge)   │         │  (Native APIs)    │   │
│  └───────────────────┘         └───────────────────┘   │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

---

## Core Components

### 1. Rust Backend (src-tauri/)

The backend handles:
- **Business Logic** - Graph processing, analysis
- **File System Access** - Reading/writing tracking data
- **Heavy Computation** - Graph algorithms, statistics
- **System Integration** - Native dialogs, notifications
- **State Management** - Application state

**Example Structure:**
```rust
// src-tauri/src/main.rs
#[tauri::command]
fn load_graph(path: String) -> Result<GraphData, String> {
    // Load and process graph
}

#[tauri::command]
fn analyze_conflicts(graph: GraphData) -> Vec<Conflict> {
    // Analyze borrow conflicts
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            load_graph,
            analyze_conflicts
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 2. Web Frontend (src/)

The frontend handles:
- **User Interface** - Visual components, layouts
- **Graph Visualization** - Cytoscape.js rendering
- **Timeline View** - D3.js timeline
- **User Interactions** - Click, drag, zoom
- **State Display** - Show current state

**Example Structure:**
```javascript
// src/main.js
import { invoke } from '@tauri-apps/api/tauri';

async function loadGraph(path) {
    try {
        const data = await invoke('load_graph', { path });
        renderGraph(data);
    } catch (error) {
        console.error('Failed to load graph:', error);
    }
}
```

### 3. IPC Bridge

Communication between frontend and backend:

```
Frontend                    Backend
   │                           │
   │  invoke('command', args)  │
   ├──────────────────────────►│
   │                           │
   │                      Process
   │                           │
   │      Return result        │
   │◄──────────────────────────┤
   │                           │
```

---

## Security Model

### Sandboxing

Tauri uses a **capability-based security model**:

1. **Frontend is Sandboxed** - Cannot access file system directly
2. **Explicit Permissions** - Must declare what frontend can access
3. **Command Whitelist** - Only registered commands are callable
4. **CSP (Content Security Policy)** - Prevents XSS attacks

### Configuration

**tauri.conf.json:**
```json
{
  "tauri": {
    "allowlist": {
      "all": false,
      "fs": {
        "readFile": true,
        "writeFile": true,
        "scope": ["$APP/*"]
      },
      "dialog": {
        "open": true,
        "save": true
      }
    }
  }
}
```

### Security Best Practices

1. **Validate Input** - Always validate data from frontend
2. **Sanitize Paths** - Prevent directory traversal
3. **Use Scopes** - Limit file system access
4. **Error Handling** - Don't leak sensitive info in errors
5. **CSP Headers** - Prevent inline scripts

---

## BorrowScope Architecture

### Application Structure

```
borrowscope-ui/
├── src/                    # Frontend (Web)
│   ├── index.html
│   ├── main.js
│   ├── styles.css
│   ├── components/
│   │   ├── GraphView.js
│   │   ├── Timeline.js
│   │   ├── Sidebar.js
│   │   └── CodeView.js
│   └── utils/
│       ├── graph.js
│       └── api.js
│
├── src-tauri/              # Backend (Rust)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── icons/
│   └── src/
│       ├── main.rs
│       ├── commands/
│       │   ├── mod.rs
│       │   ├── graph.rs
│       │   ├── file.rs
│       │   └── analysis.rs
│       ├── state.rs
│       └── error.rs
│
└── package.json            # Frontend dependencies
```

### Data Flow

```
User Action (Click node)
    ↓
JavaScript Event Handler
    ↓
Invoke Tauri Command
    ↓
Rust Backend Processing
    ↓
Return Result
    ↓
Update UI
```

### State Management

**Backend State:**
```rust
use std::sync::Mutex;
use tauri::State;

struct AppState {
    current_graph: Mutex<Option<OwnershipGraph>>,
    recent_files: Mutex<Vec<String>>,
}

#[tauri::command]
fn get_current_graph(state: State<AppState>) -> Option<GraphExport> {
    state.current_graph.lock().unwrap()
        .as_ref()
        .map(|g| g.export())
}
```

**Frontend State:**
```javascript
class AppState {
    constructor() {
        this.graph = null;
        this.selectedNode = null;
        this.timelinePosition = 0;
    }
    
    async loadGraph(path) {
        this.graph = await invoke('load_graph', { path });
        this.render();
    }
}
```

---

## IPC Communication Patterns

### 1. Request-Response

**Frontend:**
```javascript
const result = await invoke('command_name', { arg1, arg2 });
```

**Backend:**
```rust
#[tauri::command]
fn command_name(arg1: String, arg2: i32) -> Result<String, String> {
    Ok("result".to_string())
}
```

### 2. Events (Backend → Frontend)

**Backend:**
```rust
use tauri::Manager;

fn emit_progress(app: &tauri::AppHandle, progress: f64) {
    app.emit_all("progress", progress).unwrap();
}
```

**Frontend:**
```javascript
import { listen } from '@tauri-apps/api/event';

await listen('progress', (event) => {
    console.log('Progress:', event.payload);
});
```

### 3. Streaming Data

**Backend:**
```rust
#[tauri::command]
async fn stream_events(app: tauri::AppHandle) {
    for i in 0..100 {
        app.emit_all("event", i).unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
```

**Frontend:**
```javascript
const unlisten = await listen('event', (event) => {
    console.log('Event:', event.payload);
});

// Later: unlisten();
```

---

## Performance Considerations

### 1. Minimize IPC Calls

**Bad:**
```javascript
// Multiple calls
for (let node of nodes) {
    await invoke('process_node', { node });
}
```

**Good:**
```javascript
// Single batch call
await invoke('process_nodes', { nodes });
```

### 2. Use Async Commands

```rust
#[tauri::command]
async fn heavy_computation() -> Result<String, String> {
    tokio::task::spawn_blocking(|| {
        // CPU-intensive work
    }).await.map_err(|e| e.to_string())
}
```

### 3. Cache Results

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

struct Cache {
    data: Arc<RwLock<HashMap<String, GraphData>>>,
}

#[tauri::command]
async fn get_cached(key: String, cache: State<'_, Cache>) -> Option<GraphData> {
    cache.data.read().await.get(&key).cloned()
}
```

---

## Development Workflow

### 1. Development Mode

```bash
# Terminal 1: Run Tauri dev server
cd borrowscope-ui
npm run tauri dev

# Hot reload enabled for frontend
# Rust backend recompiles on changes
```

### 2. Building for Production

```bash
# Build optimized bundle
npm run tauri build

# Output:
# - Windows: .exe, .msi
# - macOS: .app, .dmg
# - Linux: .deb, .AppImage
```

### 3. Debugging

**Frontend:**
- Open DevTools (F12)
- Console logs
- Network tab (for assets)

**Backend:**
```rust
use log::{info, error};

#[tauri::command]
fn my_command() {
    info!("Command called");
    error!("Something went wrong");
}
```

---

## Tauri Configuration

**tauri.conf.json:**
```json
{
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devPath": "http://localhost:1420",
    "distDir": "../dist"
  },
  "package": {
    "productName": "BorrowScope",
    "version": "0.1.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "fs": {
        "readFile": true,
        "writeFile": true,
        "scope": ["$HOME/.borrowscope/*", "$APP/*"]
      },
      "dialog": {
        "all": true
      },
      "shell": {
        "open": true
      }
    },
    "bundle": {
      "active": true,
      "identifier": "com.borrowscope.app",
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ],
      "resources": [],
      "externalBin": [],
      "copyright": "",
      "category": "DeveloperTool",
      "shortDescription": "Visualize Rust ownership",
      "longDescription": "Interactive visualization of Rust ownership and borrowing"
    },
    "security": {
      "csp": "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"
    },
    "windows": [
      {
        "title": "BorrowScope",
        "width": 1200,
        "height": 800,
        "resizable": true,
        "fullscreen": false
      }
    ]
  }
}
```

---

## Integration with Existing Code

### Using borrowscope-graph

**Cargo.toml:**
```toml
[dependencies]
borrowscope-graph = { path = "../../borrowscope-graph" }
borrowscope-runtime = { path = "../../borrowscope-runtime" }
tauri = { version = "1.5", features = ["api-all"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**Commands:**
```rust
use borrowscope_graph::{OwnershipGraph, GraphExport};

#[tauri::command]
fn load_graph_file(path: String) -> Result<GraphExport, String> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| e.to_string())?;
    
    let export: GraphExport = serde_json::from_str(&content)
        .map_err(|e| e.to_string())?;
    
    Ok(export)
}

#[tauri::command]
fn analyze_graph(data: GraphExport) -> Result<AnalysisResult, String> {
    let mut graph = OwnershipGraph::new();
    
    // Reconstruct graph
    for node in data.nodes {
        graph.add_variable(node);
    }
    
    // Analyze
    let conflicts = graph.find_conflicts_optimized();
    let stats = graph.statistics();
    
    Ok(AnalysisResult { conflicts, stats })
}
```

---

## Error Handling

### Backend Errors

```rust
use serde::Serialize;

#[derive(Debug, Serialize)]
struct AppError {
    message: String,
    kind: String,
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError {
            message: err.to_string(),
            kind: "IoError".to_string(),
        }
    }
}

#[tauri::command]
fn risky_operation() -> Result<String, AppError> {
    std::fs::read_to_string("file.txt")?;
    Ok("success".to_string())
}
```

### Frontend Error Handling

```javascript
async function safeInvoke(command, args) {
    try {
        return await invoke(command, args);
    } catch (error) {
        console.error(`Command ${command} failed:`, error);
        showErrorNotification(error);
        throw error;
    }
}
```

---

## Testing Strategy

### Backend Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_graph() {
        let result = load_graph_file("test.json".to_string());
        assert!(result.is_ok());
    }
}
```

### Frontend Tests

```javascript
// Using Jest or Vitest
import { invoke } from '@tauri-apps/api/tauri';

jest.mock('@tauri-apps/api/tauri');

test('loads graph successfully', async () => {
    invoke.mockResolvedValue({ nodes: [], edges: [] });
    
    const result = await loadGraph('test.json');
    expect(result).toBeDefined();
});
```

---

## Key Takeaways

✅ **Tauri Architecture** - Rust backend + web frontend  
✅ **IPC Communication** - Commands and events  
✅ **Security Model** - Sandboxed, capability-based  
✅ **Performance** - Small bundles, fast startup  
✅ **Integration** - Use existing borrowscope crates  
✅ **Development** - Hot reload, easy debugging  

---

## Further Reading

- [Tauri Documentation](https://tauri.app/v1/guides/)
- [Tauri API Reference](https://tauri.app/v1/api/js/)
- [Tauri Security](https://tauri.app/v1/guides/security/)
- [Tauri Examples](https://github.com/tauri-apps/tauri/tree/dev/examples)

---

**Previous:** [Chapter 7 Summary](../chapter-07/CHAPTER_SUMMARY.md)  
**Next:** [90-setting-up-tauri-project.md](./90-setting-up-tauri-project.md)

**Progress:** 1/17 ⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜
