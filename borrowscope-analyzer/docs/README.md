# borrowscope-analyzer

> Static type analyzer for BorrowScope using rust-analyzer

## Overview

The borrowscope-analyzer extracts complete type information from Rust projects using rust-analyzer's semantic analysis engine. This enables the `#[trace_borrow]` macro to make accurate tracking decisions based on resolved types rather than syntactic pattern matching.

**Key Achievement: 100% Semantic Coverage** - All ownership-related operations are now classified through rust-analyzer's type system. Zero heuristic pattern matching required.

## Quick Start

```bash
# Analyze your project
cargo run -p borrowscope-analyzer -- /path/to/your/project

# Output written to: /path/to/your/project/.borrowscope/type-info.json
```

## What's Tracked

### Variables (Schema v2.5)
- Fully resolved types (including generics and type aliases)
- Trait implementations: Copy, Clone, Send, Sync, Drop, Future, Iterator
- Type classifications: smart pointers, interior mutability, guards, collections
- 78 semantic initializer categories

### Method Calls
All method calls on tracked variables with:
- Semantic operation paths (e.g., `core::cell::set`, `std::thread::join`)
- Self-borrow type: `immutable`, `mutable`, or `consuming`
- Receiver and result types

### Standalone Expressions
- Memory: `drop()`, `forget()`, `transmute()`, `replace()`, `swap()`, `take()`
- Threading: `thread::spawn()` with closure capture detection
- Pointers: `ptr::read()`, `ptr::write()`, `ptr::copy()`

## Output Example

```json
{
  "version": "2.5",
  "files": {
    "src/main.rs": [{
      "name": "cell",
      "ty": "Cell<i32>",
      "is_cell": true,
      "initializer_kind": "cell_new",
      "method_calls": [{
        "method": "set",
        "operation": "core::cell::set",
        "self_borrow": "immutable"
      }]
    }]
  },
  "expressions": {
    "src/main.rs": [{
      "operation": "core::mem::drop",
      "argument": "x"
    }]
  }
}
```

## How It Works

```
Phase 1: Static Analysis (borrowscope-analyzer)
┌──────────┐    ┌──────────────┐    ┌─────────────────┐
│ src/*.rs │───▶│rust-analyzer │───▶│ type-info.json  │
└──────────┘    └──────────────┘    └─────────────────┘

Phase 2: Instrumented Build (cargo build)
┌──────────┐    ┌──────────────┐    ┌─────────────────┐
│ src/*.rs │───▶│#[trace_borrow]───▶│ Instrumented    │
└──────────┘    │ reads JSON   │    │ binary          │
               └──────────────┘    └─────────────────┘
```

## Coverage Summary

| Category | Status |
|----------|--------|
| Variable initializers (78 patterns) | ✅ 100% semantic |
| Method calls (47+ patterns) | ✅ 100% semantic |
| Standalone expressions (14 patterns) | ✅ 100% semantic |
| Self-borrow inference | ✅ 100% semantic |

## Documentation

| Document | Description |
|----------|-------------|
| [Problem Statement](docs/01-PROBLEM-STATEMENT.md) | Why proc-macros can't access type information |
| [Solution Architecture](docs/02-SOLUTION-ARCHITECTURE.md) | Two-phase build strategy |
| [Implementation Details](docs/03-IMPLEMENTATION-DETAILS.md) | rust-analyzer integration |
| [Output Format](docs/04-OUTPUT-FORMAT.md) | Complete JSON schema reference |
| [Macro Integration](docs/05-MACRO-INTEGRATION.md) | How the macro uses type info |
| [Usage Guide](docs/06-USAGE-GUIDE.md) | Running the analyzer |
| [Performance](docs/07-PERFORMANCE.md) | Benchmarks and optimization |
| [Limitations](docs/08-LIMITATIONS.md) | Current constraints |
| [Roadmap](docs/09-ROADMAP.md) | Development history and phases |
| [Dependencies](docs/10-DEPENDENCIES.md) | rust-analyzer crates used |
| [Semantic Expansion Plan](docs/SEMANTIC_EXPANSION_PLAN.md) | Implementation status |

## Dependencies

```toml
ra_ap_hir = "0.0.232"
ra_ap_ide_db = "0.0.232"
ra_ap_load-cargo = "0.0.232"
ra_ap_project_model = "0.0.232"
ra_ap_syntax = "0.0.232"
ra_ap_vfs = "0.0.232"
```

## License

Apache 2.0 - See [LICENSE](../LICENSE)
