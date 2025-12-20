🦀 I built something to solve a problem that frustrated me when learning Rust.

Rust's ownership system is powerful — it guarantees memory safety without a garbage collector. But it's also invisible. When the compiler says "cannot borrow as mutable because it is also borrowed as immutable," you're left mentally tracing lifetimes through your code.

So I created BorrowScope — a runtime tracking library that makes ownership visible.

Instead of guessing where borrows start and end, you can now watch them happen:

```rust
let data = track_new("data", vec![1, 2, 3]);
let reference = track_borrow("reference", &data);
```

Every ownership transfer, every borrow, every drop — captured as events you can export to JSON and analyze.

The core library (borrowscope-runtime) provides 41 tracking functions covering:
→ Basic ownership (new, borrow, move, drop)
→ Smart pointers (Rc, Arc cloning)
→ Interior mutability (RefCell, Cell)
→ Unsafe operations (raw pointers, FFI)
→ Async patterns (futures, polling)

Zero overhead when disabled — all tracking compiles away in release builds.

This is just the beginning. The roadmap includes automatic instrumentation via macros, graph-based ownership analysis, a CLI tool, and interactive visualization.

Whether you're learning Rust, teaching it, or debugging complex ownership patterns in production — I hope BorrowScope helps make the invisible visible.

📖 Full article with code examples in the comments

🔗 GitHub: github.com/mehmet-ylcnky/BorrowScope

Open source under Apache 2.0. Contributions and feedback welcome!

#Rust #Programming #OpenSource #MemorySafety #DeveloperTools #SoftwareEngineering
