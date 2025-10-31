# Section 90: Setting Up Tauri Project

## Learning Objectives

By the end of this section, you will:
- Initialize a Tauri project from scratch
- Configure the project structure
- Set up development dependencies
- Configure build system
- Create the basic application shell
- Integrate with existing borrowscope crates

## Prerequisites

- Section 89 (Tauri Architecture Overview)
- Node.js and npm installed
- Rust toolchain installed
- Understanding of package managers

---

## Installation Prerequisites

### System Requirements

**Rust:**
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

**Node.js:**
```bash
# Install Node.js (v16 or later)
# Using nvm (recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
nvm use 18

# Verify installation
node --version
npm --version
```

**System Dependencies:**

**Linux (Ubuntu/Debian):**
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

**macOS:**
```bash
# Install Xcode Command Line Tools
xcode-select --install
```

**Windows:**
```powershell
# Install Microsoft C++ Build Tools
# Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/

# Install WebView2
# Usually pre-installed on Windows 11
```

---

## Project Initialization

### Step 1: Create Tauri Project

```bash
cd borrowscope
npm create tauri-app@latest
```

**Interactive prompts:**
```
✔ Project name · borrowscope-ui
✔ Choose which language to use for your frontend · TypeScript / JavaScript
✔ Choose your package manager · npm
✔ Choose your UI template · Vanilla
✔ Choose your UI flavor · JavaScript
```

**Alternative: Manual setup:**
```bash
mkdir borrowscope-ui
cd borrowscope-ui
npm init -y
npm install --save-dev @tauri-apps/cli
npm install @tauri-apps/api
npx tauri init
```

### Step 2: Project Structure

After initialization:
```
borrowscope-ui/
├── src/                    # Frontend source
│   ├── index.html
│   ├── main.js
│   └── styles.css
│
├── src-tauri/              # Rust backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── icons/
│   │   ├── 32x32.png
│   │   ├── 128x128.png
│   │   ├── icon.icns
│   │   └── icon.ico
│   └── src/
│       └── main.rs
│
├── package.json
├── package-lock.json
└── README.md
```

---

## Configuration Files

### package.json

```json
{
  "name": "borrowscope-ui",
  "version": "0.1.0",
  "description": "BorrowScope Desktop Application",
  "type": "module",
  "scripts": {
    "dev": "tauri dev",
    "build": "tauri build",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^1.5.0",
    "cytoscape": "^3.26.0",
    "cytoscape-dagre": "^2.5.0",
    "d3": "^7.8.5"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^1.5.0",
    "vite": "^5.0.0"
  }
}
```

### src-tauri/Cargo.toml

```toml
[package]
name = "borrowscope-ui"
version = "0.1.0"
description = "BorrowScope Desktop Application"
authors = ["Your Name <you@example.com>"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourusername/borrowscope"
edition = "2021"

[build-dependencies]
tauri-build = { version = "1.5", features = [] }

[dependencies]
tauri = { version = "1.5", features = ["api-all"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
log = "0.4"
env_logger = "0.11"

# BorrowScope dependencies
borrowscope-graph = { path = "../../borrowscope-graph" }
borrowscope-runtime = { path = "../../borrowscope-runtime" }

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]

[profile.release]
panic = "abort"
codegen-units = 1
lto = true
opt-level = "z"
strip = true
```

### src-tauri/tauri.conf.json

```json
{
  "$schema": "../node_modules/@tauri-apps/cli/schema.json",
  "build": {
    "beforeDevCommand": "npm run dev:frontend",
    "beforeBuildCommand": "npm run build:frontend",
    "devPath": "http://localhost:1420",
    "distDir": "../dist",
    "withGlobalTauri": false
  },
  "package": {
    "productName": "BorrowScope",
    "version": "0.1.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "fs": {
        "all": false,
        "readFile": true,
        "writeFile": true,
        "readDir": true,
        "scope": [
          "$HOME/.borrowscope/**",
          "$APP/**",
          "$DOCUMENT/**"
        ]
      },
      "dialog": {
        "all": true,
        "ask": true,
        "confirm": true,
        "message": true,
        "open": true,
        "save": true
      },
      "shell": {
        "all": false,
        "open": true
      },
      "window": {
        "all": false,
        "close": true,
        "hide": true,
        "show": true,
        "maximize": true,
        "minimize": true,
        "unmaximize": true,
        "unminimize": true,
        "startDragging": true
      },
      "path": {
        "all": true
      }
    },
    "bundle": {
      "active": true,
      "targets": "all",
      "identifier": "com.borrowscope.app",
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/128x128@2x.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ],
      "resources": [],
      "externalBin": [],
      "copyright": "Copyright © 2024",
      "category": "DeveloperTool",
      "shortDescription": "Visualize Rust ownership and borrowing",
      "longDescription": "BorrowScope is an interactive visualization tool for understanding Rust's ownership and borrowing system. It tracks variable lifetimes, borrows, and moves to help developers learn and debug ownership issues.",
      "deb": {
        "depends": []
      },
      "macOS": {
        "frameworks": [],
        "minimumSystemVersion": "10.13",
        "exceptionDomain": "",
        "signingIdentity": null,
        "providerShortName": null,
        "entitlements": null
      },
      "windows": {
        "certificateThumbprint": null,
        "digestAlgorithm": "sha256",
        "timestampUrl": ""
      }
    },
    "security": {
      "csp": "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:"
    },
    "windows": [
      {
        "fullscreen": false,
        "resizable": true,
        "title": "BorrowScope",
        "width": 1400,
        "height": 900,
        "minWidth": 800,
        "minHeight": 600,
        "center": true,
        "decorations": true,
        "transparent": false,
        "alwaysOnTop": false,
        "skipTaskbar": false
      }
    ],
    "systemTray": {
      "iconPath": "icons/icon.png",
      "iconAsTemplate": true,
      "menuOnLeftClick": false
    }
  }
}
```

---

## Basic Application Shell

### src-tauri/src/main.rs

```rust
// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::{info, error};
use std::sync::Mutex;
use tauri::State;

// Application state
struct AppState {
    current_file: Mutex<Option<String>>,
}

// Basic commands
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to BorrowScope.", name)
}

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
fn set_current_file(path: String, state: State<AppState>) -> Result<(), String> {
    let mut current = state.current_file.lock()
        .map_err(|e| e.to_string())?;
    *current = Some(path);
    Ok(())
}

#[tauri::command]
fn get_current_file(state: State<AppState>) -> Option<String> {
    state.current_file.lock().ok()
        .and_then(|guard| guard.clone())
}

fn main() {
    // Initialize logger
    env_logger::init();
    
    info!("Starting BorrowScope UI");
    
    tauri::Builder::default()
        .manage(AppState {
            current_file: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_app_version,
            set_current_file,
            get_current_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### src/index.html

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BorrowScope</title>
    <link rel="stylesheet" href="styles.css">
</head>
<body>
    <div id="app">
        <header>
            <h1>🦀 BorrowScope</h1>
            <p>Visualize Rust Ownership and Borrowing</p>
        </header>
        
        <main>
            <div id="welcome">
                <h2>Welcome to BorrowScope</h2>
                <p>Get started by loading a tracking data file.</p>
                <button id="load-btn">Load Graph Data</button>
            </div>
            
            <div id="content" style="display: none;">
                <div id="sidebar">
                    <h3>Controls</h3>
                    <button id="reset-btn">Reset View</button>
                    <button id="export-btn">Export</button>
                </div>
                
                <div id="graph-container">
                    <div id="graph"></div>
                </div>
            </div>
        </main>
        
        <footer>
            <span id="version">v0.1.0</span>
            <span id="status">Ready</span>
        </footer>
    </div>
    
    <script type="module" src="main.js"></script>
</body>
</html>
```

### src/styles.css

```css
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 
                 'Helvetica Neue', Arial, sans-serif;
    background: #1e1e1e;
    color: #d4d4d4;
    overflow: hidden;
}

#app {
    display: flex;
    flex-direction: column;
    height: 100vh;
}

header {
    background: #2d2d30;
    padding: 20px;
    border-bottom: 1px solid #3e3e42;
    text-align: center;
}

header h1 {
    font-size: 28px;
    color: #569cd6;
    margin-bottom: 5px;
}

header p {
    font-size: 14px;
    color: #9cdcfe;
}

main {
    flex: 1;
    display: flex;
    overflow: hidden;
}

#welcome {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px;
}

#welcome h2 {
    font-size: 32px;
    margin-bottom: 20px;
    color: #569cd6;
}

#welcome p {
    font-size: 16px;
    margin-bottom: 30px;
    color: #9cdcfe;
}

button {
    padding: 12px 24px;
    background: #0e639c;
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 14px;
    cursor: pointer;
    transition: background 0.2s;
}

button:hover {
    background: #1177bb;
}

button:active {
    transform: scale(0.98);
}

#content {
    flex: 1;
    display: flex;
}

#sidebar {
    width: 250px;
    background: #252526;
    border-right: 1px solid #3e3e42;
    padding: 20px;
}

#sidebar h3 {
    font-size: 18px;
    margin-bottom: 15px;
    color: #569cd6;
}

#sidebar button {
    width: 100%;
    margin-bottom: 10px;
}

#graph-container {
    flex: 1;
    position: relative;
}

#graph {
    width: 100%;
    height: 100%;
    background: #1e1e1e;
}

footer {
    background: #2d2d30;
    padding: 10px 20px;
    border-top: 1px solid #3e3e42;
    display: flex;
    justify-content: space-between;
    font-size: 12px;
    color: #858585;
}
```

### src/main.js

```javascript
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';

// Initialize app
async function init() {
    console.log('Initializing BorrowScope UI');
    
    // Get version
    const version = await invoke('get_app_version');
    document.getElementById('version').textContent = `v${version}`;
    
    // Set up event listeners
    document.getElementById('load-btn').addEventListener('click', loadFile);
    document.getElementById('reset-btn').addEventListener('click', resetView);
    document.getElementById('export-btn').addEventListener('click', exportGraph);
    
    // Test backend connection
    const greeting = await invoke('greet', { name: 'Developer' });
    console.log(greeting);
}

async function loadFile() {
    try {
        // Open file dialog
        const selected = await open({
            multiple: false,
            filters: [{
                name: 'JSON',
                extensions: ['json']
            }]
        });
        
        if (selected) {
            console.log('Selected file:', selected);
            
            // Save to state
            await invoke('set_current_file', { path: selected });
            
            // Show content area
            document.getElementById('welcome').style.display = 'none';
            document.getElementById('content').style.display = 'flex';
            
            // Update status
            document.getElementById('status').textContent = `Loaded: ${selected}`;
        }
    } catch (error) {
        console.error('Failed to load file:', error);
        alert(`Error: ${error}`);
    }
}

function resetView() {
    console.log('Resetting view');
    // TODO: Implement reset logic
}

function exportGraph() {
    console.log('Exporting graph');
    // TODO: Implement export logic
}

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', init);
```

---

## Development Setup

### Install Dependencies

```bash
cd borrowscope-ui

# Install frontend dependencies
npm install

# Install additional packages
npm install cytoscape cytoscape-dagre d3
```

### Run Development Server

```bash
# Start Tauri in development mode
npm run tauri dev

# This will:
# 1. Start frontend dev server (Vite)
# 2. Compile Rust backend
# 3. Open application window
# 4. Enable hot reload
```

---

## Build for Production

### Development Build

```bash
npm run tauri build -- --debug
```

### Release Build

```bash
npm run tauri build

# Output locations:
# - Windows: src-tauri/target/release/bundle/msi/
# - macOS: src-tauri/target/release/bundle/dmg/
# - Linux: src-tauri/target/release/bundle/deb/ or appimage/
```

---

## Testing the Setup

### Test Backend Commands

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        let result = greet("Test");
        assert!(result.contains("Test"));
    }

    #[test]
    fn test_version() {
        let version = get_app_version();
        assert!(!version.is_empty());
    }
}
```

### Test Frontend

```javascript
// Add to main.js for testing
async function testBackend() {
    try {
        const result = await invoke('greet', { name: 'Test' });
        console.log('Backend test:', result);
        return true;
    } catch (error) {
        console.error('Backend test failed:', error);
        return false;
    }
}

// Run on init
testBackend();
```

---

## Troubleshooting

### Common Issues

**1. Webview not found (Linux):**
```bash
sudo apt install libwebkit2gtk-4.0-dev
```

**2. Build fails on Windows:**
```powershell
# Install Visual Studio Build Tools
# Ensure WebView2 is installed
```

**3. Port already in use:**
```json
// Change port in tauri.conf.json
"devPath": "http://localhost:1421"
```

**4. Hot reload not working:**
```bash
# Clear cache and restart
rm -rf node_modules
npm install
npm run tauri dev
```

---

## Key Takeaways

✅ **Project initialized** - Tauri app structure created  
✅ **Dependencies configured** - Rust and Node.js packages  
✅ **Basic shell** - Working application with UI  
✅ **Development workflow** - Hot reload enabled  
✅ **Build system** - Production builds configured  

---

**Previous:** [89-tauri-architecture-overview.md](./89-tauri-architecture-overview.md)  
**Next:** [91-tauri-commands-and-ipc.md](./91-tauri-commands-and-ipc.md)

**Progress:** 2/17 ⬛⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜
