## 8. Limitations & Current Capabilities

### ✅ Completed Capabilities (Previously Future Work)

The following items from the original roadmap have been fully implemented:

**Method Call Tracking**: All method calls on tracked variables are now recorded with:
- Semantic operation paths via `sema.resolve_method_call()`
- Self-borrow type detection (immutable, mutable, consuming)
- Receiver and result type information

**Standalone Expression Tracking**: Function calls like `drop()`, `thread::spawn()`, `transmute()` are tracked via semantic `FunctionId` comparison (zero string heuristics).

**Self-Borrow Inference**: The `resolve_self_borrow()` function uses rust-analyzer's `Access` enum to determine whether methods take `&self`, `&mut self`, or `self`.

**100% Semantic Coverage**: All 109+ ownership patterns are now classified through rust-analyzer's type system. The macro no longer requires any heuristic pattern matching.

### Current Limitations

**Pattern Binding Decomposition**: Destructuring patterns like `let (a, b, c) = tuple;` are recorded as a single entry. Individual components can be looked up via tuple element indexing, but this adds complexity.

**Line Number Drift**: Type information contains absolute line numbers. Source modifications between analysis and compilation may cause lookup failures. The macro falls back gracefully but loses semantic precision.

**Macro-Generated Code**: Variables created by other procedural macros are not visible because the analyzer runs before macro expansion.

**Workspace-External Files**: Files outside the Cargo workspace receive only syntax-based analysis.

**Generic Type Parameters**: Generic parameters like `T` in `fn foo<T>(x: T)` cannot be resolved to concrete types.

**Closure Capture Details**: Closures are identified with full capture mode analysis (by-ref, by-move, unique-shared-ref) via `closure_hir.captured_items(db)`.

### Potential Future Work

**Incremental Analysis**: Caching rust-analyzer database between runs would improve performance.

**IDE Extension**: Direct integration with rust-analyzer LSP could eliminate the separate analysis step.

**User-Defined Type Classification**: Configuration file for custom smart pointer types:
```toml
# .borrowscope/config.toml
[classifications]
"MySmartPtr<" = "smart_pointer"
```

**Cross-Crate Analysis**: Deep analysis of dependency types for ownership semantics.

**Watch Mode**: File-watching with incremental re-analysis for development workflows.

---
