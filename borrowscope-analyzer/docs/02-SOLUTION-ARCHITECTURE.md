## 2. Solution Architecture

The borrowscope-analyzer implements a two-phase compilation strategy that decouples type analysis from macro expansion. By running semantic analysis as a pre-build step, we extract type information into a structured format that the procedural macro can consume during its execution. This approach works within Rust's compilation model rather than against it.

### Two-Phase Build Strategy

The solution introduces an explicit analysis phase before the standard Cargo build:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      TWO-PHASE BUILD STRATEGY                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  PHASE 1: STATIC ANALYSIS (borrowscope-analyzer)                            │
│  ════════════════════════════════════════════════                           │
│                                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                   │
│  │  Cargo.toml  │───▶│rust-analyzer │───▶│ type-info.json│                  │
│  │  src/*.rs    │    │   engine     │    │              │                   │
│  └──────────────┘    └──────────────┘    └──────────────┘                   │
│                                                                             │
│        User's                Full semantic          Extracted type          │
│        project               analysis with          metadata for            │
│        source                type resolution        all variables           │
│                                                                             │
│                                    │                                        │
│                                    ▼                                        │
│  PHASE 2: INSTRUMENTED BUILD (cargo build)                                  │
│  ═════════════════════════════════════════                                  │
│                                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                   │
│  │  src/*.rs    │───▶│#[trace_borrow]───▶│ Instrumented │                   │
│  │              │    │    macro     │    │    binary    │                   │
│  └──────────────┘    └──────────────┘    └──────────────┘                   │
│                             │                                               │
│                             │ reads                                         │
│                             ▼                                               │
│                      ┌──────────────┐                                       │
│                      │type-info.json│                                       │
│                      └──────────────┘                                       │
│                                                                             │
│        Macro now has complete type information for accurate tracking        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

This architecture preserves the standard Rust compilation model while augmenting it with pre-computed type information. The analyzer runs independently of `rustc`, using the same semantic analysis engine that powers rust-analyzer IDE features.

### Component Overview

The solution consists of three interconnected components:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         COMPONENT ARCHITECTURE                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                     borrowscope-analyzer                            │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │    │
│  │  │   main.rs   │  │ analysis.rs │  │  output.rs  │                  │    │
│  │  │             │  │             │  │             │                  │    │
│  │  │ CLI entry   │  │ Semantic    │  │ JSON        │                  │    │
│  │  │ point       │  │ analysis    │  │ serialization│                 │    │
│  │  └─────────────┘  └─────────────┘  └─────────────┘                  │    │
│  │         │                │                │                         │    │
│  │         └────────────────┴────────────────┘                         │    │
│  │                          │                                          │    │
│  │                          ▼                                          │    │
│  │              ┌───────────────────────┐                              │    │
│  │              │  ra_ap_* crates       │                              │    │
│  │              │  (rust-analyzer libs) │                              │    │
│  │              └───────────────────────┘                              │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                          │                                                  │
│                          │ writes                                           │
│                          ▼                                                  │
│                ┌───────────────────────┐                                    │
│                │  .borrowscope/        │                                    │
│                │    type-info.json     │                                    │
│                └───────────────────────┘                                    │
│                          │                                                  │
│                          │ reads                                            │
│                          ▼                                                  │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                     borrowscope-macro                               │    │
│  │                                                                     │    │
│  │  #[trace_borrow] ──▶ lookup type by file:line:col ──▶ emit correct │    │
│  │                      from type-info.json              tracking call │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**borrowscope-analyzer** is a standalone binary that loads a Rust project using rust-analyzer's workspace loading infrastructure. It performs full semantic analysis, extracting type information for every variable binding, and writes the results to a JSON file in the project's `.borrowscope/` directory.

**type-info.json** serves as the bridge between static analysis and macro expansion. It contains a structured representation of every variable's type, including classification flags for smart pointers, interior mutability types, and Copy semantics. The file is keyed by source location (file path, line, column) enabling precise lookup during macro expansion.

**borrowscope-macro** (to be enhanced) will read the type information file at macro expansion time. When transforming a `let` binding, it looks up the variable's location in the JSON file to retrieve complete type information, enabling accurate selection of tracking functions.

### Design Rationale

Several alternative approaches were considered before settling on this architecture:

**Compiler Plugin**: A rustc plugin could access type information directly during compilation. However, compiler plugins are unstable, require nightly Rust, and couple tightly to rustc internals that change frequently.

**Build Script Integration**: A `build.rs` script could theoretically perform analysis, but build scripts execute before compilation and cannot access the type-checked HIR. They also cannot easily invoke rust-analyzer's analysis infrastructure.

**Runtime Reflection**: Rust intentionally lacks runtime reflection. While `std::any::TypeId` exists, it cannot be used at compile time and provides only type identity, not structural information.

**Separate Analysis Tool**: The chosen approach uses rust-analyzer's published crates (`ra_ap_*`) which provide stable, well-maintained APIs for semantic analysis. These crates are the same ones powering the rust-analyzer IDE, ensuring correctness and compatibility with Rust's evolving type system.

The two-phase approach adds a build step but provides complete type information without requiring unstable features or compiler modifications. It integrates cleanly with existing Cargo workflows and can be automated through build scripts or CI pipelines.

---

