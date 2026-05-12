# Milestone 6: Real-Time Incremental Updates - Detailed Specification

## 6.1 Salsa Incremental Computation

**Objective:** Leverage the Salsa framework (used internally by `ra_ap_*`) to ensure that only the minimum necessary recomputation happens after a file change. When the user edits a single function, only that function's ownership analysis is recomputed. All other functions return cached results instantly.

**How Salsa works:**
```
┌─────────────────────────────────────────────────────────────────┐
│  Salsa Dependency Graph (automatic, invisible to our code)       │
│                                                                  │
│  file_text("main.rs")                                            │
│       │                                                          │
│       ▼                                                          │
│  parse("main.rs")  ──────────────────────────────────────┐      │
│       │                                                   │      │
│       ▼                                                   ▼      │
│  item_tree("main.rs")                              resolve_type  │
│       │                                            (for fn bar)  │
│       ▼                                                          │
│  resolve_type(var "x" in fn main)  ← INVALIDATED on edit        │
│       │                                                          │
│  resolve_type(var "y" in fn bar)   ← CACHED (fn bar unchanged)  │
└─────────────────────────────────────────────────────────────────┘
```

**Steps:**
1. When VFS reports a file change, call `db.apply_change(change)`
2. Salsa marks all queries that transitively depend on the changed file as "dirty"
3. On next access, dirty queries recompute; clean queries return cached values
4. Our analysis layer calls `sema.type_of_pat()` etc. which internally uses Salsa
5. Only the affected function's analysis recomputes

**Code (state.rs):**
```rust
impl GlobalState {
    /// Apply pending VFS changes to the Salsa database.
    /// Returns the list of files that were modified.
    pub fn apply_pending_changes(&mut self) -> Vec<FileId> {
        let ws = match &mut self.workspace {
            Some(ws) => ws,
            None => return vec![],
        };

        let changes = ws.vfs.take_changes();
        if changes.is_empty() {
            return vec![];
        }

        let mut change = ra_ap_ide::Change::new();
        let mut modified_files = Vec::new();

        for vfs_change in &changes {
            let file_id = vfs_change.file_id;
            modified_files.push(file_id);

            if vfs_change.exists() {
                let content = ws.vfs.file_contents(file_id).to_vec();
                change.change_file(file_id, Some(content));
            } else {
                change.change_file(file_id, None); // File deleted
            }
        }

        ws.db.apply_change(change);
        tracing::debug!("Applied {} file changes", modified_files.len());
        modified_files
    }
}
```

**Performance characteristics:**
```
Scenario                          │ Without Salsa  │ With Salsa
──────────────────────────────────┼────────────────┼──────────────
Edit one function in 100-fn file  │ Re-analyze all │ Re-analyze 1
                                  │ ~5000ms        │ ~50ms
Add a comment (no semantic change)│ Re-analyze all │ Re-parse only
                                  │ ~5000ms        │ ~5ms
Change a type definition          │ Re-analyze all │ Re-analyze
                                  │ ~5000ms        │ dependents ~200ms
```

**Expectation:** Single-function edits produce updated analysis in < 100ms. The user perceives instant feedback.

**Tests for 6.1:**
- Editing function A does not invalidate analysis of function B
- Editing a comment does not trigger re-analysis (parse changes but types don't)
- Changing a type definition invalidates all functions using that type
- `apply_pending_changes()` returns correct list of modified files
- After change, `type_of_pat()` returns the new type (not stale cached value)
- Performance: single-function edit → analysis ready in < 100ms

---

## 6.2 Debounced Analysis

**Objective:** Avoid re-analyzing on every keystroke. Instead, wait until the user pauses typing (300ms debounce), then trigger analysis. This prevents wasted computation during rapid editing while still feeling responsive.

**Steps:**
1. On each `didChange` notification, reset a debounce timer
2. When the timer fires (300ms after last change), trigger analysis
3. If another change arrives before the timer fires, reset it again
4. Send `analysisUpdated` notification only after debounced analysis completes

**Code (server.rs):**
```rust
use std::time::{Duration, Instant};

struct AnalysisScheduler {
    last_change: Instant,
    debounce_duration: Duration,
    pending_files: HashSet<FileId>,
    timer_handle: Option<JoinHandle<()>>,
}

impl AnalysisScheduler {
    fn new() -> Self {
        Self {
            last_change: Instant::now(),
            debounce_duration: Duration::from_millis(300),
            pending_files: HashSet::new(),
            timer_handle: None,
        }
    }

    fn on_file_changed(&mut self, file_id: FileId) {
        self.last_change = Instant::now();
        self.pending_files.insert(file_id);

        // Cancel existing timer
        if let Some(handle) = self.timer_handle.take() {
            handle.abort();
        }

        // Start new timer
        let duration = self.debounce_duration;
        let sender = self.sender.clone();
        self.timer_handle = Some(tokio::spawn(async move {
            tokio::time::sleep(duration).await;
            sender.send(InternalMessage::AnalyzeNow).ok();
        }));
    }

    fn on_analyze_now(&mut self) -> Vec<FileId> {
        let files: Vec<_> = self.pending_files.drain().collect();
        files
    }
}
```

**Timing diagram:**
```
User types:  a   b   c   d   [pause 300ms]   e   f   [pause 300ms]
             │   │   │   │        │           │   │        │
Debounce:    ├───┼───┼───┤        │           ├───┤        │
             reset reset reset    FIRE         reset       FIRE
                                   │                        │
Analysis:                     analyze(abcd)            analyze(ef)
                                   │                        │
Notification:                 analysisUpdated          analysisUpdated
```

**Expectation:** Rapid typing produces at most one analysis per pause. The 300ms debounce is configurable via extension settings.

**Tests for 6.2:**
- Single change followed by 300ms pause triggers analysis
- Rapid changes (< 300ms apart) produce only one analysis
- Debounce duration is configurable
- Analysis fires immediately if debounce is set to 0
- Pending files accumulate correctly across multiple changes
- Timer cancellation doesn't leak resources

---

## 6.3 Partial Results

**Objective:** While analysis is in progress (especially during initial workspace loading), show whatever data is already available rather than an empty state. Functions that have been analyzed show their graph; functions still pending show a loading indicator.

**Steps:**
1. Track analysis state per function: `NotAnalyzed`, `Analyzing`, `Ready`
2. When client requests a graph, return cached result if available (even if stale)
3. Send partial updates as functions complete analysis
4. Show "analyzing..." indicator for pending functions in the WebView

**Code (state.rs):**
```rust
#[derive(Debug, Clone, PartialEq)]
enum AnalysisState {
    NotAnalyzed,
    Analyzing,
    Ready(FunctionOwnershipSummary),
    Stale(FunctionOwnershipSummary), // Data available but may be outdated
}

struct FileAnalysisCache {
    functions: HashMap<String, AnalysisState>,
}

impl FileAnalysisCache {
    fn get_or_stale(&self, function_name: &str) -> Option<&FunctionOwnershipSummary> {
        match self.functions.get(function_name) {
            Some(AnalysisState::Ready(s)) => Some(s),
            Some(AnalysisState::Stale(s)) => Some(s), // Return stale data while re-analyzing
            _ => None,
        }
    }

    fn mark_stale(&mut self, function_name: &str) {
        if let Some(state) = self.functions.get_mut(function_name) {
            if let AnalysisState::Ready(summary) = state.clone() {
                *state = AnalysisState::Stale(summary);
            }
        }
    }
}
```

**User experience:**
```
┌─────────────────────────────────────────────┐
│  Ownership Graph: process_data()             │
│                                              │
│  ┌─────┐      ┌─────┐                       │
│  │ data │──&──▶│  r  │   ← rendered (ready) │
│  └─────┘      └─────┘                       │
│                                              │
│  ⟳ Updating... (file changed 0.2s ago)      │
└─────────────────────────────────────────────┘
```

**Expectation:** The graph is never blank. Stale data is shown with a subtle "updating..." indicator until fresh data arrives. This prevents the jarring experience of the graph disappearing and reappearing on every keystroke.

**Tests for 6.3:**
- Stale data is returned while re-analysis is in progress
- "Analyzing" state is reported to client
- Fresh data replaces stale data when analysis completes
- Functions not yet analyzed show loading state (not empty)
- Cache is cleared when file is closed

---

## 6.4 Diff-Based UI Updates

**Objective:** When the graph data changes, compute the diff between old and new data and send only the changes to the WebView. This prevents full re-renders and enables smooth animations (nodes sliding to new positions rather than jumping).

**Steps:**
1. Compare old graph data with new graph data
2. Identify: added nodes, removed nodes, modified nodes, added edges, removed edges
3. Send a `graphDiff` message to the WebView instead of full `updateGraph`
4. WebView applies the diff with D3.js enter/update/exit pattern

**Code (extension side):**
```typescript
interface GraphDiff {
    addedNodes: GraphNode[];
    removedNodeIds: string[];
    modifiedNodes: { id: string; changes: Partial<GraphNode> }[];
    addedEdges: GraphEdge[];
    removedEdgeIds: string[];
}

function computeGraphDiff(oldData: GraphData, newData: GraphData): GraphDiff {
    const oldNodeIds = new Set(oldData.nodes.map(n => n.id));
    const newNodeIds = new Set(newData.nodes.map(n => n.id));

    return {
        addedNodes: newData.nodes.filter(n => !oldNodeIds.has(n.id)),
        removedNodeIds: oldData.nodes.filter(n => !newNodeIds.has(n.id)).map(n => n.id),
        modifiedNodes: newData.nodes
            .filter(n => oldNodeIds.has(n.id))
            .filter(n => JSON.stringify(n) !== JSON.stringify(oldData.nodes.find(o => o.id === n.id)))
            .map(n => ({ id: n.id, changes: n })),
        addedEdges: newData.edges.filter(e => !oldData.edges.find(o => o.id === e.id)),
        removedEdgeIds: oldData.edges.filter(e => !newData.edges.find(n => n.id === e.id)).map(e => e.id),
    };
}
```

**Code (WebView side):**
```javascript
function applyGraphDiff(diff) {
    // Remove nodes with exit animation
    diff.removedNodeIds.forEach(id => {
        d3.select(`#node-${id}`)
            .transition().duration(300)
            .style('opacity', 0)
            .remove();
    });

    // Add nodes with enter animation
    const newNodes = nodeGroup.selectAll('g.node.new')
        .data(diff.addedNodes, d => d.id)
        .join('g')
        .attr('class', 'node new')
        .style('opacity', 0);
    // ... render new nodes ...
    newNodes.transition().duration(300).style('opacity', 1);

    // Update modified nodes (animate position/color changes)
    diff.modifiedNodes.forEach(({ id, changes }) => {
        const node = d3.select(`#node-${id}`);
        if (changes.category) {
            node.select('rect')
                .transition().duration(300)
                .attr('fill', nodeColor(changes.category));
        }
    });

    // Update simulation with new data
    simulation.nodes(currentNodes);
    simulation.force('link').links(currentEdges);
    simulation.alpha(0.3).restart();
}
```

**Expectation:** Small edits (adding a variable, changing a type) produce smooth animations rather than full graph rebuilds. The layout remains stable for unchanged nodes.

**Tests for 6.4:**
- Adding a variable: new node fades in, existing nodes don't move
- Removing a variable: node fades out, remaining nodes adjust smoothly
- Changing a type: node color transitions smoothly
- Adding a borrow: new edge draws in with animation
- No-op edit (comment change): no diff, no re-render
- Diff computation is < 5ms for typical graphs (< 50 nodes)

---

## 6.5 Performance Budget

**Objective:** Define and enforce performance targets for the entire update pipeline. From keystroke to visual update, the total latency must be under 500ms for the user to perceive the system as "real-time."

**Performance budget breakdown:**
```
Keystroke
    │
    │  0ms     VS Code sends didChange to server
    │
    │  300ms   Debounce timer fires
    │
    │  50ms    Salsa incremental re-analysis (single function)
    │
    │  10ms    Compute graph diff
    │
    │  5ms     Send LSP notification + response
    │
    │  50ms    D3.js re-render with animation
    │
    │  ────────────────────────────────────
    │  Total: ~415ms (within 500ms budget)
    ▼
Visual update visible
```

**Monitoring code (server side):**
```rust
pub fn analyze_with_timing(
    db: &dyn HirDatabase,
    sema: &Semantics<'_, RootDatabase>,
    function: &ast::Fn,
) -> (FunctionOwnershipSummary, Duration) {
    let start = Instant::now();
    let result = analyze_function(db, sema, function);
    let elapsed = start.elapsed();

    if elapsed > Duration::from_millis(100) {
        tracing::warn!(
            "Analysis of {} took {:?} (exceeds 100ms budget)",
            result.function_name, elapsed
        );
    }

    (result, elapsed)
}
```

**Expectation:** 95% of updates complete within 500ms. Functions with > 50 variables may exceed the budget; these are logged as warnings for optimization.

**Tests for 6.5:**
- Single-variable addition: full pipeline < 500ms
- Type change: full pipeline < 500ms
- Function with 20 variables: analysis < 100ms
- Function with 50 variables: analysis < 200ms
- Diff computation: < 10ms for 50-node graph
- WebView re-render: < 100ms (measured via performance.now())

---

## 6.6 Memory Management

**Objective:** Manage memory usage as files are opened and closed. Analysis results for closed files should be evicted to prevent unbounded memory growth. Open files retain their analysis cache for instant responses.

**Steps:**
1. Track which files are open (via `didOpen`/`didClose`)
2. Keep analysis cache for open files
3. Evict cache for closed files after a grace period (30 seconds)
4. Monitor total memory usage; evict LRU entries if threshold exceeded
5. Log memory usage periodically for debugging

**Code:**
```rust
struct AnalysisCache {
    entries: HashMap<FileId, CacheEntry>,
    max_memory_mb: usize,
    eviction_grace_period: Duration,
}

struct CacheEntry {
    data: FileAnalysisCache,
    last_accessed: Instant,
    is_open: bool,
    estimated_size_bytes: usize,
}

impl AnalysisCache {
    fn evict_closed_files(&mut self) {
        let now = Instant::now();
        self.entries.retain(|_, entry| {
            entry.is_open || now.duration_since(entry.last_accessed) < self.eviction_grace_period
        });
    }

    fn evict_if_over_budget(&mut self) {
        let total_mb: usize = self.entries.values()
            .map(|e| e.estimated_size_bytes)
            .sum::<usize>() / (1024 * 1024);

        if total_mb > self.max_memory_mb {
            // Evict least recently accessed closed files
            let mut closed: Vec<_> = self.entries.iter()
                .filter(|(_, e)| !e.is_open)
                .map(|(id, e)| (*id, e.last_accessed))
                .collect();
            closed.sort_by_key(|(_, t)| *t);

            for (id, _) in closed {
                self.entries.remove(&id);
                let new_total: usize = self.entries.values()
                    .map(|e| e.estimated_size_bytes).sum::<usize>() / (1024 * 1024);
                if new_total <= self.max_memory_mb { break; }
            }
        }
    }
}
```

**Memory budget:**
```
Component                    │ Estimated Memory
─────────────────────────────┼─────────────────
RootDatabase (Salsa)         │ 500MB - 1GB (workspace dependent)
Analysis cache (10 files)    │ ~50MB
VFS (file contents)          │ ~20MB
LSP message buffers          │ ~5MB
─────────────────────────────┼─────────────────
Total                        │ ~600MB - 1.1GB
```

**Expectation:** Memory usage stays bounded even with many files opened and closed over a long session. The server does not grow unboundedly.

**Tests for 6.6:**
- Closing a file evicts its cache after grace period
- Opening a file creates a cache entry
- Memory stays under budget with 20 files opened/closed
- LRU eviction removes oldest closed files first
- Open files are never evicted (even under memory pressure)
- `estimated_size_bytes` is approximately correct (within 2x)

---

## 6.T Integration Test Suite

```rust
#[test]
fn test_incremental_single_function_edit() {
    let mut server = TestServer::start();
    server.initialize(fixture_path());
    server.wait_ready();
    server.open_file("src/main.rs");

    // Get initial analysis
    let graph1 = server.request_ownership_graph("src/main.rs", 5, 0);
    let initial_vars = graph1.stats.total_variables;

    // Edit: add a variable
    server.change_file("src/main.rs", insert_line(10, "    let new_var = 42;"));

    // Wait for debounced analysis
    server.wait_for_notification::<AnalysisUpdated>();

    // Get updated analysis
    let graph2 = server.request_ownership_graph("src/main.rs", 5, 0);
    assert_eq!(graph2.stats.total_variables, initial_vars + 1);
}

#[test]
fn test_debounce_coalesces_rapid_changes() {
    let mut server = TestServer::start();
    server.initialize(fixture_path());
    server.wait_ready();
    server.open_file("src/main.rs");

    // Send 10 rapid changes
    for i in 0..10 {
        server.change_file("src/main.rs", insert_line(10 + i, &format!("    let v{} = {};", i, i)));
        std::thread::sleep(Duration::from_millis(50)); // < 300ms debounce
    }

    // Should receive only ONE analysisUpdated notification
    let notif = server.wait_for_notification::<AnalysisUpdated>();
    // No second notification within 500ms
    assert!(server.try_receive_notification::<AnalysisUpdated>(Duration::from_millis(500)).is_none());
}

#[test]
fn test_performance_budget() {
    let mut server = TestServer::start();
    server.initialize(fixture_path());
    server.wait_ready();
    server.open_file("src/main.rs");

    let start = Instant::now();
    server.change_file("src/main.rs", insert_line(10, "    let x = vec![1, 2, 3];"));
    server.wait_for_notification::<AnalysisUpdated>();
    let elapsed = start.elapsed();

    // Total pipeline (including 300ms debounce) should be < 500ms
    assert!(elapsed < Duration::from_millis(800)); // 300ms debounce + 500ms budget
}
```
