# borrowscope-graph: Publication Plan

## Overview

Five technical whitepapers documenting the design, implementation, and evaluation of `borrowscope-graph`. Each paper is self-contained but references the others. Published in sequence as development milestones complete.

All papers published at: `https://mehmet-ylcnky.github.io/BorrowScope/`

---

## Paper 1: Ownership Graph Construction from Runtime Event Streams

### Scope
Milestones 1 (Core Data Structures) + 7 (Runtime Integration)

### Abstract
Presents a graph construction system that transforms flat ownership event streams into structured, queryable graphs. Defines a type system of 11 edge kinds covering all Rust ownership relationships, and demonstrates both batch and streaming construction modes with feature-gated integration.

### Sections

1. **Introduction**
   - The gap between event streams and structural understanding
   - Why a flat event list is insufficient for ownership reasoning
   - Contributions: node/edge type system, construction algorithm, streaming protocol

2. **Background**
   - BorrowScope runtime event architecture (88 event types, 14 categories)
   - Existing graph.rs limitations (only 3 edge types, no smart pointer support)
   - Requirements for a complete ownership graph

3. **Node Type System**
   - VariableNode: ownership-relevant metadata (name, type, lifetime, mutability)
   - ScopeNode: containment hierarchy (function, block, loop)
   - NodeId design: newtype usize, Copy semantics, O(1) lookup
   - Why two node types are sufficient (variables are the primary entity)

4. **Edge Type System**
   - 11 EdgeKind variants and their semantic meaning
   - Temporal edges: start/end timestamps for lifetime-aware queries
   - Mapping from Rust ownership concepts to edge kinds
   - CaptureMode for closure edges (ByRef, ByMutRef, ByMove)

5. **Graph Construction Algorithm**
   - Event-to-graph mapping: which events create nodes, which create edges
   - Active borrow tracking: HashMap for edge termination on drop
   - Handling shadowed variables (name + function + decl_index disambiguation)
   - Complexity analysis: O(n) single-pass construction

6. **Streaming Construction**
   - GraphStream: incremental event processing
   - Callback-based updates for real-time visualization
   - Equivalence proof: streaming produces same graph as batch
   - Delta export for efficient client synchronization

7. **Runtime Integration**
   - Feature-gated design: zero cost when disabled
   - Re-export API: single import for tracking + graph
   - Feature matrix: track, graph, track+graph

8. **Evaluation**
   - Construction performance: 1K/10K/100K events benchmarks
   - Memory usage: bytes per node, bytes per edge
   - Battle test: ripgrep event stream produces valid graph
   - Streaming vs batch: latency and throughput comparison

9. **Conclusion**

### Key Diagrams
- Event-to-graph mapping table (event type -> node/edge operation)
- Architecture diagram: runtime -> events -> graph -> queries
- Performance chart: construction time vs event count

### Prerequisites
- Milestone 1 complete (all data structures implemented and tested)
- Milestone 7 complete (runtime integration working)
- Battle test event streams available as fixtures

---

## Paper 2: Graph Algorithms for Ownership Analysis

### Scope
Milestones 2 (Traversal) + 3 (Conflict Detection)

### Abstract
Adapts classical graph algorithms to the domain of Rust ownership analysis. Presents traversal algorithms (DFS, BFS, topological sort) with ownership-specific semantics, and introduces conflict detection algorithms that encode Rust's borrowing rules as graph invariants.

### Sections

1. **Introduction**
   - From ownership graphs to actionable insights
   - Classical algorithms need adaptation for temporal, directed ownership graphs
   - Contributions: ownership-aware traversal, conflict detection, cycle detection, validation

2. **Traversal Algorithms**
   - DFS with direction control (outgoing, incoming, both)
   - BFS with distance tracking
   - Shortest path: "how is variable X related to Y?"
   - Early termination for targeted queries

3. **Topological Ordering and Drop Order**
   - Kahn's algorithm adapted for ownership constraints
   - Borrow edges as ordering constraints (borrower before owner)
   - Handling cycles: CycleError with participant identification
   - Verification against Rust's actual drop semantics

4. **Reachability and Connected Components**
   - Reachability queries for ownership chain verification
   - Union-find for efficient component detection
   - Ownership clusters: independent variable groups
   - Borrow chain and depth computation

5. **Borrow Conflict Detection**
   - Interval overlap algorithm (sweep-line)
   - ConflictKind: MutableAndShared, MultipleMutable
   - Conflict timeline generation for visualization
   - Relationship to Rust's borrow checker rules

6. **Reference Cycle Detection**
   - DFS coloring (white/gray/black) for back-edge detection
   - Extracting cycle participants from DFS stack
   - Distinguishing Rc cycles (memory leaks) from Arc cycles (deadlock risk)

7. **Graph Validation**
   - Encoding ownership invariants as checkable properties
   - BorrowOutlivesOwner, MoveWhileBorrowed, DanglingEdgeReference
   - Use-after-move detection
   - Dangling pointer detection for unsafe code

8. **Evaluation**
   - Algorithm correctness: property-based tests on random graphs
   - Performance: traversal and conflict detection benchmarks
   - Real-world: conflicts detected in battle test graphs
   - False positive analysis: valid patterns not flagged

9. **Conclusion**

### Key Diagrams
- Conflict detection sweep-line visualization
- DFS coloring for cycle detection (step-by-step)
- Topological order example with borrow constraints
- Validation error taxonomy

### Prerequisites
- Paper 1 published (graph construction is the foundation)
- Milestones 2 and 3 complete with full test coverage
- Property-based tests passing (invariant verification)

---

## Paper 3: Temporal Ownership Analysis and Lifetime Visualization

### Scope
Milestones 4 (Temporal Queries) + 5 (Statistics)

### Abstract
Introduces temporal query capabilities that treat the ownership graph as a time-varying structure. Presents lifetime span computation, NLL-aware borrow scopes, ownership transfer timelines, and reference count history tracking. Complements temporal analysis with statistical metrics for identifying ownership complexity hotspots.

### Sections

1. **Introduction**
   - Ownership is inherently temporal: borrows start and end, values move
   - Static graphs lose temporal information; temporal graphs preserve it
   - Contributions: lifetime analysis, ownership timelines, ref-count history, hotspot detection

2. **Lifetime Span Computation**
   - LifetimeSpan: start, end, duration, is_alive_at
   - Open-ended lifetimes (never dropped)
   - Overlap detection via sweep-line
   - Contemporaries: all variables alive during a given variable's lifetime

3. **Ownership Snapshots**
   - "Time travel" debugging: inspect state at any timestamp
   - OwnershipSnapshot: alive variables, active borrows, active locks
   - Range queries: variables alive throughout an interval
   - Snapshot diff: what changed between two timestamps

4. **NLL-Aware Borrow Scopes**
   - Lexical vs non-lexical lifetimes
   - Effective scope: last usage, not drop point
   - Integration with analyzer's usage data
   - Visualization: borrow scope highlighting in source

5. **Ownership Transfer Timelines**
   - Move chain reconstruction (origin -> ... -> current owner)
   - find_origin and find_current_owner queries
   - Timeline visualization for value lifecycle
   - Multi-move patterns in real code

6. **Reference Count History**
   - Tracking Rc/Arc counts over time
   - Clone increments, drop decrements
   - Peak count and leak detection (final count > 0)
   - Visualization: ref-count time series chart

7. **Statistical Analysis**
   - GraphStatistics: comprehensive counts and derived metrics
   - Hotspot detection: variables with disproportionate edge counts
   - Borrow frequency analysis with burst detection
   - Smart pointer usage patterns (Rc families, RefCell ratios, Mutex contention)

8. **Evaluation**
   - Temporal query performance on large graphs
   - Leak detection accuracy on known-leaky programs
   - Hotspot identification on battle test codebases
   - Comparison: temporal graph vs flat event stream for debugging

9. **Conclusion**

### Key Diagrams
- Lifetime span visualization (Gantt-chart style)
- Ownership transfer timeline (value moving between variables)
- Reference count time series (clone/drop events)
- Hotspot heatmap (variables colored by edge count)

### Prerequisites
- Papers 1-2 published (graph construction and algorithms)
- Milestones 4 and 5 complete
- Benchmark suite running with performance targets met

---

## Paper 4: Static-Dynamic Ownership Graph Fusion

### Scope
Milestone 8 (Analyzer Integration)

### Abstract
Presents a technique for combining static analysis data (from borrowscope-analyzer) with runtime observation (from borrowscope-runtime) into a unified ownership graph. Introduces static graph construction without program execution, node enrichment with type metadata, and scope hierarchy reconstruction. Demonstrates that the fusion of static and dynamic analysis produces richer insights than either alone.

### Sections

1. **Introduction**
   - Static analysis knows what is possible; dynamic analysis knows what happened
   - Neither alone is complete: static over-approximates, dynamic under-approximates
   - Contributions: static graph mode, enrichment protocol, scope hierarchy, source mapping

2. **Background: The Analyzer's Output**
   - Schema v3.0: 105 fields per variable, 22 top-level maps
   - Type classification, trait detection, method call analysis
   - Relationship to Papers 1-3 (runtime graph construction)

3. **Static Graph Construction**
   - Building ownership graphs from type-info.json alone
   - Inferring edges from initializer_kind (rc_clone -> RcClone edge)
   - Inferring borrows from method_calls (self_borrow: "mutable" -> BorrowMut edge)
   - Closure captures as edges (from closure_captures array)
   - Limitations: static graph shows potential, not actual execution

4. **Node Enrichment**
   - Matching runtime nodes to analyzer entries (function + name + decl_index)
   - Attaching type flags: is_copy, is_smart_pointer, is_interior_mutable
   - Attaching trait information: Send, Sync, Clone, Drop
   - Impact on statistics and visualization (smarter grouping)

5. **Scope Hierarchy Reconstruction**
   - Building containment tree from function_name and scope_id
   - ScopeContains edges for nesting visualization
   - Source-location-based ordering when scope_id is flat
   - Visualization: nested boxes in DOT export

6. **Source Location Mapping**
   - Attaching file/line/column to graph nodes
   - node_at_location: click-to-source for IDE integration
   - Drop location from analyzer (precise without runtime Drop event)
   - Source-level lifetime: declaration line to drop line

7. **Static vs Dynamic Comparison**
   - Static graph: all potential relationships (over-approximation)
   - Dynamic graph: only observed relationships (under-approximation)
   - Fusion: dynamic graph enriched with static metadata
   - Use cases: static for code review, dynamic for debugging, fusion for complete picture

8. **Evaluation**
   - Enrichment coverage: % of runtime nodes matched to analyzer entries
   - Static graph accuracy: edges that match runtime observations
   - Source mapping precision: correct file/line for all nodes
   - Performance: enrichment overhead on large graphs

9. **Conclusion**

### Key Diagrams
- Static vs dynamic graph comparison (same code, different edges)
- Enrichment flow: analyzer JSON -> graph nodes gain metadata
- Scope hierarchy tree visualization
- Venn diagram: static-only edges, dynamic-only edges, both

### Prerequisites
- Papers 1-3 published (full runtime graph pipeline)
- Milestone 8 complete
- Analyzer producing v3.0 schema (already done)
- Enrichment tested on battle test codebases

---

## Paper 5: Multi-Format Graph Export for Ownership Visualization

### Scope
Milestone 6 (Serialization and Export)

### Abstract
Presents a multi-format export system for ownership graphs, enabling integration with diverse visualization tools. Covers JSON (full and compact), Graphviz DOT, MessagePack, D3.js force-directed format, and a delta protocol for real-time streaming. Evaluates format trade-offs in size, speed, and tool compatibility.

### Sections

1. **Introduction**
   - Ownership graphs are only useful if they can be visualized and shared
   - Different tools require different formats (Graphviz, D3.js, custom UIs)
   - Contributions: 5 export formats, delta protocol, import with validation

2. **JSON Export**
   - Full format: all metadata, human-readable, pretty-printed
   - Compact format: abbreviated keys, omitted defaults, 40-60% smaller
   - Schema design: self-describing with version header
   - Import with validation and version checking

3. **Graphviz DOT Export**
   - Mapping ownership concepts to DOT visual elements
   - Node shapes by type (box = variable, ellipse = scope)
   - Edge colors by kind (blue = shared borrow, red = mutable, green = clone)
   - Configurable options: types, timestamps, direction, color scheme
   - Subgraphs for scope containment

4. **MessagePack Binary Format**
   - 30-50% smaller than JSON, 2x faster deserialization
   - Version header for forward compatibility
   - Use cases: large graph storage, network transmission, caching

5. **D3.js Force-Directed Format**
   - Nodes with id, name, group (for color coding), size (by edge count)
   - Links with source, target, value (for edge length), kind, color
   - Direct consumption by d3.forceSimulation()
   - Group assignment strategy for meaningful clustering

6. **Delta Export Protocol**
   - Tracking mutations since last export
   - GraphDelta: added nodes, added edges, dropped nodes, ended edges
   - Sequence numbers for ordering and gap detection
   - apply_delta for client-side graph reconstruction
   - Use case: real-time streaming to web visualization

7. **Format Comparison**
   - Size comparison table (same graph in all formats)
   - Parse/serialize speed benchmarks
   - Tool compatibility matrix
   - When to use which format

8. **Evaluation**
   - Round-trip correctness: export -> import produces identical graph
   - Size reduction: compact JSON vs full, MessagePack vs JSON
   - DOT rendering: valid output for graphs up to 500 nodes
   - Delta efficiency: update size vs full export for incremental changes
   - D3.js integration: renders correctly in browser

9. **Conclusion**

### Key Diagrams
- Format size comparison bar chart
- DOT rendering example (small ownership graph as SVG)
- Delta protocol sequence diagram (server -> client updates)
- D3.js force-directed layout screenshot

### Prerequisites
- Papers 1-4 published (complete graph pipeline)
- Milestone 6 complete with all formats implemented
- D3.js demo page working (for screenshots)
- Benchmark results collected

---

## Publication Timeline

| Paper | Title | Publish After | Estimated Date |
|-------|-------|---------------|----------------|
| 1 | Ownership Graph Construction | Milestones 1 + 7 complete | TBD |
| 2 | Graph Algorithms for Ownership Analysis | Milestones 2 + 3 complete | TBD |
| 3 | Temporal Analysis and Lifetime Visualization | Milestones 4 + 5 complete | TBD |
| 4 | Static-Dynamic Ownership Graph Fusion | Milestone 8 complete | TBD |
| 5 | Multi-Format Graph Export | Milestone 6 complete | TBD |

---

## Cross-References

Each paper references the others and the existing BorrowScope publications:

| From | References |
|------|-----------|
| Paper 1 | Previous whitepaper (analyzer), "Eliminating Heuristics" paper |
| Paper 2 | Paper 1 (graph construction), Rust Reference (borrow rules) |
| Paper 3 | Papers 1-2 (graph + algorithms), NLL RFC |
| Paper 4 | Papers 1-3 (runtime pipeline), analyzer whitepaper, PR #21835 |
| Paper 5 | Papers 1-4 (complete system), D3.js docs, Graphviz docs |

---

## Format and Hosting

- Same HTML/CSS/JS format as existing papers
- Published on gh-pages branch under `/borrowscope-graph-paper-{N}/`
- Listed on main landing page under `borrowscope-graph` section
- Each paper: 8-10 sections, code examples, diagrams, evaluation tables
