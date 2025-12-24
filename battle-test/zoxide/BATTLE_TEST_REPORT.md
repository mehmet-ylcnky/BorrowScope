# BorrowScope Battle Test: zoxide

**Project:** [zoxide](https://github.com/ajeetdsouza/zoxide)  
**Version:** Latest (cloned on 2024-12-24)  
**Stars:** ~23k  
**Description:** A smarter cd command, inspired by z and autojump  

---

## Phase 1: Reconnaissance

### Lines of Code
```
~2,126 lines total
```

### Key Modules to Test
| Module | Description | Ownership Patterns |
|--------|-------------|-------------------|
| `src/util.rs` | Utilities | Path handling, string manipulation |
| `src/db/` | Database read/write | File I/O, borrows, Result handling |
| `src/cmd/` | CLI commands | String ownership, Option/Result |

---

## Error Log

### ERR-004: Tracking calls inserted into const context

**Location:**
- File: `src/util.rs`
- Line: 234 (macro), 239 (const definition)
- Function: `fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()>`

**Error Message:**
```
error[E0015]: cannot call non-const function `track_branch` in constants
   --> src/util.rs:234:1
    |
234 | #[borrowscope_macro::trace_borrow]
    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ 
    |
    = note: calls in constants are limited to constant functions, tuple structs and tuple variants
```

**Category:** Const Context Issue  
**Severity:** Critical  
**Component:** borrowscope-macro  
**Frequency:** TBD

**Minimal Reproducer:**
```rust
#[borrowscope_macro::trace_borrow]
fn example() {
    const VALUE: usize = if cfg!(windows) { 5 } else { 1 };
    // Error: track_branch inserted into const context
}
```

**Macro Expansion (problematic):**
```rust
const MAX_ATTEMPTS: usize = if false {
    borrowscope_runtime::track_branch(6usize, "then", "src/util.rs:239");  // NOT ALLOWED IN CONST!
    { 5 }
} else {
    borrowscope_runtime::track_branch(6usize, "else", "src/util.rs:239");  // NOT ALLOWED IN CONST!
    { 1 }
};
```

**Root Cause:**
The macro transforms `if` expressions by inserting `track_branch` calls, but doesn't check if the `if` is inside a const context (`const`, `static`, or const fn). Rust const evaluation doesn't allow non-const function calls.

**Proposed Solution:**
- File: `borrowscope-macro/src/transform_visitor.rs`
- Change: Track when visiting const items and skip transformation:

```rust
// Add a flag to OwnershipVisitor
struct OwnershipVisitor {
    in_const_context: bool,
    // ...
}

// In visit_item_const or when encountering const/static
fn visit_item_const(&mut self, item: &mut ItemConst) {
    self.in_const_context = true;
    // visit children
    self.in_const_context = false;
}

// In transform_if, check the flag
fn transform_if(&mut self, expr: &mut Expr) {
    if self.in_const_context {
        return; // Skip tracking in const context
    }
    // ... normal transformation
}
```

**New Feature Required:** Yes
- Const-context awareness in borrowscope-macro
- Add test in `borrowscope-macro/tests/` for functions with const items
- Add example in `borrowscope-macro/examples/` demonstrating const handling

**Workaround:** 
Skip `#[trace_borrow]` on functions containing `const` or `static` definitions with conditional expressions.

---

### ERR-003: Mutable method call on tracked variable becomes immutable borrow

**Location:**
- File: `src/util.rs`
- Line: 28 (macro), 42 (method call)
- Function: `impl Fzf { fn new() -> Result<Self> }`

**Error Message:**
```
error[E0596]: cannot borrow data in a `&` reference as mutable
  --> src/util.rs:28:5
   |
28 |     #[borrowscope_macro::trace_borrow]
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ cannot borrow as mutable
```

**Category:** Mutability Issue  
**Severity:** Critical  
**Component:** borrowscope-macro  
**Frequency:** TBD (common pattern)

**Minimal Reproducer:**
```rust
#[borrowscope_macro::trace_borrow]
fn example() {
    let mut cmd = Command::new("ls");
    cmd.args(["--help"]);  // Error: cannot borrow as mutable
}
```

**Macro Expansion (problematic):**
```rust
let mut cmd = borrowscope_runtime::__track_new_with_id_helper(..., Command::new(program));
borrowscope_runtime::track_borrow("method_borrow", &cmd)  // Returns &cmd (immutable!)
    .args([...])  // .args() requires &mut self - FAILS
    .stdin(Stdio::piped())
    .stdout(Stdio::piped());
```

**Root Cause:**
When transforming method calls like `cmd.args(...)`, the macro wraps the receiver with `track_borrow("method_borrow", &cmd)`. This returns an immutable reference `&Command`, but `.args()` requires `&mut self`.

The macro doesn't distinguish between:
- Methods that take `&self` (immutable) 
- Methods that take `&mut self` (mutable)

**Proposed Solution:**
- File: `borrowscope-macro/src/transform_visitor.rs`
- Change: For method calls on mutable variables, use `track_borrow_mut` instead of `track_borrow`:

```rust
// Option 1: Use track_borrow_mut for mutable receivers
borrowscope_runtime::track_borrow_mut("method_borrow", &mut cmd)
    .args([...])

// Option 2: Don't wrap method receivers at all (simpler)
// Just track the variable creation, not every method call
let mut cmd = track_new("cmd", Command::new(program));
cmd.args([...]);  // Leave method calls alone

// Option 3: Track after the method chain completes
let mut cmd = Command::new(program);
cmd.args([...]).stdin(...).stdout(...);
track_new("cmd", &cmd);  // Track state after mutations
```

**New Feature Required:** Yes
- Mutability-aware method call transformation
- Add test in `borrowscope-macro/tests/` for mutable method chains
- Add example in `borrowscope-macro/examples/` demonstrating builder patterns

**Workaround:** 
Skip `#[trace_borrow]` on functions with mutable method chains (builder pattern).

---

### ERR-002: Tuple destructuring pattern not properly handled

**Location:**
- File: `src/util.rs`
- Line: 161
- Function: `fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Result<()>`

**Error Message:**
```
error[E0425]: cannot find value `tmp_file` in this scope
   --> src/util.rs:164:13
    |
164 |         _ = tmp_file.set_len(contents.len() as u64);
    |             ^^^^^^^^ help: a function with a similar name exists: `tmpfile`
```

**Category:** Unsupported Pattern  
**Severity:** Critical  
**Component:** borrowscope-macro  
**Frequency:** TBD

**Minimal Reproducer:**
```rust
#[borrowscope_macro::trace_borrow]
fn example() -> Result<()> {
    let (mut file, path) = create_temp_file()?;
    file.write_all(b"hello")?;  // Error: cannot find value `file`
    Ok(())
}
```

**Macro Expansion (problematic):**
```rust
// Original: let (mut tmp_file, tmp_path) = tmpfile(dir)?;
// Expanded:
let __pattern_temp_8 = borrowscope_runtime::track_new(
    "__pattern_temp_8",
    tmpfile(dir)?,
);
// Missing: extraction of tmp_file and tmp_path from the tuple!
```

**Root Cause:**
The macro transforms tuple destructuring patterns into a single temporary variable but fails to:
1. Extract the individual tuple elements
2. Bind them to their original names (`tmp_file`, `tmp_path`)

The `visit_local` in `transform_visitor.rs` likely handles simple `let x = ...` patterns but not `let (a, b) = ...` tuple patterns.

**Proposed Solution:**
- File: `borrowscope-macro/src/transform_visitor.rs`
- Change: When encountering tuple patterns in `let` statements:

```rust
// Option 1: Track the whole tuple, then destructure
let __tracked_tuple = borrowscope_runtime::track_new("tuple", tmpfile(dir)?);
let (mut tmp_file, tmp_path) = __tracked_tuple;

// Option 2: Skip tracking for tuple patterns (simpler)
// Just leave tuple destructuring as-is

// Option 3: Track each element individually after destructuring
let (mut tmp_file, tmp_path) = tmpfile(dir)?;
borrowscope_runtime::track_new("tmp_file", &tmp_file);
borrowscope_runtime::track_new("tmp_path", &tmp_path);
```

**New Feature Required:** Yes
- Proper tuple pattern handling in borrowscope-macro
- Add test in `borrowscope-macro/tests/` for tuple destructuring
- Add example in `borrowscope-macro/examples/` demonstrating tuple patterns

**Workaround:** 
Skip `#[trace_borrow]` on functions with tuple destructuring patterns.

---

### ERR-001: Lifetime mismatch when function returns reference to input parameter

**Location:**
- File: `src/util.rs`
- Line: 269-272
- Function: `fn path_to_str(path: &impl AsRef<Path>) -> Result<&str>`

**Error Message:**
```
error[E0515]: cannot return reference to function parameter `path`
   --> src/util.rs:269:1
    |
269 | #[borrowscope_macro::trace_borrow]
    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ returns a reference to data owned by the current function
```

**Category:** Lifetime/Borrow Issue  
**Severity:** Critical  
**Component:** borrowscope-macro  
**Frequency:** TBD (first occurrence)

**Minimal Reproducer:**
```rust
#[borrowscope_macro::trace_borrow]
pub fn path_to_str(path: &impl AsRef<Path>) -> Result<&str> {
    let path = path.as_ref();
    path.to_str().with_context(|| format!("invalid unicode in path: {}", path.display()))
}
```

**Macro Expansion (problematic):**
```rust
pub fn path_to_str(path: &impl AsRef<Path>) -> Result<&str> {
    let path = borrowscope_runtime::__track_new_with_id_helper(
        2usize,
        "path",
        "src/util.rs:271",
        borrowscope_runtime::track_borrow("method_borrow", &path).as_ref(),
    );
    // ... path is now a NEW binding, not the original parameter
    // returning a reference to this new binding fails lifetime check
}
```

**Root Cause:**
The macro transforms `let path = path.as_ref();` by wrapping it with tracking, which creates a new binding. When the function returns a reference (`&str`) that should be tied to the INPUT parameter's lifetime, it instead gets tied to the LOCAL binding's lifetime, which doesn't live long enough.

The issue is that `#[trace_borrow]` doesn't understand that:
1. The function returns a reference
2. That reference's lifetime is tied to an input parameter
3. Shadowing the input parameter breaks this lifetime relationship

**Proposed Solution:**
- File: `borrowscope-macro/src/transform_visitor.rs`
- Change: Detect when a function returns a reference and an input parameter is shadowed. In such cases, either:
  1. Skip tracking the shadowing assignment, OR
  2. Use a different variable name for tracking (e.g., `__path_tracked`) to preserve the original binding

```rust
// Option 1: Skip tracking when shadowing would break lifetimes
// In visit_local(), check if:
// - Function returns a reference
// - The variable being assigned shadows an input parameter
// - The input parameter's lifetime flows to the return type

// Option 2: Rename tracked variable
let __path_tracked = borrowscope_runtime::track_borrow("path", &path);
let path = path.as_ref();  // Original shadowing preserved
```

**New Feature Required:** Yes
- Lifetime-aware transformation in borrowscope-macro
- Add test in `borrowscope-macro/tests/` for functions returning references
- Add example in `borrowscope-macro/examples/` demonstrating this pattern

**Workaround:** 
Skip `#[trace_borrow]` on functions that return references tied to input parameters.

---

### ERR-005: File included by build.rs cannot use proc macro

**Location:**
- File: `src/cmd/cmd.rs`
- Line: 11
- Function: `HelpTemplate::into_resettable`

**Error Message:**
```
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `borrowscope_macro`
  --> src/cmd/cmd.rs:11:7
   |
11 |     #[borrowscope_macro::trace_borrow]
```

**Category:** Build System Issue  
**Severity:** Critical  
**Component:** borrowscope-macro (usage constraint)  
**Frequency:** Rare (only affects files included by build.rs)

**Root Cause:**
The file `src/cmd/cmd.rs` is included directly by `build.rs` via `#[path = "src/cmd/cmd.rs"]`. Build scripts compile in a separate context without access to the main crate's dependencies. When the macro is added to a function in this file, the build script fails because `borrowscope_macro` isn't available in the build context.

**Proposed Solution:**
This is a usage constraint, not a macro bug. Options:
1. Document that `#[trace_borrow]` cannot be used in files included by build.rs
2. Add a compile-time check/warning when this pattern is detected

**New Feature Required:** No (documentation only)

**Workaround:** 
Skip `#[trace_borrow]` on functions in files that are included by build.rs.

---

### ERR-006: Temporary value dropped while borrowed (E0716)

**Location:**
- File: `src/cmd/init.rs`, `src/cmd/query.rs`
- Functions: `Init::run`, `Query::query_list`, `Query::query_first`

**Error Message:**
```
error[E0716]: temporary value dropped while borrowed
  --> src/cmd/init.rs:12:5
   |
12 |     #[borrowscope_macro::trace_borrow]
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^-
   |     |                                |
   |     |                                temporary value is freed at the end of this statement
```

**Category:** Lifetime Issue  
**Severity:** Critical  
**Component:** borrowscope-macro  
**Frequency:** Common (method chains with temporaries)

**Minimal Reproducer:**
```rust
#[borrowscope_macro::trace_borrow]
fn example() {
    let handle = &mut io::stdout().lock();  // temporary!
    writeln!(handle, "hello")?;
}
```

**Root Cause:**
The macro wraps expressions in tracking calls, which can extend the lifetime requirements of temporaries. When a temporary (like `io::stdout().lock()`) is borrowed, the macro's transformation causes the temporary to be dropped before the borrow ends.

**Proposed Solution:**
Similar to ERR-003 - avoid wrapping method call receivers that create temporaries.

**New Feature Required:** Yes
- Detect temporary-creating expressions and skip tracking

**Workaround:** 
Skip `#[trace_borrow]` on functions with `&mut temporary().method()` patterns.

---

## Progress Log

### src/config.rs (6 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `data_dir` | ✅ Pass | Match, Option handling |
| `echo` | ✅ Pass | Simple env check |
| `exclude_dirs` | ✅ Pass | Complex closures, iterators |
| `fzf_opts` | ✅ Pass | Simple env check |
| `maxage` | ✅ Pass | map_or, closures |
| `resolve_symlinks` | ✅ Pass | Simple env check |

### src/error.rs (2 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `SilentExit::fmt` | ✅ Pass | Display trait impl |
| `BrokenPipeHandler::pipe_exit` | ✅ Pass | Trait impl, match with guards |

### src/main.rs (1 function)
| Function | Status | Notes |
|----------|--------|-------|
| `main` | ✅ Pass | unsafe blocks, match, downcast |

### src/shell.rs (1 function + 20 test functions)
| Function | Status | Notes |
|----------|--------|-------|
| `deref` (in macro) | ✅ Pass | Macro-generated Deref impl |
| `opts` (test template) | ✅ Pass | rstest template |
| `bash_bash` | ✅ Pass | Test function |
| `bash_shellcheck` | ✅ Pass | Test function |
| `bash_shfmt` | ✅ Pass | Test function |
| `elvish_elvish` | ✅ Pass | Test function |
| `fish_no_builtin_abbr` | ✅ Pass | Test function |
| `fish_fish` | ✅ Pass | Test function |
| `fish_fishindent` | ✅ Pass | Test function |
| `nushell_nushell` | ✅ Pass | Test function |
| `posix_bash` | ✅ Pass | Test function |
| `posix_dash` | ✅ Pass | Test function |
| `posix_shellcheck` | ✅ Pass | Test function |
| `posix_shfmt` | ✅ Pass | Test function |
| `powershell_pwsh` | ✅ Pass | Test function |
| `tcsh_tcsh` | ✅ Pass | Test function |
| `xonsh_black` | ✅ Pass | Test function |
| `xonsh_mypy` | ✅ Pass | Test function |
| `xonsh_pylint` | ✅ Pass | Test function |
| `xonsh_xonsh` | ✅ Pass | Test function |
| `zsh_shellcheck` | ✅ Pass | Test function |
| `zsh_zsh` | ✅ Pass | Test function |

### src/util.rs (17 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `Fzf::new` | ❌ ERR-003 | Mutable method chain (builder) |
| `Fzf::enable_preview` | ❌ ERR-001,003 | Returns &mut self + mutable chain |
| `Fzf::args` | ✅ Pass | Generic method with &mut self |
| `Fzf::env` | ✅ Pass | Generic method |
| `Fzf::envs` | ✅ Pass | Generic method |
| `Fzf::spawn` | ✅ Pass | Match with guards |
| `FzfChild::write` | ✅ Pass | Match with guards, write! macro |
| `FzfChild::wait` | ✅ Pass | Complex match, mem::drop |
| `write` | ❌ ERR-002 | Tuple destructuring pattern |
| `tmpfile` | ✅ Pass | Loop, match, const |
| `rename` | ❌ ERR-004 | const with cfg! conditional |
| `canonicalize` | ✅ Pass | impl AsRef parameter |
| `current_dir` | ✅ Pass | Simple wrapper |
| `current_time` | ✅ Pass | Result return, method chaining |
| `path_to_str` | ❌ ERR-001 | Returns reference to input |
| `resolve_path` | ✅ Pass | Complex control flow, nested functions |
| `get_drive_letter` | ✅ Pass | Nested function (Windows only) |
| `get_drive_path` | ✅ Pass | Nested function (Windows only) |
| `get_drive_relative` | ✅ Pass | Nested function (Windows only) |
| `to_lowercase` | ✅ Pass | Simple function |

### src/cmd/mod.rs (1 function)
| Function | Status | Notes |
|----------|--------|-------|
| `Cmd::run` | ✅ Pass | Match dispatch |

### src/cmd/cmd.rs (1 function)
| Function | Status | Notes |
|----------|--------|-------|
| `HelpTemplate::into_resettable` | ❌ ERR-005 | File included by build.rs |

### src/cmd/add.rs (1 function)
| Function | Status | Notes |
|----------|--------|-------|
| `Add::run` | ❌ ERR-003,004 | Mutable borrow + const context |

### src/cmd/edit.rs (2 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `Edit::run` | ❌ ERR-003 | Mutable borrow issue |
| `Edit::get_fzf` | ❌ ERR-003 | Mutable method chain (builder) |

### src/cmd/import.rs (4 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `Import::run` | ❌ ERR-003 | Mutable borrow issue |
| `import_autojump` | ❌ ERR-003 | Mutable borrow issue |
| `import_z` | ❌ ERR-003 | Mutable borrow issue |
| `sigmoid` | ❌ ERR-003 | (blocked by above) |

### src/cmd/init.rs (1 function)
| Function | Status | Notes |
|----------|--------|-------|
| `Init::run` | ❌ E0716 | Temporary value dropped while borrowed |

### src/cmd/query.rs (7 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `Query::run` | ❌ ERR-003 | Mutable borrow issue |
| `Query::query` | ❌ ERR-003 | Mutable borrow issue |
| `Query::query_interactive` | ❌ ERR-003 | Mutable borrow issue |
| `Query::query_list` | ❌ E0716 | Temporary dropped while borrowed |
| `Query::query_first` | ❌ E0716 | Temporary dropped while borrowed |
| `Query::get_stream` | ❌ ERR-003 | Mutable borrow issue |
| `Query::get_fzf` | ❌ ERR-003 | Mutable method chain |

### src/cmd/remove.rs (1 function)
| Function | Status | Notes |
|----------|--------|-------|
| `Remove::run` | ❌ ERR-003 | Mutable borrow issue |

### src/db/dir.rs (6 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `Dir::display` | ✅ Pass | Returns DirDisplay |
| `Dir::score` | ✅ Pass | Simple calculation |
| `DirDisplay::new` | ✅ Pass | Constructor |
| `DirDisplay::with_score` | ✅ Pass | Builder method |
| `DirDisplay::with_separator` | ✅ Pass | Builder method |
| `DirDisplay::fmt` | ✅ Pass | Display trait impl |

### src/db/mod.rs (16 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `Database::open` | ✅ Pass | Simple wrapper |
| `Database::open_dir` | ✅ Pass | Complex match, Result |
| `Database::save` | ❌ ERR-003 | Mutable borrow issue |
| `Database::add` | ❌ ERR-003 | Mutable borrow issue |
| `Database::add_unchecked` | ❌ ERR-003 | Mutable borrow issue |
| `Database::add_update` | ❌ ERR-003 | Mutable borrow issue |
| `Database::remove` | ✅ Pass | Match with position |
| `Database::swap_remove` | ❌ ERR-003 | Mutable borrow issue |
| `Database::age` | ❌ ERR-003 | Mutable borrow issue |
| `Database::dedup` | ❌ ERR-003 | Mutable borrow issue |
| `Database::sort_by_path` | ❌ ERR-003 | Mutable borrow issue |
| `Database::sort_by_score` | ❌ ERR-003 | Mutable borrow issue |
| `Database::dirty` | ✅ Pass | Simple getter |
| `Database::dirs` | ✅ Pass | Simple getter |
| `Database::serialize` | ✅ Pass | Closure, bincode |
| `Database::deserialize` | ❌ ERR-006 | Temporary dropped while borrowed |

### src/db/stream.rs (12 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `Stream::new` | ❌ ERR-003 | Mutable borrow issue |
| `Stream::next` | ✅ Pass | Complex while loop |
| `Stream::filter_by_base_dir` | ✅ Pass | Match expression |
| `Stream::filter_by_exclude` | ✅ Pass | Iterator with closure |
| `Stream::filter_by_exists` | ✅ Pass | Conditional assignment |
| `Stream::filter_by_keywords` | ✅ Pass | Complex string matching |
| `StreamOptions::new` | ✅ Pass | Struct initialization |
| `StreamOptions::with_keywords` | ✅ Pass | Generic builder |
| `StreamOptions::with_exclude` | ✅ Pass | Builder method |
| `StreamOptions::with_exists` | ✅ Pass | Builder method |
| `StreamOptions::with_resolve_symlinks` | ✅ Pass | Builder method |
| `StreamOptions::with_base_dir` | ✅ Pass | Builder method |

---

## Summary (In Progress)

### Test Results by File

| File | Pass | Fail | Total | Pass Rate |
|------|------|------|-------|-----------|
| config.rs | 6 | 0 | 6 | 100% |
| error.rs | 2 | 0 | 2 | 100% |
| main.rs | 1 | 0 | 1 | 100% |
| shell.rs | 21 | 0 | 21 | 100% |
| util.rs | 12 | 5 | 17 | 71% |
| cmd/mod.rs | 1 | 0 | 1 | 100% |
| cmd/cmd.rs | 0 | 1 | 1 | 0% |
| cmd/add.rs | 0 | 1 | 1 | 0% |
| cmd/edit.rs | 0 | 2 | 2 | 0% |
| cmd/import.rs | 0 | 4 | 4 | 0% |
| cmd/init.rs | 0 | 1 | 1 | 0% |
| cmd/query.rs | 0 | 7 | 7 | 0% |
| cmd/remove.rs | 0 | 1 | 1 | 0% |
| db/dir.rs | 6 | 0 | 6 | 100% |
| db/mod.rs | 6 | 10 | 16 | 38% |
| db/stream.rs | 11 | 1 | 12 | 92% |
| **TOTAL** | **66** | **33** | **99** | **67%** |

### Patterns Tested

| Pattern | Status | Notes |
|---------|--------|-------|
| Basic let bindings | ✅ | Works |
| impl AsRef<T> parameters | ✅ | Works |
| Result<T> returns | ✅ | Works |
| Method chaining | ✅ | Works |
| Complex control flow | ✅ | Works |
| Returns reference to input | ❌ | ERR-001 |

### Gaps Identified

#### Critical (Blocks Usage)
| ID | Gap | Description | Workaround |
|----|-----|-------------|------------|
| ERR-001 | Lifetime-breaking shadowing | Macro shadows input params, breaking return lifetime | Skip these functions |
| ERR-002 | Tuple destructuring | Tuple patterns not properly extracted | Skip these functions |
| ERR-003 | Mutable method chains | track_borrow returns &T, but method needs &mut T | Skip builder patterns |
| ERR-004 | Const context tracking | track_branch called in const/static context | Skip functions with const conditionals |
| ERR-005 | Build.rs inclusion | Files included by build.rs can't use proc macros | Skip these files |
| ERR-006 | Temporary dropped | Tracking extends temporary lifetime requirements | Skip &mut temp().method() patterns |
