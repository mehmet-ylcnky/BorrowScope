# Milestone 9: Coexistence with rust-analyzer - Detailed Specification

## 9.1 Shared Workspace Loading

**Objective:** Both rust-analyzer and borrowscope-lsp load the same workspace, which means duplicate sysroot discovery, dependency resolution, and standard library indexing. This milestone explores strategies to reduce the duplication without coupling the two servers.

**The problem:**
```
┌─────────────────────────────────────────────────────────────┐
│  Current: Two independent workspace loads                    │
│                                                              │
│  rust-analyzer:     load_workspace() → 500MB-1GB RAM        │
│  borrowscope-lsp:   load_workspace() → 500MB-1GB RAM        │
│                                                              │
│  Total: 1-2GB RAM for the same project                      │
│  Both resolve the same Cargo.toml, same sysroot, same deps  │
└─────────────────────────────────────────────────────────────┘
```

**Mitigation strategies (in order of feasibility):**

**Strategy A: Accept the duplication (simplest)**
- Modern machines have 16-32GB RAM; 1-2GB is acceptable
- No coupling between servers; independent release cycles
- This is what clippy, miri, and other tools do

**Strategy B: Shared sysroot cache on disk**
- Both servers discover the same sysroot path
- The OS filesystem cache ensures the sysroot metadata is only read from disk once
- No code changes needed; the OS handles it transparently

**Strategy C: Delayed loading (load only what's needed)**
- borrowscope-lsp doesn't need the full workspace immediately
- Load only the active file's crate initially
- Load additional crates on-demand when cross-crate types are encountered

**Code (workspace.rs - Strategy C):**
```rust
pub struct LazyWorkspace {
    /// Fully loaded crates (active file + dependencies as needed)
    loaded_crates: HashSet<CrateId>,
    /// The full workspace graph (lightweight, just Cargo.toml structure)
    workspace: ProjectWorkspace,
    /// Database (grows as crates are loaded)
    db: RootDatabase,
}

impl LazyWorkspace {
    pub fn load_minimal(root: &Path) -> anyhow::Result<Self> {
        // Load only workspace structure (fast, < 1s)
        let workspace = ProjectWorkspace::discover(root, &CargoConfig::default())?;

        // Load sysroot (required for std types)
        let sysroot = Sysroot::discover(root)?;

        // Create database with sysroot only
        let mut db = RootDatabase::default();
        db.set_sysroot(sysroot);

        Ok(Self {
            loaded_crates: HashSet::new(),
            workspace,
            db,
        })
    }

    pub fn ensure_crate_loaded(&mut self, file_id: FileId) {
        let crate_id = self.workspace.crate_for_file(file_id);
        if self.loaded_crates.contains(&crate_id) {
            return; // Already loaded
        }

        // Load this crate and its direct dependencies
        load_crate_into_db(&mut self.db, &self.workspace, crate_id);
        self.loaded_crates.insert(crate_id);
    }
}
```

**Startup time comparison:**
```
Strategy              │ Initial Load  │ First Analysis │ RAM
──────────────────────┼───────────────┼────────────────┼──────────
A: Full load          │ 30-40s        │ instant        │ ~1GB
C: Lazy load          │ 1-2s          │ 5-10s (first)  │ ~200MB initially
                      │               │ instant (after)│ grows to ~800MB
```

**Expectation:** The extension starts quickly (< 5s to first useful output) even if full analysis takes longer. Strategy C is preferred for UX; Strategy A is the fallback if lazy loading proves too complex.

**Tests for 9.1:**
- Server starts and responds within 5 seconds (lazy mode)
- Active file's types resolve correctly after lazy load
- Cross-crate types resolve after on-demand loading
- Memory usage stays under 500MB for single-crate projects
- Full workspace eventually loads in background
- Both servers running simultaneously doesn't crash the system

---

## 9.2 Complementary Diagnostics

**Objective:** BorrowScope diagnostics should complement, not duplicate, rust-analyzer's borrow checker errors. RA already shows "cannot borrow `x` as mutable because it is also borrowed as immutable." BorrowScope should show educational context (scope visualization, relationship explanation) rather than repeating the same error.

**Rules for diagnostic deduplication:**
```
┌─────────────────────────────────────────────────────────────┐
│  rust-analyzer shows:                                        │
│  - Compiler errors (E0502, E0505, etc.)                     │
│  - "cannot borrow X as mutable..."                          │
│  - Red squiggles, error severity                            │
│                                                              │
│  BorrowScope shows:                                          │
│  - Educational context (WHY the conflict exists)            │
│  - Borrow scope visualization (WHERE the overlap is)        │
│  - Ownership graph (HOW variables relate)                   │
│  - Information severity (blue, not red)                     │
│  - Only when RA is NOT already showing the same error       │
└─────────────────────────────────────────────────────────────┘
```

**Steps:**
1. Before publishing diagnostics, check if rust-analyzer already has a diagnostic at the same location
2. If RA has a borrow error (E0502, E0505, E0499, E0382), suppress BorrowScope's diagnostic for that specific conflict
3. Instead, add "related information" to BorrowScope's diagnostic pointing to the borrow scopes
4. For patterns that compile successfully (no RA error), BorrowScope can show informational hints about ownership complexity

**Code (server - diagnostics.rs):**
```rust
pub fn filter_diagnostics(
    our_conflicts: &[BorrowConflict],
    ra_diagnostics: &[Diagnostic], // from client, if available
) -> Vec<BorrowConflict> {
    // If we can't see RA's diagnostics, publish all of ours as Information
    // The user will see both, but ours are clearly labeled [BorrowScope]
    // and are Information severity (not Error), so they don't duplicate

    // Key: we NEVER use Error severity. Only Information or Hint.
    // This ensures BorrowScope diagnostics are visually distinct from RA's errors.
    our_conflicts.iter()
        .map(|c| c.clone())
        .collect()
}

fn diagnostic_severity(conflict: &BorrowConflict) -> DiagnosticSeverity {
    // Always Information - never Error (that's RA's job)
    DiagnosticSeverity::INFORMATION
}

fn diagnostic_message(conflict: &BorrowConflict) -> String {
    // Educational message, not just "error: cannot borrow"
    match &conflict.conflict_kind {
        ConflictKind::MutableAndShared => format!(
            "Ownership insight: `{}` holds a shared borrow (&) of `{}` while `{}` holds a mutable borrow (&mut). \
             These scopes overlap on lines {}-{}.",
            conflict.borrow_a.borrower, conflict.variable,
            conflict.borrow_b.borrower,
            conflict.overlap_start_line, conflict.overlap_end_line
        ),
        ConflictKind::MultipleMutable => format!(
            "Ownership insight: Both `{}` and `{}` hold mutable borrows (&mut) of `{}`. \
             Only one &mut can exist at a time.",
            conflict.borrow_a.borrower, conflict.borrow_b.borrower, conflict.variable
        ),
    }
}
```

**Expectation:** Users see RA's errors (red, actionable) AND BorrowScope's insights (blue, educational) without confusion. The two never show the same message.

**Tests for 9.2:**
- BorrowScope diagnostics are always Information severity (never Error)
- BorrowScope diagnostic messages are educational (explain WHY, not just WHAT)
- BorrowScope diagnostics have source="BorrowScope" (distinguishable from RA)
- For valid code (no RA errors), BorrowScope can still show ownership insights
- Diagnostic messages include line numbers for borrow scopes
- No duplicate messages between RA and BorrowScope

---

## 9.3 Hover Integration

**Objective:** When the user hovers over a variable, rust-analyzer shows type information. BorrowScope can extend this with ownership context: who borrows this variable, where it moves, its ownership category. This is done via the standard `textDocument/hover` response, which VS Code merges from multiple providers.

**Challenge:** VS Code shows hover from the first provider that responds. If both RA and BorrowScope provide hover, only one is shown. Solution: BorrowScope provides hover only for ownership-specific information, formatted to complement (not replace) RA's type info.

**Code (handlers/requests.rs):**
```rust
pub fn handle_hover(
    state: &GlobalState,
    params: HoverParams,
) -> anyhow::Result<Option<Hover>> {
    let ws = state.workspace.as_ref().ok_or_else(|| anyhow::anyhow!("Not ready"))?;

    let file_id = uri_to_file_id(&ws.vfs, &params.text_document_position_params.text_document.uri)?;
    let position = params.text_document_position_params.position;
    let offset = lsp_position_to_offset(&ws.vfs, file_id, position)?;

    let sema = Semantics::new(&ws.db);
    let source_file = sema.parse(file_id);

    // Find variable at cursor
    let var_info = find_variable_at_offset(&sema, &source_file, offset)?;

    let markdown = format!(
        "**BorrowScope** — `{}`\n\n\
         | Property | Value |\n\
         |----------|-------|\n\
         | Category | {} |\n\
         | Copy | {} |\n\
         | Borrowed by | {} |\n\
         | Borrows from | {} |\n\
         | Moved to | {} |\n",
        var_info.name,
        var_info.ownership_category,
        if var_info.is_copy { "yes" } else { "no" },
        var_info.borrowed_by.join(", "),
        var_info.borrows_from.join(", "),
        var_info.moved_to.as_deref().unwrap_or("(not moved)"),
    );

    Ok(Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: markdown,
        }),
        range: None,
    }))
}
```

**Hover appearance (combined with RA):**
```
┌─────────────────────────────────────────────┐
│ let data: Vec<i32>                           │  ← RA provides this
│                                              │
│ ─────────────────────────────────────────── │
│                                              │
│ BorrowScope — `data`                         │  ← We provide this
│                                              │
│ | Property    | Value              |         │
│ |-------------|---------------------|        │
│ | Category    | Owned               |        │
│ | Copy        | no                  |        │
│ | Borrowed by | r (line 8), m (L12) |        │
│ | Moved to    | result (line 20)    |        │
└─────────────────────────────────────────────┘
```

**Expectation:** Hover shows ownership relationships in a compact table format. It appears below RA's type information (VS Code concatenates hover results from multiple providers).

**Tests for 9.3:**
- Hover on a variable returns ownership info
- Hover on non-variable (keyword, operator) returns null
- Hover content is valid Markdown
- "Borrowed by" lists all borrowers with line numbers
- "Moved to" shows destination if moved
- Hover response time < 50ms

---

## 9.4 Go-to-Definition Awareness

**Objective:** When the user navigates to a different function (via Go to Definition, clicking a function call, etc.), the ownership graph panel automatically updates to show the target function's ownership structure. The graph follows the user's navigation.

**Steps:**
1. Listen for `window.onDidChangeActiveTextEditor` events
2. Listen for cursor position changes (debounced)
3. Detect when the cursor enters a different function
4. Automatically request the new function's ownership graph
5. Update the graph panel

**Code (extension.ts):**
```typescript
let currentFunction: string | undefined;

vscode.window.onDidChangeTextEditorSelection(
    debounce(async (event) => {
        const editor = event.textEditor;
        if (editor.document.languageId !== 'rust') return;

        // Ask server which function the cursor is in
        const response = await client.sendRequest('borrowscope/currentFunction', {
            textDocument: { uri: editor.document.uri.toString() },
            position: editor.selection.active,
        });

        if (response.functionName && response.functionName !== currentFunction) {
            currentFunction = response.functionName;

            // Auto-update graph panel if it's open
            if (GraphPanel.currentPanel) {
                const graph = await client.sendRequest('borrowscope/ownershipGraph', {
                    textDocument: { uri: editor.document.uri.toString() },
                    position: editor.selection.active,
                });
                GraphPanel.currentPanel.updateGraph(graph);
            }
        }
    }, 500) // 500ms debounce to avoid rapid updates during scrolling
);
```

**Expectation:** The graph panel always shows the function the user is currently editing. Navigation between functions automatically updates the graph without manual action.

**Tests for 9.4:**
- Moving cursor into a different function updates the graph
- Rapid cursor movement (scrolling) doesn't spam requests (debounced)
- Go-to-Definition to another file updates the graph
- Graph panel title shows current function name
- If graph panel is closed, no requests are made (no wasted work)

---

## 9.5 Semantic Token Coordination

**Objective:** Ensure BorrowScope's inline decorations don't visually conflict with rust-analyzer's semantic highlighting. RA colors variables by type (blue for functions, green for types, etc.). BorrowScope's ownership hints should not clash with these colors.

**Rules:**
1. BorrowScope decorations use opacity/background, not text color (RA owns text color)
2. Borrow scope highlights use very low opacity (8-10%) to not obscure RA's highlighting
3. Gutter icons are in the gutter (separate from RA's inline decorations)
4. Inlay hints use italic style to distinguish from RA's type hints

**Code (decorations.ts):**
```typescript
// BorrowScope uses BACKGROUND color (not text color) to avoid conflicts
const borrowScopeDecoration = vscode.window.createTextEditorDecorationType({
    // Background only - doesn't affect RA's text coloring
    backgroundColor: 'rgba(52, 152, 219, 0.08)',
    // No color, fontStyle, or fontWeight - those are RA's domain
    isWholeLine: true,
});

// Inlay hints use italic to distinguish from RA's type annotations
const ownershipHint = vscode.window.createTextEditorDecorationType({
    after: {
        fontStyle: 'italic',        // RA's hints are not italic
        color: 'rgba(150, 150, 150, 0.6)', // Subtle, doesn't compete
        margin: '0 0 0 1em',
    }
});
```

**Visual separation:**
```rust
fn example() {
    let data: Vec<i32> = vec![1, 2, 3];
    //   ^^^^  ^^^^^^^    ← RA colors these (semantic tokens)
    //                                        [Owned]  ← BorrowScope hint (italic, gray)
    //
    // ┃ (blue background, 8% opacity)        ← BorrowScope borrow scope
    let r = &data;
    // ┃
    println!("{}", r);
    // ┃ (scope ends here)
}
```

**Expectation:** Both extensions' visual elements are simultaneously visible without clashing. RA's syntax highlighting is fully preserved.

**Tests for 9.5:**
- BorrowScope decorations don't override text color
- Borrow scope background is visible but doesn't obscure code
- Inlay hints are visually distinct from RA's type annotations
- Disabling BorrowScope restores editor to RA-only appearance
- No flickering when both extensions update simultaneously

---

## 9.6 Resource Sharing Strategy

**Objective:** Define the resource budget when both rust-analyzer and borrowscope-lsp run simultaneously. Ensure the combined memory and CPU usage is acceptable on typical developer machines (16GB RAM, 4-8 cores).

**Resource budget:**
```
Component                    │ RAM Budget  │ CPU (idle) │ CPU (active)
─────────────────────────────┼─────────────┼────────────┼─────────────
rust-analyzer                │ 500MB-1.5GB │ < 1%       │ < 30%
borrowscope-lsp              │ 500MB-1GB   │ < 1%       │ < 20%
VS Code (extension host)     │ 200MB       │ < 1%       │ < 5%
─────────────────────────────┼─────────────┼────────────┼─────────────
Total                        │ 1.2-2.7GB   │ < 3%       │ < 55%
```

**Mitigation for constrained machines:**
1. Setting: `borrowscope.server.memoryLimit` (default: 1024MB)
2. Server monitors its own RSS and evicts caches when approaching limit
3. If system memory is low (< 4GB free), show warning and suggest disabling features
4. Lazy loading (Strategy C from 9.1) reduces initial memory footprint

**Code (server - memory monitor):**
```rust
fn check_memory_usage() -> usize {
    // Read /proc/self/status on Linux, task_info on macOS
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
        // Parse VmRSS line
        status.lines()
            .find(|l| l.starts_with("VmRSS:"))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0) * 1024 // Convert KB to bytes
    }
}

fn maybe_evict_caches(state: &mut GlobalState, limit_bytes: usize) {
    let current = check_memory_usage();
    if current > limit_bytes {
        tracing::warn!("Memory usage {}MB exceeds limit {}MB, evicting caches",
            current / (1024*1024), limit_bytes / (1024*1024));
        state.analysis_cache.evict_closed_files();
    }
}
```

**Expectation:** The combined system runs comfortably on a 16GB machine. On 8GB machines, a warning is shown but the extension still functions (with reduced caching).

**Tests for 9.6:**
- Server memory stays under configured limit
- Cache eviction triggers when limit is approached
- Both servers running simultaneously doesn't cause OOM
- Memory monitor reports correct RSS values
- Warning shown when system memory is low
- Extension degrades gracefully (fewer features) under memory pressure

---

## 9.T Integration Test Suite

```rust
#[test]
fn test_both_servers_coexist() {
    // Start both rust-analyzer and borrowscope-lsp for the same workspace
    let ra = start_rust_analyzer(fixture_path());
    let bs = start_borrowscope_lsp(fixture_path());

    // Both complete initialization
    ra.wait_ready();
    bs.wait_ready();

    // Both respond to requests
    let ra_hover = ra.hover("src/main.rs", 5, 8);
    let bs_hover = bs.hover("src/main.rs", 5, 8);

    assert!(ra_hover.contains("Vec<i32>")); // RA shows type
    assert!(bs_hover.contains("BorrowScope")); // BS shows ownership

    // Memory check
    let total_mem = ra.memory_usage() + bs.memory_usage();
    assert!(total_mem < 3 * 1024 * 1024 * 1024); // < 3GB combined
}

#[test]
fn test_diagnostics_dont_duplicate() {
    let bs = start_borrowscope_lsp(fixture_with_conflict());
    bs.wait_ready();
    bs.open_file("src/main.rs");

    let diagnostics = bs.wait_for_diagnostics("src/main.rs");

    // All BorrowScope diagnostics are Information (not Error)
    for d in &diagnostics {
        assert_eq!(d.severity, DiagnosticSeverity::INFORMATION);
        assert_eq!(d.source, Some("BorrowScope".into()));
    }
}

#[test]
fn test_graph_follows_navigation() {
    let bs = start_borrowscope_lsp(fixture_path());
    bs.wait_ready();
    bs.open_file("src/main.rs");

    // Cursor in function A
    let graph_a = bs.request_ownership_graph("src/main.rs", 5, 0);
    assert_eq!(graph_a.function_name, "function_a");

    // Move cursor to function B
    let graph_b = bs.request_ownership_graph("src/main.rs", 25, 0);
    assert_eq!(graph_b.function_name, "function_b");
}
```
