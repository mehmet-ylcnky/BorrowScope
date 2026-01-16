## 7. Performance Characteristics

The analyzer's performance is dominated by workspace loading rather than the actual type extraction. Understanding this breakdown helps set appropriate expectations and identify optimization opportunities.

### Timing Breakdown

Analysis of a small project (single file, ~100 variables) shows the following timing distribution:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      PERFORMANCE BREAKDOWN                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Total Time: ~45-50 seconds                                                 │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│    │
│  │            Workspace Loading (~32s, 65%)                           │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────┐                                    │
│  │░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│                                    │
│  │    Type Analysis (~12s, 25%)       │                                    │
│  └─────────────────────────────────────┘                                    │
│  ┌──────────────┐                                                           │
│  │░░░░░░░░░░░░░░│                                                           │
│  │ Compile (~5s)│                                                           │
│  └──────────────┘                                                           │
│                                                                             │
│  Workspace Loading includes:                                                │
│    • Sysroot discovery (rustc --print sysroot)                              │
│    • Standard library metadata loading                                      │
│    • Cargo.toml parsing and dependency resolution                           │
│    • Building the semantic database                                         │
│                                                                             │
│  Type Analysis includes:                                                    │
│    • Parsing source files                                                   │
│    • Walking syntax trees                                                   │
│    • Resolving types via Semantics API                                      │
│    • JSON serialization                                                     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

The workspace loading phase is expensive because rust-analyzer must build a complete semantic model of the project and its dependencies. This includes loading metadata for the entire standard library, which contains thousands of types and trait implementations.

### Scaling Characteristics

Analysis time scales primarily with project complexity rather than line count:

| Project Size | Dependencies | Variables | Load Time | Analysis Time | Total |
|--------------|--------------|-----------|-----------|---------------|-------|
| Small (1 file) | std only | ~100 | ~32s | ~12s | ~45s |
| Medium (10 files) | 5-10 crates | ~500 | ~35s | ~15s | ~50s |
| Large (100 files) | 50+ crates | ~2000 | ~60s | ~30s | ~90s |

The workspace loading time increases modestly with dependency count because rust-analyzer must resolve and load metadata for each crate. The analysis time scales roughly linearly with the number of variables.

### Optimization Opportunities

Several strategies can improve performance for different use cases:

**Incremental Analysis**: The current implementation performs full analysis on every run. A future version could cache the loaded workspace and only re-analyze changed files. rust-analyzer's internal architecture supports incremental updates, but exposing this through the `ra_ap_*` crates requires additional implementation work.

**Parallel File Processing**: Type extraction for different files is independent and could be parallelized. The current implementation processes files sequentially, but the `Semantics` API is thread-safe for read operations.

**Selective Analysis**: For large projects, analyzing only files containing `#[trace_borrow]` annotations would reduce work. This requires a two-pass approach: first scan for annotations, then analyze only relevant files.

**Workspace Caching**: The loaded `RootDatabase` could theoretically be serialized and reused across runs. However, rust-analyzer's database structures are not designed for serialization, making this approach impractical without significant upstream changes.

### When to Run the Analyzer

Given the analysis cost, consider these usage patterns:

**Development**: Run the analyzer once when starting work on a feature, then re-run only when adding new variables or changing types significantly. Minor edits that don't affect types don't require re-analysis.

**CI/CD**: Include analysis in the CI pipeline to ensure type information is always current for release builds. The ~1 minute overhead is acceptable for CI but may be too slow for rapid local iteration.

**IDE Integration**: A future rust-analyzer extension could provide type information directly to the macro, eliminating the need for a separate analysis step. This would leverage rust-analyzer's existing incremental analysis and caching.

### Memory Usage

The analyzer's memory footprint is dominated by rust-analyzer's semantic database:

| Phase | Memory Usage |
|-------|--------------|
| Startup | ~50 MB |
| After workspace load | ~500-800 MB |
| During analysis | ~600-900 MB |
| Peak (large projects) | ~1-2 GB |

The memory usage reflects rust-analyzer's design for IDE responsiveness—it caches extensively to enable fast queries. For the analyzer's batch processing use case, this caching is less beneficial but unavoidable given the current API design.

---

