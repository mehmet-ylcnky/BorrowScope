# Section 92: Loading and Parsing JSON

## Learning Objectives

By the end of this section, you will:
- Implement file loading with Tauri dialog API
- Parse and validate JSON tracking data
- Handle file system errors gracefully
- Implement drag-and-drop file loading
- Cache loaded data for performance
- Validate data integrity
- Handle large files efficiently
- Implement file watching for auto-reload

## Prerequisites

- Section 91 (Tauri Commands and IPC)
- Understanding of JSON format
- Familiarity with file system operations
- Knowledge of async/await patterns

---

## File Loading Architecture

```
User Action (Open File)
    ↓
Tauri Dialog API
    ↓
File System Read
    ↓
JSON Parsing
    ↓
Data Validation
    ↓
Cache Storage
    ↓
Return to Frontend
```

---

## Backend Implementation

### File Loading Command

**src-tauri/src/commands/file.rs:**

```rust
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use anyhow::{Context, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadResult {
    pub success: bool,
    pub data: Option<GraphData>,
    pub error: Option<String>,
    pub file_info: FileInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub size: u64,
    pub modified: u64,
    pub checksum: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<NodeData>,
    pub edges: Vec<EdgeData>,
    pub metadata: GraphMetadata,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeData {
    pub id: usize,
    pub name: String,
    pub type_name: String,
    pub created_at: u64,
    pub dropped_at: Option<u64>,
    pub scope_depth: usize,
    pub is_alive: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EdgeData {
    pub from_id: usize,
    pub to_id: usize,
    pub relationship: String,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub node_count: usize,
    pub edge_count: usize,
    pub file_path: String,
    pub version: String,
}

#[tauri::command]
pub async fn load_json_file(path: String) -> Result<LoadResult, String> {
    let file_path = Path::new(&path);
    
    // Validate file exists
    if !file_path.exists() {
        return Ok(LoadResult {
            success: false,
            data: None,
            error: Some(format!("File not found: {}", path)),
            file_info: FileInfo {
                path: path.clone(),
                size: 0,
                modified: 0,
                checksum: String::new(),
            },
        });
    }
    
    // Get file metadata
    let metadata = fs::metadata(&path).await
        .map_err(|e| format!("Failed to read metadata: {}", e))?;
    
    let modified = metadata.modified()
        .map_err(|e| format!("Failed to get modified time: {}", e))?
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Read file content
    let content = fs::read_to_string(&path).await
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    // Calculate checksum
    let checksum = calculate_checksum(&content);
    
    // Parse JSON
    match parse_and_validate(&content, &path) {
        Ok(graph_data) => {
            Ok(LoadResult {
                success: true,
                data: Some(graph_data),
                error: None,
                file_info: FileInfo {
                    path,
                    size: metadata.len(),
                    modified,
                    checksum,
                },
            })
        }
        Err(e) => {
            Ok(LoadResult {
                success: false,
                data: None,
                error: Some(e.to_string()),
                file_info: FileInfo {
                    path,
                    size: metadata.len(),
                    modified,
                    checksum,
                },
            })
        }
    }
}

fn calculate_checksum(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn parse_and_validate(content: &str, path: &str) -> Result<GraphData> {
    // Parse JSON
    let export: borrowscope_graph::GraphExport = serde_json::from_str(content)
        .context("Invalid JSON format")?;
    
    // Validate data
    validate_graph_data(&export)?;
    
    // Convert to frontend format
    let graph_data = convert_to_graph_data(export, path);
    
    Ok(graph_data)
}

fn validate_graph_data(export: &borrowscope_graph::GraphExport) -> Result<()> {
    // Check for empty data
    if export.nodes.is_empty() {
        anyhow::bail!("Graph contains no nodes");
    }
    
    // Validate node IDs are unique
    let mut seen_ids = std::collections::HashSet::new();
    for node in &export.nodes {
        if !seen_ids.insert(node.id) {
            anyhow::bail!("Duplicate node ID: {}", node.id);
        }
    }
    
    // Validate edges reference valid nodes
    for edge in &export.edges {
        if !seen_ids.contains(&edge.from_id) {
            anyhow::bail!("Edge references non-existent node: {}", edge.from_id);
        }
        if !seen_ids.contains(&edge.to_id) {
            anyhow::bail!("Edge references non-existent node: {}", edge.to_id);
        }
    }
    
    // Validate timestamps
    for node in &export.nodes {
        if let Some(dropped) = node.dropped_at {
            if dropped < node.created_at {
                anyhow::bail!("Invalid timestamps for node {}: dropped before created", node.id);
            }
        }
    }
    
    Ok(())
}

fn convert_to_graph_data(
    export: borrowscope_graph::GraphExport,
    path: &str
) -> GraphData {
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64;
    
    let nodes = export.nodes.iter().map(|node| NodeData {
        id: node.id,
        name: node.name.clone(),
        type_name: node.type_name.clone(),
        created_at: node.created_at,
        dropped_at: node.dropped_at,
        scope_depth: node.scope_depth,
        is_alive: node.dropped_at.map_or(true, |d| d > current_time),
    }).collect();
    
    let edges = export.edges.iter().map(|edge| EdgeData {
        from_id: edge.from_id,
        to_id: edge.to_id,
        relationship: format!("{:?}", edge.relationship),
        timestamp: extract_timestamp(&edge.relationship),
    }).collect();
    
    GraphData {
        nodes,
        edges,
        metadata: GraphMetadata {
            node_count: export.nodes.len(),
            edge_count: export.edges.len(),
            file_path: path.to_string(),
            version: "1.0".to_string(),
        },
    }
}

fn extract_timestamp(rel: &borrowscope_graph::Relationship) -> u64 {
    match rel {
        borrowscope_graph::Relationship::BorrowsImmut { at } => *at,
        borrowscope_graph::Relationship::BorrowsMut { at } => *at,
        borrowscope_graph::Relationship::Moves { at } => *at,
        borrowscope_graph::Relationship::RcClone { at, .. } => *at,
        borrowscope_graph::Relationship::ArcClone { at, .. } => *at,
        borrowscope_graph::Relationship::RefCellBorrow { at, .. } => *at,
    }
}
```

### Validation Command

```rust
#[tauri::command]
pub async fn validate_json_file(path: String) -> Result<ValidationResult, String> {
    let content = fs::read_to_string(&path).await
        .map_err(|e| e.to_string())?;
    
    match serde_json::from_str::<borrowscope_graph::GraphExport>(&content) {
        Ok(export) => {
            match validate_graph_data(&export) {
                Ok(_) => Ok(ValidationResult {
                    valid: true,
                    errors: vec![],
                    warnings: vec![],
                }),
                Err(e) => Ok(ValidationResult {
                    valid: false,
                    errors: vec![e.to_string()],
                    warnings: vec![],
                }),
            }
        }
        Err(e) => Ok(ValidationResult {
            valid: false,
            errors: vec![format!("JSON parse error: {}", e)],
            warnings: vec![],
        }),
    }
}

#[derive(Debug, Serialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
```

### File Watching

```rust
use notify::{Watcher, RecursiveMode, Event};
use std::sync::mpsc::channel;

#[tauri::command]
pub async fn watch_file(
    path: String,
    app: tauri::AppHandle
) -> Result<(), String> {
    let (tx, rx) = channel();
    
    let mut watcher = notify::recommended_watcher(tx)
        .map_err(|e| e.to_string())?;
    
    watcher.watch(Path::new(&path), RecursiveMode::NonRecursive)
        .map_err(|e| e.to_string())?;
    
    tokio::spawn(async move {
        while let Ok(event) = rx.recv() {
            if let Ok(Event { kind: notify::EventKind::Modify(_), .. }) = event {
                app.emit_all("file-changed", &path).unwrap();
            }
        }
    });
    
    Ok(())
}
```

---

## Frontend Implementation

### File Loader Class

**src/utils/file-loader.js:**

```javascript
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import { listen } from '@tauri-apps/api/event';

export class FileLoader {
    constructor() {
        this.currentFile = null;
        this.cache = new Map();
        this.maxCacheSize = 10;
        this.listeners = new Map();
    }
    
    /**
     * Open file dialog and load selected file
     */
    async openDialog() {
        const selected = await open({
            multiple: false,
            filters: [{
                name: 'JSON',
                extensions: ['json']
            }],
            defaultPath: await this.getDefaultPath(),
            title: 'Open BorrowScope Tracking Data'
        });
        
        if (selected) {
            return await this.loadFile(selected);
        }
        
        return null;
    }
    
    /**
     * Load file from path
     */
    async loadFile(path) {
        // Check cache first
        if (this.cache.has(path)) {
            console.log('Loading from cache:', path);
            const cached = this.cache.get(path);
            
            // Verify file hasn't changed
            if (await this.verifyChecksum(path, cached.checksum)) {
                this.currentFile = path;
                return cached.data;
            } else {
                console.log('File changed, reloading...');
                this.cache.delete(path);
            }
        }
        
        // Load from disk
        const result = await invoke('load_json_file', { path });
        
        if (result.success) {
            this.currentFile = path;
            
            // Cache the data
            this.addToCache(path, result.data, result.file_info.checksum);
            
            // Add to recent files
            await this.addToRecent(path);
            
            // Start watching for changes
            await this.watchFile(path);
            
            return result.data;
        } else {
            throw new Error(result.error);
        }
    }
    
    /**
     * Validate file before loading
     */
    async validateFile(path) {
        try {
            const result = await invoke('validate_json_file', { path });
            return result;
        } catch (error) {
            return {
                valid: false,
                errors: [error.toString()],
                warnings: []
            };
        }
    }
    
    /**
     * Verify file checksum
     */
    async verifyChecksum(path, expectedChecksum) {
        try {
            const result = await invoke('load_json_file', { path });
            return result.file_info.checksum === expectedChecksum;
        } catch {
            return false;
        }
    }
    
    /**
     * Add to cache with LRU eviction
     */
    addToCache(path, data, checksum) {
        // Evict oldest if cache is full
        if (this.cache.size >= this.maxCacheSize) {
            const firstKey = this.cache.keys().next().value;
            this.cache.delete(firstKey);
        }
        
        this.cache.set(path, {
            data,
            checksum,
            timestamp: Date.now()
        });
    }
    
    /**
     * Clear cache
     */
    clearCache() {
        this.cache.clear();
    }
    
    /**
     * Get default path for dialog
     */
    async getDefaultPath() {
        try {
            const recent = await invoke('get_recent_files');
            if (recent.length > 0) {
                return recent[0];
            }
        } catch (error) {
            console.error('Failed to get recent files:', error);
        }
        return null;
    }
    
    /**
     * Add to recent files
     */
    async addToRecent(path) {
        try {
            await invoke('add_recent_file', { path });
        } catch (error) {
            console.error('Failed to add to recent:', error);
        }
    }
    
    /**
     * Watch file for changes
     */
    async watchFile(path) {
        try {
            await invoke('watch_file', { path });
            
            // Listen for file change events
            const unlisten = await listen('file-changed', (event) => {
                if (event.payload === path) {
                    this.onFileChanged(path);
                }
            });
            
            this.listeners.set(path, unlisten);
        } catch (error) {
            console.error('Failed to watch file:', error);
        }
    }
    
    /**
     * Handle file change
     */
    async onFileChanged(path) {
        console.log('File changed:', path);
        
        // Remove from cache
        this.cache.delete(path);
        
        // Emit event for UI to handle
        const event = new CustomEvent('file-reload-needed', {
            detail: { path }
        });
        window.dispatchEvent(event);
    }
    
    /**
     * Stop watching file
     */
    stopWatching(path) {
        const unlisten = this.listeners.get(path);
        if (unlisten) {
            unlisten();
            this.listeners.delete(path);
        }
    }
    
    /**
     * Get current file info
     */
    getCurrentFile() {
        return this.currentFile;
    }
    
    /**
     * Get cache statistics
     */
    getCacheStats() {
        return {
            size: this.cache.size,
            maxSize: this.maxCacheSize,
            entries: Array.from(this.cache.keys())
        };
    }
}
```

---

## Drag and Drop Support

**src/utils/drag-drop.js:**

```javascript
export class DragDropHandler {
    constructor(fileLoader, dropZone) {
        this.fileLoader = fileLoader;
        this.dropZone = dropZone;
        this.setupHandlers();
    }
    
    setupHandlers() {
        this.dropZone.addEventListener('dragover', (e) => {
            e.preventDefault();
            e.stopPropagation();
            this.dropZone.classList.add('drag-over');
        });
        
        this.dropZone.addEventListener('dragleave', (e) => {
            e.preventDefault();
            e.stopPropagation();
            this.dropZone.classList.remove('drag-over');
        });
        
        this.dropZone.addEventListener('drop', async (e) => {
            e.preventDefault();
            e.stopPropagation();
            this.dropZone.classList.remove('drag-over');
            
            const files = Array.from(e.dataTransfer.files);
            const jsonFiles = files.filter(f => f.name.endsWith('.json'));
            
            if (jsonFiles.length > 0) {
                await this.handleDrop(jsonFiles[0]);
            } else {
                this.showError('Please drop a JSON file');
            }
        });
    }
    
    async handleDrop(file) {
        try {
            // Tauri doesn't support File objects directly
            // We need to get the file path
            const path = file.path;
            
            if (!path) {
                throw new Error('Cannot access file path');
            }
            
            // Validate before loading
            const validation = await this.fileLoader.validateFile(path);
            
            if (!validation.valid) {
                this.showError(`Invalid file: ${validation.errors.join(', ')}`);
                return;
            }
            
            // Load the file
            const data = await this.fileLoader.loadFile(path);
            
            // Emit success event
            const event = new CustomEvent('file-loaded', { detail: data });
            window.dispatchEvent(event);
            
        } catch (error) {
            this.showError(error.message);
        }
    }
    
    showError(message) {
        const event = new CustomEvent('file-load-error', { 
            detail: { message } 
        });
        window.dispatchEvent(event);
    }
}
```

---

## Usage Example

**src/main.js:**

```javascript
import { FileLoader } from './utils/file-loader.js';
import { DragDropHandler } from './utils/drag-drop.js';

// Initialize file loader
const fileLoader = new FileLoader();

// Setup drag and drop
const dropZone = document.getElementById('app');
const dragDrop = new DragDropHandler(fileLoader, dropZone);

// Handle file load button
document.getElementById('load-btn').addEventListener('click', async () => {
    try {
        const data = await fileLoader.openDialog();
        if (data) {
            renderGraph(data);
        }
    } catch (error) {
        console.error('Failed to load file:', error);
        showError(error.message);
    }
});

// Handle file reload
window.addEventListener('file-reload-needed', async (event) => {
    const { path } = event.detail;
    
    const shouldReload = confirm('File has changed. Reload?');
    if (shouldReload) {
        try {
            const data = await fileLoader.loadFile(path);
            renderGraph(data);
        } catch (error) {
            showError(error.message);
        }
    }
});

// Handle successful file load
window.addEventListener('file-loaded', (event) => {
    const data = event.detail;
    renderGraph(data);
});

// Handle file load errors
window.addEventListener('file-load-error', (event) => {
    showError(event.detail.message);
});

function renderGraph(data) {
    console.log('Rendering graph with', data.metadata.node_count, 'nodes');
    // Render implementation...
}

function showError(message) {
    alert(`Error: ${message}`);
}
```

---

## Testing

### Backend Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_load_valid_file() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.json");
        
        let data = r#"{
            "nodes": [
                {"id": 1, "name": "x", "type_name": "i32", 
                 "created_at": 1000, "dropped_at": null, "scope_depth": 0}
            ],
            "edges": []
        }"#;
        
        fs::write(&file, data).unwrap();
        
        let result = load_json_file(file.to_str().unwrap().to_string()).await.unwrap();
        
        assert!(result.success);
        assert!(result.data.is_some());
        assert_eq!(result.data.unwrap().nodes.len(), 1);
    }

    #[tokio::test]
    async fn test_load_nonexistent_file() {
        let result = load_json_file("nonexistent.json".to_string()).await.unwrap();
        
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_validate_invalid_json() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("invalid.json");
        
        fs::write(&file, "not json").unwrap();
        
        let result = validate_json_file(file.to_str().unwrap().to_string()).await.unwrap();
        
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_checksum_calculation() {
        let content1 = "test content";
        let content2 = "test content";
        let content3 = "different content";
        
        assert_eq!(calculate_checksum(content1), calculate_checksum(content2));
        assert_ne!(calculate_checksum(content1), calculate_checksum(content3));
    }
}
```

### Frontend Tests

```javascript
import { FileLoader } from './file-loader.js';

describe('FileLoader', () => {
    let fileLoader;
    
    beforeEach(() => {
        fileLoader = new FileLoader();
    });
    
    test('initializes with empty cache', () => {
        expect(fileLoader.cache.size).toBe(0);
        expect(fileLoader.currentFile).toBeNull();
    });
    
    test('adds to cache correctly', () => {
        const data = { nodes: [], edges: [] };
        fileLoader.addToCache('test.json', data, 'abc123');
        
        expect(fileLoader.cache.size).toBe(1);
        expect(fileLoader.cache.has('test.json')).toBe(true);
    });
    
    test('evicts oldest when cache is full', () => {
        fileLoader.maxCacheSize = 2;
        
        fileLoader.addToCache('file1.json', {}, 'hash1');
        fileLoader.addToCache('file2.json', {}, 'hash2');
        fileLoader.addToCache('file3.json', {}, 'hash3');
        
        expect(fileLoader.cache.size).toBe(2);
        expect(fileLoader.cache.has('file1.json')).toBe(false);
        expect(fileLoader.cache.has('file3.json')).toBe(true);
    });
    
    test('clears cache', () => {
        fileLoader.addToCache('test.json', {}, 'hash');
        fileLoader.clearCache();
        
        expect(fileLoader.cache.size).toBe(0);
    });
});
```

---

## Key Takeaways

✅ **File loading** - Tauri dialog API integration  
✅ **JSON parsing** - Validation and error handling  
✅ **Caching** - LRU cache for performance  
✅ **File watching** - Auto-reload on changes  
✅ **Drag and drop** - User-friendly file loading  
✅ **Checksums** - Detect file modifications  
✅ **Testing** - Comprehensive test coverage  

---

**Previous:** [91-tauri-commands-and-ipc.md](./91-tauri-commands-and-ipc.md)  
**Next:** [93-frontend-project-setup.md](./93-frontend-project-setup.md)

**Progress:** 4/17 ⬛⬛⬛⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜
