# BorrowScope Battle Test: bat

**Project:** [bat](https://github.com/sharkdp/bat)  
**Version:** Latest (cloned on 2024-12-24)  
**Stars:** ~50k  
**Description:** A cat clone with syntax highlighting and Git integration  

**Note:** "Pass" means the macro transformation compiles without errors. Full runtime verification is blocked by any compilation failures.

---

## Phase 1: Reconnaissance

### Lines of Code
```
~10,947 lines total
```

### Key Modules to Test
| Module | Description | Ownership Patterns |
|--------|-------------|-------------------|
| `src/input.rs` | Input handling | File I/O, OpenedInput ownership |
| `src/output.rs` | Output handling | Handle ownership, pager integration |
| `src/printer.rs` | Line printing | Decorations, borrows, iterators |
| `src/pretty_printer.rs` | High-level API | Builder pattern, config ownership |
| `src/assets.rs` | Syntax/theme assets | Lazy loading, Rc/Arc patterns |
| `src/syntax_mapping.rs` | Syntax detection | Glob matching, mappings |
| `src/vscreen.rs` | Virtual screen | Escape sequences, iterators |
| `src/controller.rs` | Main controller | Orchestrates printing pipeline |
| `src/bin/bat/` | CLI application | Argument parsing, config handling |

---

## Error Log

### ERR-007: Type mismatch with &'static str parameter

**Location:**
- File: `src/error.rs`
- Line: 43
- Function: `impl From<&'static str> for Error { fn from(s: &'static str) -> Self }`

**Error Message:**
```
error[E0308]: mismatched types
  --> src/error.rs:43:5
   |
43 |     #[borrowscope_macro::trace_borrow]
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^- help: try using a conversion method: `.to_string()`
   |     |
   |     expected `String`, found `&str`
44 |     fn from(s: &'static str) -> Self {
45 |         Error::Msg(s.to_owned())
   |         ---------- arguments to this enum variant are incorrect
```

**Category:** Type Transformation Issue  
**Severity:** Critical  
**Component:** borrowscope-macro  
**Frequency:** TBD (first occurrence)

**Minimal Reproducer:**
```rust
#[borrowscope_macro::trace_borrow]
fn from(s: &'static str) -> Self {
    Error::Msg(s.to_owned())
}
```

**Root Cause:**
The macro transforms the `&'static str` parameter in a way that changes its type. When the function body calls `s.to_owned()`, the macro's transformation causes `s` to be tracked as `&str` instead of preserving the `&'static str` type, leading to a type mismatch when constructing `Error::Msg(String)`.

**Proposed Solution:**
- File: `borrowscope-macro/src/transform_visitor.rs`
- Change: Preserve `&'static` lifetime annotations when transforming parameters

**New Feature Required:** Yes
- Lifetime-preserving parameter transformation

**Workaround:** 
Skip `#[trace_borrow]` on functions with `&'static str` parameters that are converted to owned types.

### ERR-008: impl Into<T> parameter transformation fails

**Location:**
- File: `src/theme.rs`
- Lines: 90, 144
- Functions: `ThemePreference::new(s: impl Into<String>)`, `ThemeName::new(s: impl Into<String>)`

**Error Messages:**
```
error[E0282]: type annotations needed
  --> src/theme.rs:93:13
   |
93 |         let s = s.into();
   |             ^

error[E0277]: the trait bound `std::string::String: From<&impl Into<String>>` is not satisfied
   --> src/theme.rs:144:5
    |
144 |     #[borrowscope_macro::trace_borrow]
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `From<&impl Into<String>>` is not implemented for `std::string::String`
```

**Category:** Generic Parameter Transformation Issue  
**Severity:** Critical  
**Component:** borrowscope-macro  
**Frequency:** Common (any function with `impl Into<T>` parameters)

**Minimal Reproducer:**
```rust
#[borrowscope_macro::trace_borrow]
pub fn new(s: impl Into<String>) -> Self {
    let s = s.into();
    // ...
}
```

**Root Cause:**
The macro transforms `impl Into<String>` parameters by wrapping them with tracking calls. This changes the type from `impl Into<String>` to a tracked reference, which no longer implements `Into<String>`. When `.into()` is called, the compiler can't find the trait implementation.

**Proposed Solution:**
- File: `borrowscope-macro/src/transform_visitor.rs`
- Change: Skip tracking for parameters with `impl Trait` types, or preserve the original type

**New Feature Required:** Yes
- impl Trait parameter handling

**Workaround:** 
Skip `#[trace_borrow]` on functions with `impl Into<T>` or similar `impl Trait` parameters.

### ERR-009: Self-consuming function transformation fails

**Location:**
- File: `src/input.rs`
- Lines: 183, 188
- Functions: `Input::with_name(self, ...)`, `Input::_with_name(mut self, ...)`

**Error Messages:**
```
error[E0507]: cannot move out of a shared reference
   --> src/input.rs:183:5
    |
183 |     #[trace_borrow]
    |     ^^^^^^^^^^^^^^^ move occurs because value has type `input::Input<'_>`, which does not implement the `Copy` trait

error[E0515]: cannot return reference to function parameter `it`
   --> src/input.rs:183:5
    |
183 |     #[trace_borrow]
    |     ^^^^^^^^^^^^^^^ returns a reference to data owned by the current function
```

**Category:** Ownership Transformation Issue  
**Severity:** Critical  
**Component:** borrowscope-macro  
**Frequency:** Common (any function with `self` or `mut self` parameter)

**Minimal Reproducer:**
```rust
#[borrowscope_macro::trace_borrow]
pub fn with_name(self, name: Option<&str>) -> Self {
    // self is consumed and returned
    self
}
```

**Root Cause:**
The macro transforms `self` parameters by wrapping them with tracking calls that create references. When a function consumes `self` (takes ownership), the macro's transformation changes `self` to `&self`, preventing the ownership transfer. The function can no longer return `Self` because it only has a borrowed reference.

**Proposed Solution:**
- File: `borrowscope-macro/src/transform_visitor.rs`
- Change: Detect `self` and `mut self` parameters and either:
  1. Skip tracking for self-consuming functions entirely
  2. Track ownership transfer differently (track_move instead of track_borrow)

```rust
// In transform_visitor.rs, detect self-consuming patterns:
fn is_self_consuming(sig: &Signature) -> bool {
    sig.inputs.iter().any(|arg| {
        matches!(arg, FnArg::Receiver(r) if r.reference.is_none())
    })
}

// Skip or use track_move for these functions
if is_self_consuming(&item_fn.sig) {
    // Either skip transformation or use move tracking
    return item_fn.into_token_stream();
}
```

**Workaround:** 
Skip `#[trace_borrow]` on functions that consume `self` (builder patterns, transformation methods).

### ERR-010: Range indexing method calls fail

**Location:**
- File: `src/preprocessor.rs`
- Line: 47
- Function: `try_parse_utf8_char`

**Error Message:**
```
error[E0061]: this method takes 1 argument but 0 arguments were supplied
   --> src/preprocessor.rs:47:1
    |
47  | #[trace_borrow]
    | ^^^^^^^^^^^^^^^ argument #1 is missing
    |
note: method defined here
   --> .../core/src/slice/mod.rs:593:12
    |
593 |     pub fn get<I>(&self, index: I) -> Option<&I::Output>
    |            ^^^
```

**Category:** Method Call Transformation Issue  
**Severity:** Critical  
**Component:** borrowscope-macro  
**Frequency:** Uncommon (functions using `.get(range)` on slices)

**Minimal Reproducer:**
```rust
#[borrowscope_macro::trace_borrow]
fn try_parse_utf8_char(input: &[u8]) -> Option<(char, usize)> {
    input.get(0..1)  // Range argument gets lost in transformation
}
```

**Root Cause:**
The macro transforms the `input` parameter and wraps it with tracking calls. When `.get(0..1)` is called on the tracked reference, the range argument `0..1` is not properly passed through the transformation, resulting in a call to `.get()` with no arguments.

**Proposed Solution:**
- File: `borrowscope-macro/src/transform_visitor.rs`
- Change: Preserve method call arguments when transforming expressions with range literals

```rust
// Ensure range expressions are preserved in method calls
// When visiting MethodCall expressions, don't transform the arguments
// if they contain range expressions (ExprRange)
```

**Workaround:** 
Skip `#[trace_borrow]` on functions that use `.get(range)` or similar range-indexed method calls.

### ERR-011: Struct field access on self fails

**Location:**
- File: `src/controller.rs`
- Line: 146
- Function: `Controller::print_input`

**Error Messages:**
```
error[E0609]: no field `preprocessor` on type `&Controller<'_>`
   --> src/controller.rs:156:24
    |
156 |             match self.preprocessor {
    |                        ^^^^^^^^^^^^ unknown field

error[E0609]: no field `use_lessopen` on type `&config::Config<'_>`
   --> src/controller.rs:157:55
    |
157 |                 Some(ref preprocessor) if self.config.use_lessopen => {
    |                                                       ^^^^^^^^^^^^ unknown field
```

**Category:** Self Reference Transformation Issue  
**Severity:** Critical  
**Component:** borrowscope-macro  
**Frequency:** Common (functions accessing struct fields via self)

**Minimal Reproducer:**
```rust
struct Controller {
    preprocessor: Option<Preprocessor>,
    config: Config,
}

#[borrowscope_macro::trace_borrow]
fn print_input(&self) {
    match self.preprocessor {  // E0609: no field `preprocessor`
        Some(ref p) => {}
        None => {}
    }
}
```

**Root Cause:**
The macro transforms `&self` in a way that changes its type. When the function body accesses `self.preprocessor` or `self.config`, the compiler no longer recognizes `self` as the original struct type, causing field access to fail.

**Proposed Solution:**
- File: `borrowscope-macro/src/transform_visitor.rs`
- Change: Preserve the original type of `self` when transforming method receivers

```rust
// When transforming &self, ensure the tracked reference maintains
// the original struct type information
fn transform_self_receiver(receiver: &Receiver) -> TokenStream {
    // Don't wrap self in tracking calls that change its type
    // Instead, track at a higher level or skip self tracking
}
```

**Workaround:** 
Skip `#[trace_borrow]` on methods that access struct fields via `self`.

---

## Progress Log

### src/error.rs (3 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `From<&'static str>::from` | ❌ ERR-007 | Type mismatch with &'static str |
| `From<String>::from` | ✅ Pass | Simple conversion |
| `default_error_handler` | ✅ Pass | Match with guards |

### src/style.rs (20 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `StyleComponent::components` | ✅ Pass | Match returning static slice |
| `StyleComponent::from_str` | ✅ Pass | FromStr impl |
| `StyleComponents::new` | ✅ Pass | Constructor |
| `StyleComponents::changes` | ✅ Pass | Simple getter |
| `StyleComponents::grid` | ✅ Pass | Simple getter |
| `StyleComponents::rule` | ✅ Pass | Simple getter |
| `StyleComponents::header` | ✅ Pass | Compound getter |
| `StyleComponents::header_filename` | ✅ Pass | Simple getter |
| `StyleComponents::header_filesize` | ✅ Pass | Simple getter |
| `StyleComponents::numbers` | ✅ Pass | Simple getter |
| `StyleComponents::snip` | ✅ Pass | Simple getter |
| `StyleComponents::plain` | ✅ Pass | Iterator with closure |
| `StyleComponents::insert` | ✅ Pass | Mutable method |
| `StyleComponents::clear` | ✅ Pass | Mutable method |
| `ComponentAction::extract_from_str` | ✅ Pass | Returns tuple with &str |
| `StyleComponentList::expand_into` | ✅ Pass | Mutable HashSet |
| `StyleComponentList::contains_override` | ✅ Pass | Iterator with closure |
| `StyleComponentList::to_components` | ✅ Pass | Complex iterator fold |
| `StyleComponentList::default` | ✅ Pass | Default impl |
| `StyleComponentList::from_str` | ✅ Pass | FromStr impl |

### src/theme.rs (18 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `theme` | ✅ Pass | Simple wrapper |
| `color_scheme` | ✅ Pass | Simple wrapper |
| `ThemePreference::default` | ✅ Pass | Default impl |
| `ThemePreference::new` | ❌ ERR-008 | impl Into<String> param |
| `ThemePreference::from_str` | ✅ Pass | FromStr impl |
| `ThemePreference::fmt` | ✅ Pass | Display impl |
| `ThemeName::new` | ❌ ERR-008 | impl Into<String> param |
| `ThemeName::from_str` | ✅ Pass | FromStr impl |
| `ThemeName::fmt` | ✅ Pass | Display impl |
| `ThemeResult::fmt` | ✅ Pass | Display impl |
| `theme_impl` | ✅ Pass | Match with dyn trait |
| `choose_theme_opt` | ✅ Pass | Option handling |
| `choose_theme` | ✅ Pass | Match expression |
| `color_scheme_impl` | ✅ Pass | Match with early return |
| `TerminalColorSchemeDetector::should_detect` | ✅ Pass | Trait impl |
| `TerminalColorSchemeDetector::detect` | ✅ Pass | Trait impl |
| `color_scheme_from_system` (non-macos) | ✅ Pass | cfg-gated |
| `color_scheme_from_system` (macos) | ✅ Pass | cfg-gated |

### src/wrapping.rs (1 function)
| Function | Status | Notes |
|----------|--------|-------|
| `WrappingMode::default` | ✅ Pass | Default impl |

### src/diff.rs (1 function)
| Function | Status | Notes |
|----------|--------|-------|
| `get_git_diff` | ❌ ERR-003 | Mutable borrow issue |

### src/terminal.rs (2 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `to_ansi_color` | ✅ Pass | Complex match |
| `as_terminal_escaped` | ✅ Pass | Conditional styling |

### src/line_range.rs (12 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `LineRange::default` | ✅ Pass | Default impl |
| `LineRange::new` | ✅ Pass | Constructor |
| `LineRange::from` | ✅ Pass | Wrapper |
| `LineRange::parse_range` | ✅ Pass | Complex parsing |
| `LineRange::is_inside` | ✅ Pass | Match with tuples |
| `LineRanges::none` | ✅ Pass | Constructor |
| `LineRanges::all` | ✅ Pass | Constructor |
| `LineRanges::from` | ✅ Pass | Iterator chains |
| `LineRanges::check` | ✅ Pass | Match with guards |
| `LineRanges::largest_offset_from_end` | ✅ Pass | Simple getter |
| `LineRanges::default` | ✅ Pass | Default impl |
| `HighlightedLineRanges::default` | ✅ Pass | Default impl |

### src/config.rs (3 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `VisibleLines::diff_mode` | ✅ Pass | Match on self |
| `VisibleLines::default` | ✅ Pass | Default impl |
| `get_pager_executable` | ✅ Pass | Option chain |

### src/input.rs (17 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `InputDescription::new` | ❌ ERR-008 | impl Into<String> param |
| `InputDescription::set_kind` | ✅ Pass | Mutable setter |
| `InputDescription::set_summary` | ✅ Pass | Mutable setter |
| `InputDescription::set_title` | ✅ Pass | Mutable setter |
| `InputDescription::title` | ✅ Pass | Match returning ref |
| `InputDescription::kind` | ✅ Pass | Option getter |
| `InputDescription::summary` | ✅ Pass | Clone with fallback |
| `InputKind::description` | ✅ Pass | Match on self |
| `Input::ordinary_file` | ❌ ERR-008 | impl AsRef<Path> param |
| `Input::_ordinary_file` | ✅ Pass | Constructor |
| `Input::stdin` | ✅ Pass | Constructor |
| `Input::from_reader` | ✅ Pass | Box<dyn Read> param |
| `Input::is_stdin` | ✅ Pass | matches! macro |
| `Input::with_name` | ❌ ERR-009 | Consumes self + impl AsRef |
| `Input::_with_name` | ❌ ERR-009 | Consumes mut self |
| `Input::description` | ✅ Pass | Simple getter |
| `Input::description_mut` | ✅ Pass | Mutable getter |
| `read_utf16_line` | ✅ Pass | Generic BufRead |

### src/output.rs (7 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `BuiltinPager::new` | ✅ Pass | spawn + closure |
| `BuiltinPager::fmt` | ❌ ERR-003 | Debug impl, method chain |
| `OutputType::from_mode` | ✅ Pass | Match expression |
| `OutputType::try_pager` | ✅ Pass | Complex pager setup |
| `OutputType::handle` | ✅ Pass | Match with refs |
| `OutputType::drop` | ✅ Pass | Drop impl |
| `OutputHandle::write_fmt` | ✅ Pass | Match on self |

### src/pager.rs (2 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `PagerKind::from_bin` | ✅ Pass | Match with Path ops |
| `Pager::new` | ✅ Pass | Constructor |

### src/paging.rs (0 functions)
| Function | Status | Notes |
|----------|--------|-------|
| *No functions* | N/A | Enum only |

### src/less.rs (3 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `retrieve_less_version` | ✅ Pass | dyn AsRef param |
| `parse_less_version` | ✅ Pass | Byte slice parsing |
| `parse_less_version_busybox` | ✅ Pass | Match with guard |

### src/lessopen.rs (7 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `LessOpenPreprocessor::new` | ✅ Pass | Complex env parsing |
| `LessOpenPreprocessor::open` | ✅ Pass | Generic BufRead |
| `LessOpenPreprocessor::fall_back_to_original_file` | ✅ Pass | Simple bool |
| `LessOpenPreprocessor::mock_new` | ✅ Pass | Test helper |
| `PreprocessedKind::read` | ✅ Pass | Read impl |
| `Preprocessed::read` | ✅ Pass | Read impl |
| `Preprocessed::drop` | ✅ Pass | Drop impl |

### src/nonprintable_notation.rs (0 functions)
| Function | Status | Notes |
|----------|--------|-------|
| *No functions* | N/A | Enums only |

### src/macros.rs (0 functions)
| Function | Status | Notes |
|----------|--------|-------|
| *No functions* | N/A | Macros only |

### src/preprocessor.rs (5 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `expand_tabs` | ✅ Pass | Complex string processing |
| `try_parse_utf8_char` | ❌ ERR-010 | Range indexing .get(0..1) |
| `replace_nonprintable` | ✅ Pass | Large match expression |
| `strip_ansi` | ✅ Pass | Iterator processing |
| `strip_overstrike` | ✅ Pass | String manipulation |

### src/decorations.rs (12 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `LineNumberDecoration::new` | ✅ Pass | Constructor |
| `LineNumberDecoration::generate` | ✅ Pass | Trait impl |
| `LineNumberDecoration::width` | ✅ Pass | Simple getter |
| `LineChangesDecoration::generate_cached` | ✅ Pass | Static helper |
| `LineChangesDecoration::new` | ✅ Pass | Constructor |
| `LineChangesDecoration::generate` | ✅ Pass | Trait impl |
| `LineChangesDecoration::width` | ✅ Pass | Simple getter |
| `GridBorderDecoration::new` | ✅ Pass | Constructor |
| `GridBorderDecoration::generate` | ✅ Pass | Trait impl |
| `GridBorderDecoration::width` | ✅ Pass | Simple getter |

### src/printer.rs (22 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `SimplePrinter::new` | ✅ Pass | Constructor |
| `SimplePrinter::print_header` | ✅ Pass | Trait impl |
| `SimplePrinter::print_footer` | ✅ Pass | Trait impl |
| `SimplePrinter::print_snip` | ✅ Pass | Trait impl |
| `SimplePrinter::print_line` | ✅ Pass | Trait impl |
| `HighlighterFromSet::new` | ✅ Pass | Constructor |
| `InteractivePrinter::new` | ✅ Pass | Complex constructor |
| `InteractivePrinter::print_horizontal_line_term` | ✅ Pass | Helper |
| `InteractivePrinter::print_horizontal_line` | ✅ Pass | Helper |
| `InteractivePrinter::create_fake_panel` | ✅ Pass | String builder |
| `InteractivePrinter::get_header_component_indent_length` | ✅ Pass | Simple getter |
| `InteractivePrinter::print_header_component_indent` | ✅ Pass | Write helper |
| `InteractivePrinter::print_header_component_with_indent` | ✅ Pass | Write helper |
| `InteractivePrinter::print_header_multiline_component` | ✅ Pass | Complex write |
| `InteractivePrinter::highlight_regions_for_line` | ❌ ERR-003 | Mutable borrow |
| `InteractivePrinter::preprocess` | ✅ Pass | String processing |
| `InteractivePrinter::print_header` | ✅ Pass | Trait impl |
| `InteractivePrinter::print_footer` | ✅ Pass | Trait impl |
| `InteractivePrinter::print_snip` | ✅ Pass | Trait impl |
| `InteractivePrinter::print_line` | ✅ Pass | Trait impl |
| `Colors::plain` | ✅ Pass | Constructor |
| `Colors::colored` | ✅ Pass | Constructor |

### src/pretty_printer.rs (45 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `PrettyPrinter::new` | ✅ Pass | Constructor |
| `PrettyPrinter::input` | ✅ Pass | Builder |
| `PrettyPrinter::inputs` | ✅ Pass | Builder |
| `PrettyPrinter::input_file` | ❌ ERR-008 | impl AsRef<Path> |
| `PrettyPrinter::input_files` | ❌ ERR-008 | impl IntoIterator |
| `PrettyPrinter::input_stdin` | ✅ Pass | Builder |
| `PrettyPrinter::input_from_bytes` | ✅ Pass | Builder |
| `PrettyPrinter::input_from_reader` | ❌ ERR-008 | Generic R: Read |
| `PrettyPrinter::language` | ✅ Pass | Builder |
| `PrettyPrinter::term_width` | ✅ Pass | Builder |
| `PrettyPrinter::tab_width` | ✅ Pass | Builder |
| `PrettyPrinter::colored_output` | ✅ Pass | Builder |
| `PrettyPrinter::true_color` | ✅ Pass | Builder |
| `PrettyPrinter::header` | ✅ Pass | Builder |
| `PrettyPrinter::line_numbers` | ✅ Pass | Builder |
| `PrettyPrinter::grid` | ✅ Pass | Builder |
| `PrettyPrinter::rule` | ✅ Pass | Builder |
| `PrettyPrinter::vcs_modification_markers` | ✅ Pass | Builder |
| `PrettyPrinter::show_nonprintable` | ✅ Pass | Builder |
| `PrettyPrinter::snip` | ✅ Pass | Builder |
| `PrettyPrinter::strip_ansi` | ✅ Pass | Builder |
| `PrettyPrinter::wrapping_mode` | ✅ Pass | Builder |
| `PrettyPrinter::use_italics` | ✅ Pass | Builder |
| `PrettyPrinter::paging_mode` | ✅ Pass | Builder |
| `PrettyPrinter::pager` | ✅ Pass | Builder |
| `PrettyPrinter::line_ranges` | ✅ Pass | Builder |
| `PrettyPrinter::highlight` | ✅ Pass | Builder |
| `PrettyPrinter::highlight_range` | ✅ Pass | Builder |
| `PrettyPrinter::squeeze_empty_lines` | ✅ Pass | Builder |
| `PrettyPrinter::theme` | ✅ Pass | Builder |
| `PrettyPrinter::syntax_mapping` | ✅ Pass | Builder |
| `PrettyPrinter::themes` | ✅ Pass | Iterator |
| `PrettyPrinter::syntaxes` | ✅ Pass | Iterator |
| `PrettyPrinter::print` | ✅ Pass | Main method |
| `PrettyPrinter::print_with_writer` | ❌ ERR-003 | Generic W: Write |
| `PrettyPrinter::default` | ✅ Pass | Default impl |
| `Input::from_reader` | ✅ Pass | Constructor |
| `Input::from_file` | ✅ Pass | Constructor |
| `Input::from_bytes` | ✅ Pass | Constructor |
| `Input::from_stdin` | ✅ Pass | Constructor |
| `Input::name` | ✅ Pass | Builder (consumes self) |
| `Input::kind` | ❌ ERR-008 | impl Into<String> |
| `Input::title` | ❌ ERR-008 | impl Into<String> |
| `From<input::Input>::from` | ✅ Pass | From impl |
| `From<Input>::from` | ✅ Pass | From impl |

### src/controller.rs (6 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `Controller::new` | ✅ Pass | Constructor |
| `Controller::run` | ✅ Pass | Main entry |
| `Controller::run_with_error_handler` | ✅ Pass | impl FnMut param |
| `Controller::print_input` | ❌ ERR-011 | Field access on self |
| `Controller::print_file` | ✅ Pass | dyn Printer param |
| `Controller::print_file_ranges` | ✅ Pass | Complex loop |

### src/vscreen.rs (28 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `VirtualTerminalState::new` | ✅ Pass | Constructor |
| `VirtualTerminalState::update` | ✅ Pass | Match on sequence |
| `VirtualTerminalState::to_reset_sequence` | ✅ Pass | String builder |
| `VirtualTerminalState::fmt` | ✅ Pass | Debug impl |
| `Attributes::new` | ✅ Pass | Constructor |
| `Attributes::update` | ❌ ERR-003 | Mutable method chains |
| `Attributes::sgr_reset` | ✅ Pass | Mutable setter |
| `Attributes::update_with_sgr` | ✅ Pass | Complex parsing |
| `Attributes::update_with_unsupported` | ✅ Pass | Simple check |
| `Attributes::update_with_hyperlink` | ✅ Pass | String parsing |
| `Attributes::update_with_charset` | ✅ Pass | Iterator param |
| `Attributes::parse_color` | ✅ Pass | Static helper |
| `Attributes::to_reset_sequence` | ✅ Pass | String builder |
| `Attributes::fmt` | ✅ Pass | Debug impl |
| `join` | ❌ ERR-003 | Mutable iterator param |
| `EscapeSequenceOffsets::index_of_start` | ✅ Pass | Match on self |
| `EscapeSequenceOffsets::index_past_end` | ✅ Pass | Match on self |
| `EscapeSequenceOffsetsIterator::new` | ✅ Pass | Constructor |
| `EscapeSequenceOffsetsIterator::chars_take_while` | ✅ Pass | Closure param |
| `EscapeSequenceOffsetsIterator::next_text` | ❌ ERR-003 | Method chain |
| `EscapeSequenceOffsetsIterator::next_sequence` | ✅ Pass | Match expression |
| `EscapeSequenceOffsetsIterator::next_osc` | ✅ Pass | Complex parsing |
| `EscapeSequenceOffsetsIterator::next_csi` | ❌ ERR-003 | Method chain |
| `EscapeSequenceOffsetsIterator::next_nf` | ✅ Pass | Match expression |
| `Iterator::next (OffsetsIterator)` | ✅ Pass | Trait impl |
| `EscapeSequenceIterator::new` | ✅ Pass | Constructor |
| `Iterator::next (SequenceIterator)` | ✅ Pass | Trait impl |
| `EscapeSequence::raw` | ✅ Pass | Match on self |

### src/syntax_mapping.rs (10 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `make_glob_matcher` | ✅ Pass | Result return |
| `SyntaxMapping::drop` | ✅ Pass | Drop impl |
| `SyntaxMapping::new` | ✅ Pass | Default wrapper |
| `SyntaxMapping::start_offload_build_all` | ✅ Pass | Thread spawn |
| `SyntaxMapping::insert` | ✅ Pass | Mutable method |
| `SyntaxMapping::all_mappings` | ✅ Pass | impl Iterator return |
| `SyntaxMapping::builtin_mappings` | ✅ Pass | impl Iterator return |
| `SyntaxMapping::custom_mappings` | ✅ Pass | Slice return |
| `SyntaxMapping::get_syntax_for` | ✅ Pass | impl AsRef param |
| `SyntaxMapping::insert_ignored_suffix` | ✅ Pass | Mutable method |

### src/syntax_mapping/builtin.rs (2 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `build_matcher_fixed` | ✅ Pass | Simple wrapper |
| `build_matcher_dynamic` | ✅ Pass | Option return |

### src/syntax_mapping/ignored_suffixes.rs (4 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `IgnoredSuffixes::default` | ✅ Pass | Default impl |
| `IgnoredSuffixes::add_suffix` | ✅ Pass | Mutable method |
| `IgnoredSuffixes::strip_suffix` | ✅ Pass | Option return |
| `IgnoredSuffixes::try_with_stripped_suffix` | ✅ Pass | Generic closure param |

### src/assets.rs (20 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `HighlightingAssets::new` | ✅ Pass | Constructor |
| `HighlightingAssets::from_cache` | ✅ Pass | Result return |
| `HighlightingAssets::from_binary` | ✅ Pass | Constructor |
| `HighlightingAssets::set_fallback_theme` | ✅ Pass | Mutable setter |
| `HighlightingAssets::get_syntax_set` | ✅ Pass | OnceCell init |
| `HighlightingAssets::syntaxes` | ✅ Pass | Deprecated wrapper |
| `HighlightingAssets::get_syntaxes` | ✅ Pass | Result wrapper |
| `HighlightingAssets::get_theme_set` | ✅ Pass | Simple getter |
| `HighlightingAssets::themes` | ✅ Pass | impl Iterator return |
| `HighlightingAssets::syntax_for_file_name` | ✅ Pass | impl AsRef param |
| `HighlightingAssets::get_syntax_for_path` | ✅ Pass | impl AsRef param |
| `HighlightingAssets::get_theme` | ✅ Pass | Match with fallback |
| `HighlightingAssets::get_syntax` | ✅ Pass | Complex detection |
| `HighlightingAssets::find_syntax_by_name` | ✅ Pass | Option return |
| `HighlightingAssets::find_syntax_by_extension` | ✅ Pass | Option return |
| `HighlightingAssets::find_syntax_by_token` | ✅ Pass | Option return |
| `HighlightingAssets::get_syntax_for_file_name` | ✅ Pass | Recursive |
| `HighlightingAssets::get_syntax_for_file_extension` | ✅ Pass | Recursive |
| `HighlightingAssets::get_first_line_syntax` | ✅ Pass | Reader param |
| `get_serialized_integrated_syntaxset` | ✅ Pass | Static return |
| `get_integrated_themeset` | ✅ Pass | Binary load |
| `get_acknowledgements` | ✅ Pass | Binary load |
| `from_binary` | ✅ Pass | Generic deserialize |
| `asset_from_contents` | ✅ Pass | Generic deserialize |
| `asset_from_cache` | ✅ Pass | File read + deserialize |

### src/assets/assets_metadata.rs (5 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `AssetsMetadata::new` | ✅ Pass | Constructor (cfg-gated) |
| `AssetsMetadata::save_to_folder` | ✅ Pass | File write (cfg-gated) |
| `AssetsMetadata::try_load_from_folder` | ✅ Pass | File read |
| `AssetsMetadata::load_from_folder` | ✅ Pass | Match with fallback |
| `AssetsMetadata::is_compatible_with` | ✅ Pass | Version comparison |

### src/assets/build_assets.rs (7 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `build` | ❌ ERR-009 | E0507 move out of shared ref |
| `build_theme_set` | ❌ ERR-008 | TryFrom type mismatch |
| `build_syntax_set_builder` | ✅ Pass | Complex builder |
| `print_unlinked_contexts` | ✅ Pass | Simple iteration |
| `write_assets` | ✅ Pass | Multiple file writes |
| `asset_to_contents` | ✅ Pass | Generic serialize |
| `asset_to_cache` | ✅ Pass | File write |

### src/assets/build_assets/acknowledgements.rs (10 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `build_acknowledgements` | ✅ Pass | Complex iteration |
| `to_path_and_stem` | ✅ Pass | Option return |
| `handle_file` | ✅ Pass | Match expression |
| `handle_notice` | ✅ Pass | File read |
| `handle_license` | ✅ Pass | Conditional logic |
| `include_license_in_acknowledgments` | ✅ Pass | Marker check |
| `license_not_needed_in_acknowledgements` | ✅ Pass | Marker check |
| `license_contains_marker` | ✅ Pass | Iterator any |
| `append_to_acknowledgements` | ❌ ERR-003 | &mut String param |
| `normalize_license_text` | ✅ Pass | Regex processing |

### src/assets/lazy_theme_set.rs (5 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `LazyThemeSet::get` | ✅ Pass | OnceCell init |
| `LazyThemeSet::themes` | ❌ ERR-009 | E0515 return ref to param |
| `LazyTheme::deserialize` | ✅ Pass | Binary deserialize |
| `TryFrom<LazyThemeSet>::try_from` | ✅ Pass | Conversion |
| `TryFrom<ThemeSet>::try_from` | ✅ Pass | Conversion (cfg-gated) |

### src/assets/serialized_syntax_set.rs (1 function)
| Function | Status | Notes |
|----------|--------|-------|
| `SerializedSyntaxSet::deserialize` | ✅ Pass | Match on self |

### src/lib.rs (0 functions)
| Function | Status | Notes |
|----------|--------|-------|
| *No functions* | N/A | Module declarations and re-exports only |

### src/bin/bat/main.rs (12 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `build_assets` | ✅ Pass | cfg-gated |
| `run_cache_subcommand` | ✅ Pass | Match with cfg |
| `get_syntax_mapping_to_paths` | ✅ Pass | Generic iterator |
| `get_languages` | ✅ Pass | Complex iteration |
| `theme_preview_file` | ✅ Pass | Input constructor |
| `list_themes` | ✅ Pass | Theme iteration |
| `set_terminal_title_to` | ✅ Pass | Print + flush |
| `get_new_terminal_title` | ✅ Pass | String building |
| `run_controller` | ✅ Pass | Controller setup |
| `invoke_bugreport` | ✅ Pass | cfg-gated |
| `run` | ✅ Pass | Main logic |
| `main` | ✅ Pass | Entry point |

### src/bin/bat/app.rs (12 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `is_truecolor_terminal` | ✅ Pass | Env check |
| `env_no_color` | ✅ Pass | Env check |
| `App::new` | ✅ Pass | Complex constructor |
| `App::display_help` | ✅ Pass | Help rendering |
| `App::build_args_without_config` | ✅ Pass | Arg building |
| `App::matches` | ✅ Pass | Clap parsing |
| `App::config` | ✅ Pass | Config building |
| `App::inputs` | ✅ Pass | Input collection |
| `App::forced_style_components` | ✅ Pass | Option return |
| `App::style_components` | ✅ Pass | Style parsing |
| `App::theme_options` | ✅ Pass | Simple wrapper |
| `App::theme_options_from_matches` | ✅ Pass | Theme parsing |

### src/bin/bat/clap_app.rs (1 function)
| Function | Status | Notes |
|----------|--------|-------|
| `build_app` | ✅ Pass | Clap Command builder |

### src/bin/bat/config.rs (7 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `system_config_file` | ✅ Pass | PathBuf return |
| `config_file` | ✅ Pass | Env + fallback |
| `generate_config_file` | ✅ Pass | File write |
| `get_args_from_config_file` | ✅ Pass | File read + parse |
| `get_args_from_env_opts_var` | ✅ Pass | Env var parse |
| `get_args_from_str` | ✅ Pass | String parsing |
| `get_args_from_env_vars` | ✅ Pass | Env iteration |

### src/bin/bat/directories.rs (3 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `BatProjectDirs::new` | ✅ Pass | Constructor |
| `BatProjectDirs::cache_dir` | ✅ Pass | Simple getter |
| `BatProjectDirs::config_dir` | ✅ Pass | Simple getter |

### src/bin/bat/input.rs (3 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `new_file_input` | ✅ Pass | Input constructor |
| `new_stdin_input` | ✅ Pass | Input constructor |
| `named` | ✅ Pass | Input wrapper |

### src/bin/bat/assets.rs (3 functions)
| Function | Status | Notes |
|----------|--------|-------|
| `clear_assets` | ✅ Pass | File cleanup |
| `assets_from_cache_or_binary` | ✅ Pass | Asset loading |
| `clear_asset` | ✅ Pass | Single file cleanup |

### src/bin/bat/completions.rs (0 functions)
| Function | Status | Notes |
|----------|--------|-------|
| *No functions* | N/A | Constants only |

---

## Summary (In Progress)

### Test Results by File

| File | Pass | Fail | Total | Pass Rate |
|------|------|------|-------|-----------|
| error.rs | 2 | 1 | 3 | 67% |
| style.rs | 20 | 0 | 20 | 100% |
| theme.rs | 16 | 2 | 18 | 89% |
| wrapping.rs | 1 | 0 | 1 | 100% |
| diff.rs | 0 | 1 | 1 | 0% |
| terminal.rs | 2 | 0 | 2 | 100% |
| line_range.rs | 12 | 0 | 12 | 100% |
| config.rs | 3 | 0 | 3 | 100% |
| input.rs | 14 | 4 | 18 | 78% |
| output.rs | 6 | 1 | 7 | 86% |
| pager.rs | 2 | 0 | 2 | 100% |
| paging.rs | 0 | 0 | 0 | N/A |
| less.rs | 3 | 0 | 3 | 100% |
| lessopen.rs | 7 | 0 | 7 | 100% |
| nonprintable_notation.rs | 0 | 0 | 0 | N/A |
| macros.rs | 0 | 0 | 0 | N/A |
| preprocessor.rs | 4 | 1 | 5 | 80% |
| decorations.rs | 10 | 0 | 10 | 100% |
| printer.rs | 21 | 1 | 22 | 95% |
| pretty_printer.rs | 39 | 6 | 45 | 87% |
| controller.rs | 5 | 1 | 6 | 83% |
| vscreen.rs | 24 | 4 | 28 | 86% |
| syntax_mapping.rs | 10 | 0 | 10 | 100% |
| syntax_mapping/builtin.rs | 2 | 0 | 2 | 100% |
| syntax_mapping/ignored_suffixes.rs | 4 | 0 | 4 | 100% |
| assets.rs | 25 | 0 | 25 | 100% |
| assets/assets_metadata.rs | 5 | 0 | 5 | 100% |
| assets/build_assets.rs | 5 | 2 | 7 | 71% |
| assets/build_assets/acknowledgements.rs | 9 | 1 | 10 | 90% |
| assets/lazy_theme_set.rs | 4 | 1 | 5 | 80% |
| assets/serialized_syntax_set.rs | 1 | 0 | 1 | 100% |
| lib.rs | 0 | 0 | 0 | N/A |
| bin/bat/main.rs | 12 | 0 | 12 | 100% |
| bin/bat/app.rs | 12 | 0 | 12 | 100% |
| bin/bat/clap_app.rs | 1 | 0 | 1 | 100% |
| bin/bat/config.rs | 7 | 0 | 7 | 100% |
| bin/bat/directories.rs | 3 | 0 | 3 | 100% |
| bin/bat/input.rs | 3 | 0 | 3 | 100% |
| bin/bat/assets.rs | 3 | 0 | 3 | 100% |
| bin/bat/completions.rs | 0 | 0 | 0 | N/A |
| **TOTAL** | **297** | **26** | **323** | **92%** |

### Patterns Tested

| Pattern | Status | Notes |
|---------|--------|-------|
| Basic let bindings | ✅ | Works |
| Immutable borrows | ✅ | Works |
| Mutable borrows | ⚠️ | Fails on method chains (ERR-003) |
| impl Into<T> parameters | ❌ | ERR-008 |
| impl AsRef<T> parameters | ❌ | ERR-008 |
| Builder patterns | ⚠️ | Self-consuming methods fail (ERR-009) |
| Iterator chains | ✅ | Works |
| Result<T> returns | ✅ | Works |
| Option<T> handling | ✅ | Works |
| Struct field access | ⚠️ | Fails on &mut self methods (ERR-011) |
| Range indexing | ❌ | ERR-010 |
| &'static str params | ❌ | ERR-007 |

### Gaps Identified

#### Critical (Blocks Usage)
| ID | Gap | Description | Workaround |
|----|-----|-------------|------------|
| ERR-003 | Mutable method chains | track_borrow returns &T, but method needs &mut T | Skip builder patterns |
| ERR-007 | &'static str type mismatch | Macro transforms &'static str incorrectly | Skip these functions |
| ERR-008 | impl Into<T> fails | Macro breaks impl Trait parameter types | Skip these functions |
| ERR-009 | Self-consuming functions | Macro wraps self in borrow, breaks ownership | Skip builder/transform methods |
| ERR-010 | Range indexing fails | Macro breaks .get(range) method calls | Skip functions with range indexing |
| ERR-011 | Struct field access fails | Macro changes self type, breaks field access | Skip methods accessing self fields |
