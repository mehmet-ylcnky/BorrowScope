# Section 91: Tauri Commands and IPC

## Learning Objectives

By the end of this section, you will:
- Implement Tauri commands for graph operations
- Handle IPC communication patterns
- Implement type-safe data transfer
- Use async commands for heavy operations
- Implement event-based communication
- Handle errors across the IPC boundary

## Prerequisites

- Section 90 (Setting Up Tauri Project)
- Understanding of async Rust
- Familiarity with JavaScript Promises

---

## Command Basics

### Simple Command

**Backend (Rust):**
```rust
#[tauri::command]
fn simple_command() -> String {
    "Hello from Rust!".to_string()
}
```

**Frontend (JavaScript):**
```javascript
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke('simple_command');
console.log(result); // "Hello from Rust!"
```

### Command with Arguments

**Backend:**
```rust
#[tauri::command]
fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}
```

**Frontend:**
```javascript
const sum = await invoke('add_numbers', { a: 5, b: 3 });
console.log(sum); // 8
```

---

## Graph Loading Commands

### Load Graph from File

**src-tauri/src/commands/graph.rs:**
```rust
use borrowscope_graph::{OwnershipGraph, GraphExport, Variable};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<NodeData>,
    pub edges: Vec<EdgeData>,
    pub metadata: GraphMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeData {
    pub id: usize,
    pub name: String,
    pub type_name: String,
    pub created_at: u64,
    pub dropped_at: Option<u64>,
    pub scope_depth: usize,
    pub is_alive: bool,
}

#[derive(Debug, Serialize, Deserialize)]
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
}

#[tauri::command]
pub async fn load_graph_file(path: String) -> Result<GraphData, String> {
    // Read file
    let content = tokio::fs::read_to_string(&path).await
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    // Parse JSON
    let export: GraphExport = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    
    // Convert to frontend format
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
    
    Ok(GraphData {
        nodes,
        edges,
        metadata: GraphMetadata {
            node_count: export.nodes.len(),
            edge_count: export.edges.len(),
            file_path: path,
        },
    })
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

**Frontend:**
```javascript
async function loadGraph(filePath) {
    try {
        const data = await invoke('load_graph_file', { path: filePath });
        console.log(`Loaded ${data.metadata.node_count} nodes`);
        return data;
    } catch (error) {
        console.error('Failed to load graph:', error);
        throw error;
    }
}
```

---

## Analysis Commands

### Detect Conflicts

**Backend:**
```rust
use borrowscope_graph::{BorrowConflict, ConflictType};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConflictData {
    pub conflict_type: String,
    pub owner_id: usize,
    pub owner_name: String,
    pub borrowers: Vec<BorrowerInfo>,
    pub time_range: (u64, u64),
    pub suggestion: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BorrowerInfo {
    pub id: usize,
    pub name: String,
    pub type_name: String,
}

#[tauri::command]
pub async fn analyze_conflicts(path: String) -> Result<Vec<ConflictData>, String> {
    // Load and reconstruct graph
    let content = tokio::fs::read_to_string(&path).await
        .map_err(|e| e.to_string())?;
    
    let export: GraphExport = serde_json::from_str(&content)
        .map_err(|e| e.to_string())?;
    
    let mut graph = OwnershipGraph::new();
    
    // Reconstruct graph
    for node in export.nodes {
        graph.add_variable(Variable {
            id: node.id,
            name: node.name,
            type_name: node.type_name,
            created_at: node.created_at,
            dropped_at: node.dropped_at,
            scope_depth: node.scope_depth,
        });
    }
    
    for edge in export.edges {
        match edge.relationship {
            borrowscope_graph::Relationship::BorrowsImmut { at } => {
                graph.add_borrow(edge.from_id, edge.to_id, false, at);
            }
            borrowscope_graph::Relationship::BorrowsMut { at } => {
                graph.add_borrow(edge.from_id, edge.to_id, true, at);
            }
            _ => {}
        }
    }
    
    // Find conflicts
    let conflicts = graph.find_conflicts_optimized();
    
    // Convert to frontend format
    let result = conflicts.iter().map(|conflict| {
        let owner = graph.get_variable(conflict.owner_id);
        let owner_name = owner.map(|v| v.name.clone()).unwrap_or_default();
        
        let borrowers = conflict.borrowers.iter()
            .filter_map(|&id| graph.get_variable(id))
            .map(|v| BorrowerInfo {
                id: v.id,
                name: v.name.clone(),
                type_name: v.type_name.clone(),
            })
            .collect();
        
        ConflictData {
            conflict_type: format!("{:?}", conflict.conflict_type),
            owner_id: conflict.owner_id,
            owner_name,
            borrowers,
            time_range: conflict.time_range,
            suggestion: get_suggestion(&conflict.conflict_type),
        }
    }).collect();
    
    Ok(result)
}

fn get_suggestion(conflict_type: &ConflictType) -> String {
    match conflict_type {
        ConflictType::MultipleMutableBorrows => {
            "Only one mutable borrow is allowed at a time. \
             Consider using RefCell for interior mutability.".to_string()
        }
        ConflictType::MutableWithImmutable => {
            "Mutable and immutable borrows cannot coexist. \
             Drop immutable borrows before creating mutable ones.".to_string()
        }
    }
}
```

**Frontend:**
```javascript
async function checkConflicts(filePath) {
    const conflicts = await invoke('analyze_conflicts', { path: filePath });
    
    if (conflicts.length === 0) {
        console.log('No conflicts found!');
    } else {
        console.log(`Found ${conflicts.length} conflict(s):`);
        conflicts.forEach(c => {
            console.log(`- ${c.conflict_type} on ${c.owner_name}`);
            console.log(`  Suggestion: ${c.suggestion}`);
        });
    }
    
    return conflicts;
}
```

---

## Statistics Commands

**Backend:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct GraphStatistics {
    pub total_variables: usize,
    pub alive_variables: usize,
    pub dropped_variables: usize,
    pub total_borrows: usize,
    pub immutable_borrows: usize,
    pub mutable_borrows: usize,
    pub moves: usize,
    pub avg_lifetime: f64,
    pub max_borrow_depth: usize,
    pub most_borrowed: Option<MostBorrowedInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MostBorrowedInfo {
    pub name: String,
    pub borrow_count: usize,
}

#[tauri::command]
pub async fn compute_statistics(path: String) -> Result<GraphStatistics, String> {
    let content = tokio::fs::read_to_string(&path).await
        .map_err(|e| e.to_string())?;
    
    let export: GraphExport = serde_json::from_str(&content)
        .map_err(|e| e.to_string())?;
    
    let mut graph = OwnershipGraph::new();
    
    // Reconstruct graph
    for node in export.nodes {
        graph.add_variable(Variable {
            id: node.id,
            name: node.name,
            type_name: node.type_name,
            created_at: node.created_at,
            dropped_at: node.dropped_at,
            scope_depth: node.scope_depth,
        });
    }
    
    let stats = graph.statistics();
    
    let most_borrowed = graph.query().most_borrowed()
        .map(|(var, count)| MostBorrowedInfo {
            name: var.name.clone(),
            borrow_count: count,
        });
    
    Ok(GraphStatistics {
        total_variables: stats.total_variables,
        alive_variables: stats.alive_variables,
        dropped_variables: stats.total_variables - stats.alive_variables,
        total_borrows: stats.total_borrows,
        immutable_borrows: stats.immutable_borrows,
        mutable_borrows: stats.mutable_borrows,
        moves: stats.moves,
        avg_lifetime: stats.avg_lifetime,
        max_borrow_depth: stats.max_borrow_depth,
        most_borrowed,
    })
}
```

**Frontend:**
```javascript
async function getStatistics(filePath) {
    const stats = await invoke('compute_statistics', { path: filePath });
    
    console.log('Graph Statistics:');
    console.log(`  Total Variables: ${stats.total_variables}`);
    console.log(`  Alive: ${stats.alive_variables}`);
    console.log(`  Dropped: ${stats.dropped_variables}`);
    console.log(`  Total Borrows: ${stats.total_borrows}`);
    console.log(`  Average Lifetime: ${stats.avg_lifetime.toFixed(2)}μs`);
    
    if (stats.most_borrowed) {
        console.log(`  Most Borrowed: ${stats.most_borrowed.name} (${stats.most_borrowed.borrow_count} times)`);
    }
    
    return stats;
}
```

---

## State Management

### Backend State

**src-tauri/src/state.rs:**
```rust
use std::sync::Mutex;
use borrowscope_graph::OwnershipGraph;

pub struct AppState {
    pub current_graph: Mutex<Option<OwnershipGraph>>,
    pub current_file: Mutex<Option<String>>,
    pub recent_files: Mutex<Vec<String>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_graph: Mutex::new(None),
            current_file: Mutex::new(None),
            recent_files: Mutex::new(Vec::new()),
        }
    }
}
```

**Commands using state:**
```rust
use tauri::State;

#[tauri::command]
pub fn set_current_graph(
    graph_data: GraphData,
    state: State<AppState>
) -> Result<(), String> {
    // Convert GraphData to OwnershipGraph
    let mut graph = OwnershipGraph::new();
    
    for node in graph_data.nodes {
        graph.add_variable(Variable {
            id: node.id,
            name: node.name,
            type_name: node.type_name,
            created_at: node.created_at,
            dropped_at: node.dropped_at,
            scope_depth: node.scope_depth,
        });
    }
    
    let mut current = state.current_graph.lock()
        .map_err(|e| e.to_string())?;
    *current = Some(graph);
    
    Ok(())
}

#[tauri::command]
pub fn get_current_file(state: State<AppState>) -> Option<String> {
    state.current_file.lock().ok()
        .and_then(|guard| guard.clone())
}

#[tauri::command]
pub fn add_recent_file(path: String, state: State<AppState>) -> Result<(), String> {
    let mut recent = state.recent_files.lock()
        .map_err(|e| e.to_string())?;
    
    // Remove if already exists
    recent.retain(|p| p != &path);
    
    // Add to front
    recent.insert(0, path);
    
    // Keep only last 10
    recent.truncate(10);
    
    Ok(())
}

#[tauri::command]
pub fn get_recent_files(state: State<AppState>) -> Vec<String> {
    state.recent_files.lock()
        .map(|guard| guard.clone())
        .unwrap_or_default()
}
```

---

## Event-Based Communication

### Backend Events

**Emit progress updates:**
```rust
use tauri::Manager;

#[tauri::command]
pub async fn process_large_graph(
    path: String,
    app: tauri::AppHandle
) -> Result<(), String> {
    let content = tokio::fs::read_to_string(&path).await
        .map_err(|e| e.to_string())?;
    
    app.emit_all("progress", 0.1).unwrap();
    
    let export: GraphExport = serde_json::from_str(&content)
        .map_err(|e| e.to_string())?;
    
    app.emit_all("progress", 0.3).unwrap();
    
    // Process nodes
    for (i, node) in export.nodes.iter().enumerate() {
        // Process node...
        
        if i % 100 == 0 {
            let progress = 0.3 + (i as f64 / export.nodes.len() as f64) * 0.7;
            app.emit_all("progress", progress).unwrap();
        }
    }
    
    app.emit_all("progress", 1.0).unwrap();
    app.emit_all("processing-complete", ()).unwrap();
    
    Ok(())
}
```

**Frontend listener:**
```javascript
import { listen } from '@tauri-apps/api/event';

// Listen for progress updates
const unlisten = await listen('progress', (event) => {
    const progress = event.payload;
    updateProgressBar(progress * 100);
});

// Listen for completion
await listen('processing-complete', () => {
    console.log('Processing complete!');
    hideProgressBar();
});

// Clean up listener when done
// unlisten();
```

---

## Error Handling

### Custom Error Type

**Backend:**
```rust
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AppError {
    pub message: String,
    pub kind: ErrorKind,
    pub details: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    FileNotFound,
    ParseError,
    InvalidData,
    IoError,
    Unknown,
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError {
            message: err.to_string(),
            kind: ErrorKind::IoError,
            details: None,
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError {
            message: "Failed to parse JSON".to_string(),
            kind: ErrorKind::ParseError,
            details: Some(err.to_string()),
        }
    }
}

#[tauri::command]
pub async fn safe_load_graph(path: String) -> Result<GraphData, AppError> {
    if !std::path::Path::new(&path).exists() {
        return Err(AppError {
            message: format!("File not found: {}", path),
            kind: ErrorKind::FileNotFound,
            details: None,
        });
    }
    
    let content = tokio::fs::read_to_string(&path).await?;
    let export: GraphExport = serde_json::from_str(&content)?;
    
    // Convert to GraphData...
    
    Ok(graph_data)
}
```

**Frontend:**
```javascript
async function safeLoadGraph(path) {
    try {
        const data = await invoke('safe_load_graph', { path });
        return { success: true, data };
    } catch (error) {
        console.error('Load failed:', error);
        
        // Error is structured
        if (error.kind === 'file_not_found') {
            showError('File not found', error.message);
        } else if (error.kind === 'parse_error') {
            showError('Invalid file format', error.details);
        } else {
            showError('Unknown error', error.message);
        }
        
        return { success: false, error };
    }
}
```

---

## Async Commands

### Long-Running Operations

**Backend:**
```rust
#[tauri::command]
pub async fn analyze_large_graph(path: String) -> Result<AnalysisResult, String> {
    // Spawn blocking task for CPU-intensive work
    let result = tokio::task::spawn_blocking(move || {
        // Load graph
        let content = std::fs::read_to_string(&path)?;
        let export: GraphExport = serde_json::from_str(&content)?;
        
        // Heavy computation
        let mut graph = OwnershipGraph::new();
        // ... reconstruct graph ...
        
        // Analyze
        let conflicts = graph.find_conflicts_optimized();
        let stats = graph.statistics();
        
        Ok::<_, String>(AnalysisResult { conflicts, stats })
    }).await.map_err(|e| e.to_string())?;
    
    result
}
```

---

## Register Commands

**src-tauri/src/main.rs:**
```rust
mod commands;
mod state;

use commands::graph::*;
use state::AppState;

fn main() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            // Graph commands
            load_graph_file,
            analyze_conflicts,
            compute_statistics,
            process_large_graph,
            
            // State commands
            set_current_graph,
            get_current_file,
            add_recent_file,
            get_recent_files,
            
            // Safe commands
            safe_load_graph,
            analyze_large_graph,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## Testing Commands

**Backend tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_graph() {
        let result = load_graph_file("test_data.json".to_string()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_conflicts() {
        let result = analyze_conflicts("test_data.json".to_string()).await;
        assert!(result.is_ok());
    }
}
```

**Frontend tests:**
```javascript
import { invoke } from '@tauri-apps/api/tauri';

describe('Graph Commands', () => {
    test('loads graph successfully', async () => {
        const data = await invoke('load_graph_file', { 
            path: 'test_data.json' 
        });
        
        expect(data.nodes).toBeDefined();
        expect(data.edges).toBeDefined();
    });
    
    test('handles file not found', async () => {
        await expect(
            invoke('load_graph_file', { path: 'nonexistent.json' })
        ).rejects.toThrow();
    });
});
```

---

## Key Takeaways

✅ **Commands** - Type-safe IPC communication  
✅ **Async operations** - Handle long-running tasks  
✅ **State management** - Share state across commands  
✅ **Events** - Push updates to frontend  
✅ **Error handling** - Structured error types  
✅ **Testing** - Test commands in isolation  

---

**Previous:** [90-setting-up-tauri-project.md](./90-setting-up-tauri-project.md)  
**Next:** [92-loading-and-parsing-json.md](./92-loading-and-parsing-json.md)

**Progress:** 3/17 ⬛⬛⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜
