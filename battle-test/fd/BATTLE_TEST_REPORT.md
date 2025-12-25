# BorrowScope Battle Test: fd

**Project:** [fd](https://github.com/sharkdp/fd)  
**Version:** Latest (cloned on 2024-12-24)  
**Stars:** ~35k  
**Description:** A simple, fast and user-friendly alternative to find  

**Note:** "Pass" means the macro transformation compiles without errors. Full runtime verification is blocked by any compilation failures.

---

## Phase 1: Reconnaissance

### Lines of Code
```
~4,651 lines total
```

### Key Modules to Test
| Module | Description | Ownership Patterns |
|--------|-------------|-------------------|
| `src/walk.rs` | Directory traversal | Parallel iterators, ignore patterns |
| `src/exec/` | Command execution | Process spawning, job management |
| `src/filter/` | Result filtering | Size/time/owner predicates |
| `src/dir_entry.rs` | Directory entries | Path ownership, metadata |
| `src/filesystem.rs` | FS operations | Path handling, symlinks |
| `src/cli.rs` | Argument parsing | Config building |
| `src/output.rs` | Result output | Formatting, colors |

---

## Phase 2: Error Log

### ERR-002: Tuple Destructuring Pattern

**Location:**
- File: `src/filter/owner.rs`
- Line: 103
- Function: `Check::parse`

**Error Message:**
```
error[E0425]: cannot find value `equality` in this scope
   --> src/filter/owner.rs:103:16
    |
103 |             if equality {
    |                ^^^^^^^^ not found in this scope
```

**Category:** Pattern Transformation Issue
**Severity:** Critical
**Component:** borrowscope-macro
**Frequency:** Common (any function with tuple destructuring)

**Minimal Reproducer:**
```rust
#[trace_borrow]
fn parse(input: &str) -> (String, bool) {
    let (s, flag) = match input {
        x if x.starts_with("!") => (&x[1..], true),
        x => (x, false),
    };
    (s.to_string(), flag)  // Error: cannot find value `s`
}
```

**Root Cause:**
Macro transforms tuple patterns into single temporary but fails to extract individual elements. `let (s, equality) = match s { ... }` breaks both variable bindings.

**Proposed Solution:**
- File: `borrowscope-macro/src/transform_visitor.rs`
- Change: When encountering tuple patterns, either skip tracking or destructure after tracking the whole tuple.

**New Feature Required:** Yes
- Proper tuple pattern handling in borrowscope-macro

**Workaround:**
Skip `#[trace_borrow]` on functions with tuple destructuring patterns.

---

### ERR-003: Mutable Borrow Conflicts

**Location:**
- File: `src/exec/mod.rs` (line 178), `src/filter/owner.rs` (line 29), `src/walk.rs` (lines 107, 277, 363), `src/main.rs` (line 114)
- Functions: `CommandBuilder::new_command`, `OwnerFilter::from_string`, `BatchSender::send`, `ReceiverBuffer::stream`, `WorkerState::build_walker`, `print_completions`

**Error Message:**
```
error[E0596]: cannot borrow data in a `&` reference as mutable
   --> src/exec/mod.rs:178:5
    |
178 |     #[trace_borrow]
    |     ^^^^^^^^^^^^^^^ cannot borrow as mutable
```

**Category:** Borrow Transformation Issue
**Severity:** Critical
**Component:** borrowscope-macro
**Frequency:** Very Common (any function with mutable method chains)

**Minimal Reproducer:**
```rust
#[trace_borrow]
fn example() {
    let mut cmd = Command::new("ls");
    cmd.args(["--help"]);  // Error: cannot borrow as mutable
}
```

**Root Cause:**
Macro wraps receiver with `track_borrow("method_borrow", &cmd)` which returns `&T`, but method requires `&mut self`.

**Proposed Solution:**
- File: `borrowscope-macro/src/transform_visitor.rs`
- Change: Use `track_borrow_mut` for mutable receivers, or skip method call tracking entirely.

**New Feature Required:** Yes
- Mutability-aware method call transformation

**Workaround:**
Skip `#[trace_borrow]` on functions with mutable method chains.

---

### ERR-008: impl Into/Trait Bounds Fail

**Location:**
- File: `src/filter/time.rs` (line 15), `src/exit_codes.rs` (line 34)
- Functions: `TimeFilter::from_str`, `ExitCode::exit`

**Error Message:**
```
error[E0277]: the trait bound `DateTime<Local>: From<&DateTime<FixedOffset>>` is not satisfied
  --> src/filter/time.rs:15:5
   |
15 |     #[trace_borrow]
   |     ^^^^^^^^^^^^^^^ the trait `From<&DateTime<FixedOffset>>` is not implemented
...
22 |                     .map(|dt| dt.into())
   |                                  ---- required by a bound introduced by this call
```

**Category:** Generic Parameter Transformation Issue
**Severity:** Critical
**Component:** borrowscope-macro
**Frequency:** Common (any function with `.into()` calls on tracked values)

**Minimal Reproducer:**
```rust
#[trace_borrow]
pub fn new(s: impl Into<String>) -> Self {
    let s = s.into();  // Error: trait bound not satisfied
    Self { value: s }
}
```

**Root Cause:**
Macro wraps `impl Into<String>` parameters with tracking calls, changing the type to a tracked reference which no longer implements `Into<String>`.

**Proposed Solution:**
- File: `borrowscope-macro/src/transform_visitor.rs`
- Change: Skip tracking for parameters with `impl Trait` types.

**New Feature Required:** Yes
- impl Trait parameter handling

**Workaround:**
Skip `#[trace_borrow]` on functions with `impl Into<T>` or similar `impl Trait` parameters.

---

### ERR-009: Self-Consuming / Move Semantics

**Location:**
- File: `src/cli.rs` (line 814), `src/walk.rs` (line 466), `src/dir_entry.rs` (line 147)
- Functions: `augment_args`, `spawn_senders`, `Colorable::file_name`

**Error Message:**
```
error[E0507]: cannot move out of a shared reference
   --> src/cli.rs:814:5
    |
814 |     #[trace_borrow]
    |     ^^^^^^^^^^^^^^^ move occurs because value has type `clap::Command`, which does not implement the `Copy` trait

error[E0308]: mismatched types
   --> src/dir_entry.rs:147:5
    |
147 |     #[trace_borrow]
    |     ^^^^^^^^^^^^^^^ expected `OsString`, found `&OsStr`
```

**Category:** Ownership Transformation Issue
**Severity:** Critical
**Component:** borrowscope-macro
**Frequency:** Common (any function with `self` parameter or owned return types)

**Minimal Reproducer:**
```rust
#[trace_borrow]
pub fn with_name(self, name: &str) -> Self {
    self  // Error: cannot move out of shared reference
}
```

**Root Cause:**
Macro transforms `self` by wrapping with tracking calls that create references. Function can no longer return `Self` because it only has a borrowed reference.

**Proposed Solution:**
- File: `borrowscope-macro/src/transform_visitor.rs`
- Change: Detect self-consuming functions and skip tracking or use `track_move`.

**New Feature Required:** Yes
- Self-consuming function detection

**Workaround:**
Skip `#[trace_borrow]` on functions that consume `self` (builder patterns, transformation methods).

---

### ERR-010: Range/Method Indexing Fails

**Location:**
- File: `src/filter/size.rs`
- Line: 36
- Function: `SizeFilter::parse_opt`

**Error Message:**
```
error[E0061]: this method takes 1 argument but 0 arguments were supplied
    --> src/filter/size.rs:36:5
     |
36   |     #[trace_borrow]
     |     ^^^^^^^^^^^^^^^ argument #1 of type `usize` is missing
     |
note: method defined here
    --> regex/src/regex/string.rs:1650:12
     |
1650 |     pub fn get(&self, i: usize) -> Option<Match<'h>> {
     |            ^^^
```

**Category:** Method Call Transformation Issue
**Severity:** Critical
**Component:** borrowscope-macro
**Frequency:** Uncommon (functions using `.get(index)` on captures)

**Minimal Reproducer:**
```rust
#[trace_borrow]
fn parse(input: &[u8]) -> Option<&[u8]> {
    input.get(0..4)  // Error: argument missing
}
```

**Root Cause:**
Macro transforms the receiver and wraps it with tracking calls. The range argument `0..4` is not properly passed through, resulting in `.get()` with no arguments.

**Proposed Solution:**
- File: `borrowscope-macro/src/transform_visitor.rs`
- Change: Preserve method call arguments when transforming expressions with range literals.

**New Feature Required:** Yes
- Range expression preservation in method calls

**Workaround:**
Skip `#[trace_borrow]` on functions that use `.get(range)` or similar range-indexed method calls.

---

## Phase 3: Compilation Results

### src/cli.rs (17 functions)

| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `search_paths` | ✅ Pass | - | - |
| `normalize_path` | ✅ Pass | - | - |
| `no_search_paths` | ✅ Pass | - | - |
| `rg_alias_ignore` | ✅ Pass | - | - |
| `max_depth` | ✅ Pass | - | - |
| `min_depth` | ✅ Pass | - | - |
| `threads` | ✅ Pass | - | - |
| `max_results` | ✅ Pass | - | - |
| `gen_completions` | ✅ Pass | - | cfg(feature) |
| `default_num_threads` | ✅ Pass | - | - |
| `as_str` | ✅ Pass | - | - |
| `from_arg_matches` | ✅ Pass | - | - |
| `update_from_arg_matches` | ✅ Pass | - | - |
| `augment_args` | ❌ Fail | ERR-009 | E0507 move out of shared ref |
| `augment_args_for_update` | ✅ Pass | - | - |
| `parse_millis` | ✅ Pass | - | - |
| `ensure_current_directory_exists` | ✅ Pass | - | - |

### src/config.rs (1 function)

| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `is_printing` | ✅ Pass | - | - |

### src/dir_entry.rs (17 functions)

| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `normal` | ✅ Pass | - | - |
| `broken_symlink` | ✅ Pass | - | - |
| `path` | ✅ Pass | - | - |
| `into_path` | ✅ Pass | - | self-consuming |
| `stripped_path` | ✅ Pass | - | - |
| `into_stripped_path` | ✅ Pass | - | self-consuming |
| `file_type` | ✅ Pass | - | - |
| `metadata` | ✅ Pass | - | - |
| `depth` | ✅ Pass | - | - |
| `style` | ✅ Pass | - | - |
| `eq` | ✅ Pass | - | PartialEq |
| `partial_cmp` | ✅ Pass | - | PartialOrd |
| `cmp` | ✅ Pass | - | Ord |
| `Colorable::path` | ✅ Pass | - | trait impl |
| `Colorable::file_name` | ❌ Fail | ERR-009 | E0308 expected OsString, found &OsStr |
| `Colorable::file_type` | ✅ Pass | - | trait impl |
| `Colorable::metadata` | ✅ Pass | - | trait impl |

### src/error.rs (1 function)

| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `print_error` | ✅ Pass | - | impl Into<String> works here |

### src/exit_codes.rs (4 functions)

| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `From::from` | ✅ Pass | - | - |
| `is_error` | ✅ Pass | - | - |
| `exit` | ❌ Fail | ERR-008 | E0277 trait From not implemented |
| `merge_exitcodes` | ✅ Pass | - | impl IntoIterator works |

### src/filesystem.rs (10 functions)

| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `path_absolute_form` | ✅ Pass | - | - |
| `absolute_path` | ✅ Pass | - | - |
| `is_existing_directory` | ✅ Pass | - | - |
| `is_empty` | ✅ Pass | - | - |
| `is_block_device` | ✅ Pass | - | cfg(unix) |
| `is_char_device` | ✅ Pass | - | cfg(unix) |
| `is_socket` | ✅ Pass | - | cfg(unix) |
| `is_pipe` | ✅ Pass | - | cfg(unix) |
| `osstr_to_bytes` | ✅ Pass | - | cfg(unix) |
| `strip_current_dir` | ✅ Pass | - | - |
| `default_path_separator` | ✅ Pass | - | - |

### src/filetypes.rs (1 function)

| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `should_ignore` | ✅ Pass | - | - |

### src/hyperlink.rs (5 functions)

| Function | Status | Error | Notes |
|----------|--------|-------|-------|

### src/main.rs (13 functions)

| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `main` | ✅ Pass | - | - |
| `run` | ✅ Pass | - | - |
| `print_completions` | ❌ Fail | ERR-003 | E0596 cannot borrow as mutable |
| `set_working_dir` | ✅ Pass | - | - |
| `ensure_search_pattern_is_not_a_path` | ✅ Pass | - | - |
| `build_pattern_regex` | ✅ Pass | - | - |
| `check_path_separator_length` | ✅ Pass | - | - |
| `construct_config` | ✅ Pass | - | - |
| `extract_command` | ✅ Pass | - | - |
| `determine_ls_command` | ✅ Pass | - | - |
| `extract_time_constraints` | ✅ Pass | - | - |
| `ensure_use_hidden_option_for_leading_dot_pattern` | ✅ Pass | - | - |
| `build_regex` | ✅ Pass | - | - |

### src/output.rs (7 functions)

| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `replace_path_separator` | ✅ Pass | - | - |
| `print_entry` | ✅ Pass | - | generic W: Write |
| `print_trailing_slash` | ✅ Pass | - | - |
| `print_entry_colorized` | ✅ Pass | - | - |
| `print_entry_uncolorized_base` | ✅ Pass | - | - |
| `print_entry_uncolorized` (unix) | ✅ Pass | - | cfg(unix) |
| `print_entry_uncolorized` (not unix) | ✅ Pass | - | cfg(not(unix)) |

### src/regex_helper.rs (4 functions)

| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `pattern_has_uppercase_char` | ✅ Pass | - | - |
| `hir_has_uppercase_char` | ✅ Pass | - | recursive |
| `pattern_matches_strings_with_leading_dot` | ✅ Pass | - | - |
| `hir_matches_strings_with_leading_dot` | ✅ Pass | - | - |

### src/walk.rs (21 functions)

| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `Batch::new` | ✅ Pass | - | - |
| `Batch::lock` | ✅ Pass | - | - |
| `Batch::into_iter` | ✅ Pass | - | - |
| `BatchSender::new` | ✅ Pass | - | - |
| `BatchSender::needs_flush` | ✅ Pass | - | - |
| `BatchSender::send` | ❌ Fail | ERR-003 | E0596 cannot borrow as mutable |
| `ReceiverBuffer::new` | ✅ Pass | - | - |
| `ReceiverBuffer::process` | ✅ Pass | - | - |
| `ReceiverBuffer::recv` | ✅ Pass | - | - |
| `ReceiverBuffer::poll` | ✅ Pass | - | - |
| `ReceiverBuffer::print` | ✅ Pass | - | - |
| `ReceiverBuffer::stream` | ❌ Fail | ERR-003 | E0596 cannot borrow as mutable |
| `ReceiverBuffer::stop` | ✅ Pass | - | - |
| `ReceiverBuffer::flush` | ✅ Pass | - | - |
| `WorkerState::new` | ✅ Pass | - | - |
| `WorkerState::build_overrides` | ✅ Pass | - | - |
| `WorkerState::build_walker` | ❌ Fail | ERR-003 | E0596 cannot borrow as mutable |
| `WorkerState::receive` | ✅ Pass | - | - |
| `WorkerState::spawn_senders` | ❌ Fail | ERR-009 | E0507 move out of shared ref |
| `WorkerState::scan` | ❌ Fail | ERR-003 | E0596 cannot borrow as mutable |
| `scan` | ✅ Pass | - | - |

### src/exec/mod.rs (16 functions)

| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `CommandSet::new` | ✅ Pass | - | - |
| `CommandSet::new_batch` | ✅ Pass | - | - |
| `CommandSet::in_batch_mode` | ✅ Pass | - | - |
| `CommandSet::execute` | ✅ Pass | - | - |
| `CommandSet::execute_batch` | ✅ Pass | - | - |
| `CommandBuilder::new` | ✅ Pass | - | - |
| `CommandBuilder::new_command` | ❌ Fail | ERR-003 | E0596 cannot borrow as mutable |
| `CommandBuilder::push` | ✅ Pass | - | &mut self |
| `CommandBuilder::finish` | ✅ Pass | - | &mut self |
| `CommandBuilder::exit_code` | ✅ Pass | - | - |
| `CommandTemplate::new` | ✅ Pass | - | - |
| `CommandTemplate::number_of_tokens` | ✅ Pass | - | - |
| `CommandTemplate::generate` | ✅ Pass | - | - |
| `ArgumentTemplate::has_tokens` | ✅ Pass | - | - |
| `ArgumentTemplate::generate` | ✅ Pass | - | - |
| `ArgumentTemplate::replace_separator` | ✅ Pass | - | - |

### src/exec/command.rs (5 functions)

| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `OutputBuffer::new` | ✅ Pass | - | - |
| `OutputBuffer::push` | ✅ Pass | - | &mut self |
| `OutputBuffer::write` | ✅ Pass | - | self-consuming |
| `execute_commands` | ✅ Pass | - | generic Iterator |
| `handle_cmd_error` | ✅ Pass | - | - |

### src/exec/input.rs (3 functions)

| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `basename` | ✅ Pass | - | - |
| `remove_extension` | ✅ Pass | - | - |
| `dirname` | ✅ Pass | - | - |

### src/exec/job.rs (2 functions)

| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `job` | ✅ Pass | - | impl IntoIterator |
| `batch` | ✅ Pass | - | impl IntoIterator |

### src/exec/token.rs (2 functions)

| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `tokenize` | ✅ Pass | - | - |
| `token_from_pattern_id` | ✅ Pass | - | - |

### src/filter/mod.rs (0 functions)

| Function | Status | Error | Notes |
|----------|--------|-------|-------|

### src/filter/owner.rs (5 functions)

| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `OwnerFilter::from_string` | ❌ Fail | ERR-003 | E0596 cannot borrow as mutable |
| `OwnerFilter::filter_ignore` | ✅ Pass | - | self-consuming |
| `OwnerFilter::matches` | ✅ Pass | - | - |
| `Check::check` | ✅ Pass | - | generic impl |
| `Check::parse` | ❌ Fail | ERR-002 | E0425/E0308 tuple destructuring in match |

### src/filter/size.rs (3 functions)

| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `SizeFilter::from_string` | ✅ Pass | - | - |
| `SizeFilter::parse_opt` | ❌ Fail | ERR-010 | E0061 .get() method call broken |
| `SizeFilter::is_within` | ✅ Pass | - | - |

### src/filter/time.rs (4 functions)

| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `TimeFilter::from_str` | ❌ Fail | ERR-008 | E0277 trait bound From not satisfied |
| `TimeFilter::before` | ✅ Pass | - | - |
| `TimeFilter::after` | ✅ Pass | - | - |
| `TimeFilter::applies_to` | ✅ Pass | - | - |

---

## Summary

### Test Results by File

| File | Pass | Fail | Total | Pass Rate |
|------|------|------|-------|-----------|
| src/error.rs | 1 | 0 | 1 | 100% |
| src/config.rs | 1 | 0 | 1 | 100% |
| src/exit_codes.rs | 3 | 1 | 4 | 75% |
| src/filetypes.rs | 1 | 0 | 1 | 100% |
| src/filesystem.rs | 11 | 0 | 11 | 100% |
| src/output.rs | 7 | 0 | 7 | 100% |
| src/regex_helper.rs | 4 | 0 | 4 | 100% |
| src/dir_entry.rs | 16 | 1 | 17 | 94% |
| src/cli.rs | 16 | 1 | 17 | 94% |
| src/main.rs | 12 | 1 | 13 | 92% |
| src/walk.rs | 16 | 5 | 21 | 76% |
| src/exec/mod.rs | 15 | 1 | 16 | 94% |
| src/exec/command.rs | 5 | 0 | 5 | 100% |
| src/exec/input.rs | 3 | 0 | 3 | 100% |
| src/exec/job.rs | 2 | 0 | 2 | 100% |
| src/exec/token.rs | 2 | 0 | 2 | 100% |
| src/filter/owner.rs | 3 | 2 | 5 | 60% |
| src/filter/size.rs | 2 | 1 | 3 | 67% |
| src/filter/time.rs | 3 | 1 | 4 | 75% |
| **TOTAL** | **123** | **14** | **137** | **90%** |

### Patterns Tested

| Pattern | Status | Notes |
|---------|--------|-------|
| Basic let bindings | ✅ | Works across all files |
| Immutable borrows | ✅ | Works in most cases |
| Mutable borrows | ⚠️ | ERR-003 in some contexts |
| Parallel iterators | ✅ | walk.rs parallel scanning |
| Path handling | ✅ | filesystem.rs, input.rs |
| Process spawning | ✅ | exec/command.rs |
| Regex captures | ❌ | ERR-010 breaks .get() |
| Tuple destructuring | ❌ | ERR-002 in match patterns |
| Trait conversions | ⚠️ | ERR-008 on .into() chains |

### Gaps Identified

#### Critical (Blocks Usage)
| ID | Gap | Description | Workaround |
|----|-----|-------------|------------|
| ERR-002 | Tuple destructuring | Match patterns with tuple bindings break | Skip function |
| ERR-003 | Mutable borrows | Functions returning mutable refs fail | Skip function |
| ERR-008 | Trait bounds | .into() chains break type inference | Skip function |
| ERR-009 | Self-consuming | Functions consuming self fail | Skip function |
| ERR-010 | Method indexing | .get(n) calls lose arguments | Skip function |
