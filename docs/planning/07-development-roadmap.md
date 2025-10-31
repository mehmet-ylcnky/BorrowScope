# 7. Development Roadmap

## Project Timeline

**Start Date:** November 1, 2025  
**Target Release:** February 15, 2026  
**Duration:** 14 weeks (~3.5 months)

---

## Milestone 1: MVP - Macro + Runtime (Weeks 1-3)

**Dates:** Nov 1 - Nov 21, 2025

### Week 1: Project Setup & Basic Macro
**Deliverables:**
- Cargo workspace initialized
- `borrowscope-macro` crate created
- Basic `#[trace_borrow]` attribute parsing
- Simple AST visitor implemented

**Tasks:**
- [ ] Create workspace structure
- [ ] Set up CI/CD (GitHub Actions)
- [ ] Implement basic macro skeleton
- [ ] Parse function attributes with `syn`
- [ ] Write first unit test

**Success Criteria:**
- Macro compiles without errors
- Can parse annotated functions
- Basic test passes

---

### Week 2: Runtime Tracker
**Deliverables:**
- `borrowscope-runtime` crate created
- Event tracking API implemented
- Global tracker with thread safety
- JSON serialization working

**Tasks:**
- [ ] Define Event enum
- [ ] Implement `track_new()`, `track_borrow()`, `track_drop()`
- [ ] Create global tracker with `lazy_static` + `parking_lot`
- [ ] Add `serde` serialization
- [ ] Write runtime unit tests

**Success Criteria:**
- Can track basic ownership events
- Export to valid JSON
- Thread-safe operation verified

---

### Week 3: Integration & Testing
**Deliverables:**
- Macro generates correct tracking calls
- End-to-end test cases working
- Documentation for Phase 1

**Tasks:**
- [ ] Implement AST transformation in macro
- [ ] Generate tracking calls with `quote!`
- [ ] Create 5+ test cases (simple to complex)
- [ ] Write integration tests
- [ ] Document Phase 1 API

**Success Criteria:**
```rust
#[trace_borrow]
fn test() {
    let s = String::from("hello");
    let r = &s;
}
```
Produces valid JSON with all events tracked.

**Milestone 1 Completion:** ✅ Working macro + runtime + JSON export

---

## Milestone 2: Visualization UI (Weeks 4-6)

**Dates:** Nov 22 - Dec 12, 2025

### Week 4: UI Foundation
**Deliverables:**
- Tauri project initialized
- Basic window with file loading
- JSON parser in frontend

**Tasks:**
- [ ] Set up Tauri project structure
- [ ] Create main window layout
- [ ] Implement file picker
- [ ] Parse JSON in JavaScript
- [ ] Display raw data in UI

**Success Criteria:**
- Tauri app launches
- Can load and parse Phase 1 JSON
- Data displayed in console/debug view

---

### Week 5: Graph View
**Deliverables:**
- Interactive graph visualization
- Node/edge rendering with Cytoscape.js
- Basic interactions (hover, click)

**Tasks:**
- [ ] Integrate Cytoscape.js
- [ ] Map JSON data to graph nodes/edges
- [ ] Implement color coding (owner/borrow types)
- [ ] Add hover tooltips
- [ ] Add click highlighting

**Success Criteria:**
- Graph displays ownership relationships
- Nodes show variable names and types
- Edges show borrow relationships
- Interactive and responsive

---

### Week 6: Timeline View
**Deliverables:**
- Timeline visualization with D3.js
- Event playback controls
- Synchronized graph updates

**Tasks:**
- [ ] Build timeline component with D3.js
- [ ] Display variable lifespans
- [ ] Add playback controls (play/pause/step)
- [ ] Sync timeline with graph view
- [ ] Add animation for event progression

**Success Criteria:**
- Timeline shows variable creation → drop
- Can step through events
- Graph updates in sync with timeline
- Smooth animations

**Milestone 2 Completion:** ✅ Interactive visualization UI

---

## Milestone 3: CLI Integration (Weeks 7-8)

**Dates:** Dec 13 - Dec 26, 2025

### Week 7: Cargo Subcommand
**Deliverables:**
- `borrowscope-cli` crate created
- Basic subcommands implemented
- File instrumentation working

**Tasks:**
- [ ] Create CLI crate with `clap`
- [ ] Implement `visualize` subcommand
- [ ] Add automatic file instrumentation
- [ ] Handle temporary file creation/cleanup
- [ ] Integrate with Cargo build system

**Success Criteria:**
```bash
cargo borrowscope visualize src/main.rs
```
Instruments, compiles, runs, and generates JSON.

---

### Week 8: Workflow Automation
**Deliverables:**
- Full workflow automation
- Configuration file support
- UI auto-launch
- Error handling

**Tasks:**
- [ ] Implement `run` and `export` subcommands
- [ ] Add `.borrowscope.toml` config support
- [ ] Auto-launch UI with generated data
- [ ] Improve error messages
- [ ] Add progress indicators

**Success Criteria:**
- One-command workflow works end-to-end
- Config file customizes behavior
- UI opens automatically with data
- Helpful error messages on failure

**Milestone 3 Completion:** ✅ Seamless Cargo integration

---

## Milestone 4: Advanced Analysis (Weeks 9-12)

**Dates:** Dec 27, 2025 - Jan 23, 2026

### Week 9: MIR Analysis Research
**Deliverables:**
- Research spike on `rustc_middle`
- Proof of concept for MIR access
- Design document for compiler integration

**Tasks:**
- [ ] Study `rustc_driver` and `rustc_middle` APIs
- [ ] Create minimal compiler plugin
- [ ] Extract basic MIR data
- [ ] Document integration approach
- [ ] Evaluate feasibility

**Success Criteria:**
- Can access MIR for simple functions
- Understand borrow checker data structures
- Clear path forward documented

---

### Week 10: Lifetime Visualization
**Deliverables:**
- Lifetime extraction from MIR
- Extended data model with lifetimes
- UI updates for lifetime display

**Tasks:**
- [ ] Implement MIR visitor for lifetimes
- [ ] Extend Event/Graph models
- [ ] Update JSON schema
- [ ] Add lifetime visualization to UI
- [ ] Test with explicit lifetime annotations

**Success Criteria:**
- Can extract and visualize `'a`, `'b`, etc.
- UI shows which variables share lifetimes
- Works with complex lifetime scenarios

---

### Week 11: Advanced Patterns
**Deliverables:**
- Support for closures
- Support for async/await
- Smart pointer tracking

**Tasks:**
- [ ] Handle closure captures
- [ ] Track async ownership transfers
- [ ] Add Box/Rc/Arc/RefCell support
- [ ] Update macro for complex patterns
- [ ] Create advanced test cases

**Success Criteria:**
- Closures tracked correctly
- Async functions visualized
- Smart pointers shown in graph

---

### Week 12: Error Simulation Mode
**Deliverables:**
- Borrow checker error visualization
- Error explanation system
- Suggestion engine

**Tasks:**
- [ ] Detect borrow conflicts
- [ ] Generate error messages
- [ ] Visualize conflicts in UI
- [ ] Add "why this fails" explanations
- [ ] Suggest fixes

**Success Criteria:**
- Can simulate common borrow errors
- Visual explanation of conflicts
- Helpful suggestions provided

**Milestone 4 Completion:** ✅ Advanced compiler integration

---

## Milestone 5: Polish & Release (Weeks 13-14)

**Dates:** Jan 24 - Feb 7, 2026

### Week 13: Documentation & Examples
**Deliverables:**
- Comprehensive README
- API documentation
- Tutorial examples
- User guide

**Tasks:**
- [ ] Write detailed README
- [ ] Generate rustdoc for all crates
- [ ] Create 10+ example projects
- [ ] Write user guide with mdbook
- [ ] Record demo video

**Success Criteria:**
- Clear installation instructions
- API fully documented
- Examples cover beginner to advanced
- Video demonstrates key features

---

### Week 14: Release Preparation
**Deliverables:**
- Published to crates.io
- GitHub repository public
- CI/CD fully configured
- Announcement materials

**Tasks:**
- [ ] Final testing on all platforms
- [ ] Package for crates.io
- [ ] Publish all crates
- [ ] Set up GitHub releases
- [ ] Write announcement blog post
- [ ] Submit to This Week in Rust

**Success Criteria:**
- `cargo install borrowscope` works
- All tests pass on Linux/macOS/Windows
- Documentation live at docs.rs
- Public announcement made

**Milestone 5 Completion:** ✅ v1.0.0 Released

---

## Release Schedule

### v0.1.0 - MVP (Nov 21, 2025)
- Basic macro + runtime
- JSON export
- Internal testing only

### v0.2.0 - Alpha (Dec 12, 2025)
- Visualization UI
- Limited beta testing
- Feedback collection

### v0.3.0 - Beta (Dec 26, 2025)
- CLI integration
- Public beta testing
- Bug fixes

### v0.4.0 - RC1 (Jan 23, 2026)
- Advanced features
- Feature complete
- Final testing

### v1.0.0 - Public Release (Feb 7, 2026)
- Production ready
- Full documentation
- Public announcement

---

## Success Metrics

### Technical Metrics
- [ ] Supports 95% of common Rust patterns
- [ ] Processes typical function in <1 second
- [ ] Binary size <10MB (Tauri app)
- [ ] Zero false positives in tracking
- [ ] Works with Rust 1.75+

### User Metrics
- [ ] 100+ GitHub stars in first month
- [ ] 1000+ downloads from crates.io
- [ ] 10+ community contributions
- [ ] Featured in This Week in Rust
- [ ] Positive feedback from beta testers

### Quality Metrics
- [ ] 80%+ code coverage
- [ ] All clippy warnings resolved
- [ ] Zero critical security issues
- [ ] Documentation completeness >90%
- [ ] CI passing on all platforms

---

## Risk Management

### High Risk Items

| Risk | Impact | Mitigation | Owner |
|------|--------|------------|-------|
| MIR API instability | High | Use stable `syn` fallback | Dev Team |
| Performance issues | Medium | Optimize incrementally, add sampling | Dev Team |
| UI complexity | Medium | Start with simple HTML+D3 | UI Dev |
| Timeline slip | Medium | Ship Phase 3 as v1.0 if needed | PM |

### Contingency Plans

**If Week 9-12 blocked:**
- Ship v1.0 without MIR analysis
- Release Phase 4 as v2.0 later
- Focus on polish and UX

**If UI development slow:**
- Use static HTML + D3.js instead of Tauri
- Simplify to graph-only view
- Add timeline in v1.1

**If timeline slips 2+ weeks:**
- Cut Phase 4 features
- Release MVP as v1.0
- Iterate based on feedback

---

## Post-Release Roadmap (v1.1+)

### v1.1 (March 2026)
- VS Code extension
- Real-time WebSocket streaming
- Performance optimizations

### v1.2 (April 2026)
- IDE integration (IntelliJ, Vim)
- Export to PNG/SVG
- Collaborative features

### v2.0 (Q3 2026)
- Full MIR integration
- LLVM IR analysis
- Educational mode with tutorials

---

## Team & Resources

### Roles Needed
- **Lead Developer:** Macro + runtime implementation
- **UI Developer:** Tauri + visualization
- **DevOps:** CI/CD, packaging, release
- **Technical Writer:** Documentation, examples

### Time Commitment
- **Full-time:** 14 weeks
- **Part-time (50%):** 28 weeks
- **Part-time (25%):** 56 weeks

### Budget Considerations
- Development: Time-based
- Infrastructure: GitHub Actions (free for open source)
- Domain/hosting: Optional, ~$20/year
- Marketing: Community-driven (free)

---

## Review Checkpoints

### Weekly Reviews
- Every Friday: Progress review
- Blockers identified and addressed
- Next week priorities set

### Phase Reviews
- End of each milestone
- Demo to stakeholders
- Go/no-go decision for next phase

### Monthly Reviews
- Overall progress assessment
- Timeline adjustments if needed
- Risk review and mitigation updates
