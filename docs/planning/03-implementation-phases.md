# 3. Implementation Phases

## Phase 1: MVP - Macro + Runtime Tracker (Weeks 1-3)

**Goal:** Prove the concept with a working procedural macro that tracks basic ownership events.

**Deliverables:**
- `borrowscope-macro` crate with `#[trace_borrow]` attribute
- `borrowscope-runtime` crate with event tracking
- JSON export of ownership events
- Simple test cases demonstrating tracking

**Features:**
- Track variable creation (`let x = ...`)
- Track immutable borrows (`&x`)
- Track mutable borrows (`&mut x`)
- Track moves (ownership transfer)
- Track drops (end of scope)
- Export events to `borrowscope-output.json`

**Success Criteria:**
```rust
#[trace_borrow]
fn example() {
    let s = String::from("hello");
    let r1 = &s;
    let r2 = &s;
    println!("{} {}", r1, r2);
}
```
Produces JSON output showing:
- `s` created
- `r1` borrows `s` (immutable)
- `r2` borrows `s` (immutable)
- `r1`, `r2` dropped
- `s` dropped

**Technical Tasks:**
1. Set up Cargo workspace with macro and runtime crates
2. Implement basic AST parsing with `syn`
3. Generate tracking calls with `quote!`
4. Create runtime tracker with event collection
5. Implement JSON serialization
6. Write unit tests for common patterns

---

## Phase 2: Visualization Layer (Weeks 4-6)

**Goal:** Build interactive UI to visualize the captured ownership data.

**Deliverables:**
- `borrowscope-ui` crate (Tauri or Dioxus)
- Graph view showing ownership relationships
- Timeline view showing variable lifecycles
- Load and display JSON from Phase 1

**Features:**
- **Graph View:**
  - Nodes represent variables
  - Edges represent borrow/ownership relationships
  - Color coding: owner (blue), immutable borrow (green), mutable borrow (red)
  - Interactive: hover for details, click to highlight

- **Timeline View:**
  - Horizontal axis = execution time
  - Each variable has a lifespan bar
  - Borrow periods shown as overlapping segments
  - Scrubber to step through events

- **Basic Controls:**
  - Load JSON file
  - Play/pause animation
  - Step forward/backward through events
  - Reset to initial state

**Success Criteria:**
- Load Phase 1 JSON output
- Display graph with correct relationships
- Animate timeline showing variable creation → borrows → drops
- Smooth, responsive UI

**Technical Tasks:**
1. Choose UI framework (Tauri recommended for desktop)
2. Set up project structure with web assets
3. Implement JSON parser and data model
4. Build graph renderer (D3.js or Cytoscape.js)
5. Build timeline component
6. Add event playback controls
7. Style UI for clarity and usability

---

## Phase 3: CLI Integration (Weeks 7-8)

**Goal:** Seamless Cargo integration for automatic instrumentation and visualization.

**Deliverables:**
- `borrowscope-cli` crate as Cargo subcommand
- Automatic code instrumentation without manual annotations
- One-command workflow from source to visualization

**Features:**
- `cargo borrowscope visualize <file>` - Instrument, run, and visualize single file
- `cargo borrowscope run` - Instrument and run entire project
- `cargo borrowscope export` - Generate JSON without opening UI
- Automatic cleanup of temporary files
- Configuration file support (`.borrowscope.toml`)

**Success Criteria:**
```bash
cd my-rust-project
cargo borrowscope visualize src/main.rs
# → Automatically instruments, runs, and opens visualization
```

**Technical Tasks:**
1. Create CLI crate with `clap` argument parsing
2. Implement file instrumentation (inject `#[trace_borrow]` automatically)
3. Integrate with Cargo build system
4. Launch UI with generated data
5. Add configuration file parsing
6. Handle errors gracefully with helpful messages
7. Package as installable Cargo subcommand

---

## Phase 4: Advanced Analysis (Weeks 9-12)

**Goal:** Deep compiler integration for accurate lifetime and borrow checker analysis.

**Deliverables:**
- MIR/HIR analysis for precise borrow tracking
- Lifetime visualization
- Borrow checker error simulation
- Support for complex patterns (closures, async, traits)

**Features:**
- **Compiler Integration:**
  - Hook into `rustc_middle` for MIR access
  - Extract actual borrow checker data
  - Visualize inferred lifetimes

- **Advanced Patterns:**
  - Closures capturing variables
  - Async/await ownership transfers
  - Trait objects and dynamic dispatch
  - Smart pointers (Box, Rc, Arc, RefCell)

- **Error Mode:**
  - Simulate borrow checker errors
  - Show why specific code fails
  - Suggest fixes with visual explanation

- **Performance:**
  - Optimize for large codebases
  - Incremental analysis
  - Filter/focus on specific functions

**Success Criteria:**
- Accurately visualize complex lifetime scenarios
- Show borrow checker errors with visual explanation
- Handle real-world Rust projects (1000+ lines)
- Performance: analyze typical function in <1 second

**Technical Tasks:**
1. Research `rustc_driver` and `rustc_middle` APIs
2. Implement MIR visitor to extract borrow data
3. Parse and visualize lifetime annotations
4. Build error simulation mode
5. Add support for closures and async
6. Optimize graph algorithms for performance
7. Add filtering and focus features to UI

---

## Phase 5: Polish & Release (Weeks 13-14)

**Goal:** Production-ready tool with documentation and community support.

**Deliverables:**
- Comprehensive documentation
- Tutorial and examples
- Published to crates.io
- GitHub repository with CI/CD
- Demo video and blog post

**Features:**
- User guide and API documentation
- Example projects demonstrating features
- Keyboard shortcuts and accessibility
- Export visualizations as images/videos
- Integration with VS Code (optional extension)

**Success Criteria:**
- Published on crates.io
- Documentation at docs.rs
- 5+ example projects
- CI passing on Linux, macOS, Windows
- Positive feedback from beta testers

**Technical Tasks:**
1. Write comprehensive README
2. Create rustdoc documentation
3. Build example projects (beginner to advanced)
4. Set up GitHub Actions CI/CD
5. Package for multiple platforms
6. Create demo video
7. Write announcement blog post
8. Submit to This Week in Rust

---

## Timeline Summary

| Phase | Duration | Key Milestone |
|-------|----------|---------------|
| Phase 1 | Weeks 1-3 | Working macro + JSON export |
| Phase 2 | Weeks 4-6 | Interactive visualization |
| Phase 3 | Weeks 7-8 | Cargo integration |
| Phase 4 | Weeks 9-12 | Advanced compiler analysis |
| Phase 5 | Weeks 13-14 | Public release |

**Total:** ~14 weeks (3.5 months)

## Risk Mitigation

- **Phase 1 blocked?** → Focus on runtime-only tracking without macros
- **Phase 2 UI complex?** → Start with static HTML + D3.js before Tauri
- **Phase 4 too ambitious?** → Ship Phase 3 as v1.0, make Phase 4 optional v2.0
- **Performance issues?** → Add sampling mode, limit graph size, optimize incrementally
