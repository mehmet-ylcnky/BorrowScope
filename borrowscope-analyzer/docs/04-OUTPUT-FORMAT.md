## 4. Output Format

The analyzer produces a JSON file containing type information for all variable bindings in the project. This file is written to `.borrowscope/type-info.json` relative to the project root, creating the directory if it does not exist.

### Schema Version 2.5 (Current)

**New in v2.5:**
- `method_calls` array on each variable
- `expressions` top-level field for standalone function calls
- Fully semantic operation paths (e.g., `core::cell::set` instead of `cell_set`)

### Quick Reference

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
        "line": 10,
        "column": 4,
        "operation": "core::cell::set",
        "self_borrow": "immutable",
        "receiver_type": "Cell<i32>",
        "result_type": "()"
      }]
    }]
  },
  "expressions": {
    "src/main.rs": [{
      "line": 15,
      "column": 4,
      "kind": "function_call",
      "path": "core::mem::drop",
      "operation": "core::mem::drop",
      "argument": "x",
      "result_type": "()"
    }]
  }
}
```

### MethodCallInfo Structure (v2.4+)

| Field | Type | Description |
|-------|------|-------------|
| `method` | string | Method name (e.g., "set", "clone", "join") |
| `line` | u32 | Line number of the call |
| `column` | u32 | Column number |
| `operation` | string | Semantic path (e.g., "core::cell::set") |
| `self_borrow` | string | "immutable", "mutable", or "consuming" |
| `receiver_type` | string | Fully qualified receiver type |
| `result_type` | string | Return type of the method |

### ExpressionInfo Structure (v2.5+)

| Field | Type | Description |
|-------|------|-------------|
| `line` | u32 | Line number |
| `column` | u32 | Column number |
| `kind` | string | Always "function_call" |
| `path` | string | Canonical function path |
| `operation` | string | Same as path (semantic) |
| `argument` | string | Variable name or closure info |
| `result_type` | string | Return type |

### Tracked Standalone Functions

| Function | Example Operation Path |
|----------|----------------------|
| drop | `core::mem::drop` |
| forget | `core::mem::forget` |
| transmute | `core::intrinsics::transmute` |
| transmute_copy | `core::mem::transmute_copy` |
| replace | `core::mem::replace` |
| swap | `core::mem::swap` |
| take | `core::mem::take` |
| ptr::read | `core::ptr::read` |
| ptr::write | `core::ptr::write` |
| ptr::copy | `core::intrinsics::copy` |
| ptr::copy_nonoverlapping | `core::intrinsics::copy_nonoverlapping` |

### Closure Traits (v2.5+)

The `closure_traits` field tracks which `Fn*` trait closures implement:

```json
{
  "closure_traits": {
    "src/main.rs": [
      {"line": 4, "column": 13, "fn_trait": "Fn"},
      {"line": 9, "column": 17, "fn_trait": "AsyncFn"},
      {"line": 10, "column": 18, "fn_trait": "AsyncFnMut"}
    ]
  }
}
```

| Trait | Description |
|-------|-------------|
| `Fn` | Closure that borrows captured variables immutably |
| `FnMut` | Closure that borrows captured variables mutably |
| `FnOnce` | Closure that consumes captured variables |
| `AsyncFn` | Async closure with immutable borrows (requires ra_ap_* 0.0.318+) |
| `AsyncFnMut` | Async closure with mutable borrows (requires ra_ap_* 0.0.318+) |
| `AsyncFnOnce` | Async closure that consumes captures (requires ra_ap_* 0.0.318+) |

---

### Full Schema Structure

The output follows a hierarchical structure with project-level metadata and per-file variable information:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         type-info.json SCHEMA (v2.3)                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  {                                                                          │
│    "version": "2.3",              ◄─── Schema version for compatibility     │
│    "analyzer_version": "0.1.0",   ◄─── Analyzer binary version              │
│    "files": {                     ◄─── Map: relative path → variables       │
│      "src/main.rs": [                                                       │
│        {                                                                    │
│          "name": "data",          ◄─── Variable name from source            │
│          "ty": "Rc<RefCell<Vec<i32>>>",  ◄─── Fully resolved type          │
│                                                                             │
│          // === Trait implementations (semantic via impls_trait) ===        │
│          "is_copy": false,        ◄─── Copy trait (ty.is_copy)              │
│          "is_clone": true,        ◄─── Clone trait (impls_trait)            │
│          "is_send": false,        ◄─── Send trait (import_map lookup)       │
│          "is_sync": false,        ◄─── Sync trait (impls_trait)             │
│          "is_drop": true,         ◄─── Drop trait (impls_trait)             │
│          "is_sized": true,        ◄─── Sized trait (impls_trait)            │
│          "is_future": false,      ◄─── Future trait (impls_trait)           │
│          "is_iterator": false,    ◄─── Iterator trait (impls_trait)         │
│                                                                             │
│          // === Type structure (semantic via Type methods) ===              │
│          "is_primitive": false,   ◄─── i32, bool, char, etc. (as_builtin)   │
│          "is_reference": false,   ◄─── &T or &mut T (is_reference)          │
│          "is_mutable_reference": false,  ◄─── &mut T (is_mutable_reference) │
│          "is_raw_ptr": false,     ◄─── *const T or *mut T (is_raw_ptr)      │
│          "is_slice": false,       ◄─── [T], &[T], Box<[T]> (is_slice)       │
│          "is_str": false,         ◄─── str type (as_builtin.is_str)         │
│          "is_closure": false,     ◄─── Closure type (is_closure)            │
│          "is_fn_ptr": false,      ◄─── fn(...) -> ... (is_fn)               │
│          "is_dyn_trait": false,   ◄─── dyn Trait, &dyn T, Box<dyn T>        │
│          "is_union": false,       ◄─── Union type (as_adt + Adt::Union)     │
│                                                                             │
│          // === ADT classification (semantic via canonical path) ===        │
│          "is_rc": true,           ◄─── alloc::rc::Rc                        │
│          "is_arc": false,         ◄─── alloc::sync::Arc                     │
│          "is_box": false,         ◄─── alloc::boxed::Box                    │
│          "is_weak": false,        ◄─── alloc::rc::Weak or alloc::sync::Weak │
│          "is_refcell": true,      ◄─── core::cell::RefCell                  │
│          "is_cell": false,        ◄─── core::cell::Cell                     │
│          "is_mutex": false,       ◄─── std::sync::Mutex                     │
│          "is_rwlock": false,      ◄─── std::sync::RwLock                    │
│          "is_guard": false,       ◄─── MutexGuard, Ref, RefMut, etc.        │
│          "is_vec": true,          ◄─── alloc::vec::Vec                      │
│          "is_string": false,      ◄─── alloc::string::String                │
│          "is_option": false,      ◄─── core::option::Option                 │
│          "is_result": false,      ◄─── core::result::Result                 │
│          "is_pin": false,         ◄─── core::pin::Pin                       │
│          "is_cow": false,         ◄─── alloc::borrow::Cow                   │
│          "is_once_cell": false,   ◄─── core::cell::OnceCell (v2.1+)         │
│          "is_maybe_uninit": false,◄─── core::mem::MaybeUninit (v2.1+)       │
│          "is_channel": false,     ◄─── mpsc::Sender/Receiver (v2.1+)        │
│          "is_extern_type": false, ◄─── c_void, CStr, CString, OsStr, etc.   │
│                                                                             │
│          // === Declaration type ===                                        │
│          "is_static": false,      ◄─── static declaration                   │
│          "is_const": false,       ◄─── const declaration                    │
│                                                                             │
│          // === Binding patterns for macro transformation ===               │
│          "is_tuple_binding": false,  ◄─── let (a, b) = ...                  │
│          "is_mut_binding": false,    ◄─── let mut x = ...                   │
│          "is_impl_trait": false,     ◄─── impl Trait type                   │
│                                                                             │
│          // === Initializer pattern (v2.1+) ===                             │
│          "initializer_kind": "rc_new",  ◄─── Semantic init pattern          │
│                                                                             │
│          // === Source location ===                                         │
│          "file": "src/main.rs",                                             │
│          "line": 15,                                                        │
│          "column": 8,                                                       │
│          "span_start": 1234,      ◄─── Byte offset start                    │
│          "span_end": 1238,        ◄─── Byte offset end                      │
│                                                                             │
│          // === Disambiguation (v2.2+) ===                                  │
│          "scope_id": 5,           ◄─── Scope identifier                     │
│          "function_name": "example",  ◄─── Containing function name         │
│          "decl_index": 2          ◄─── Declaration order in function        │
│        },                                                                   │
│        ...                                                                  │
│      ]                                                                      │
│    },                                                                       │
│    "by_name": {                   ◄─── Index by variable name (v2.1+)       │
│      "data": [ ... ],             ◄─── All variables named "data"           │
│      ...                                                                    │
│    },                                                                       │
│    "by_function": {               ◄─── Index by function+name (v2.2+)       │
│      "example": {                 ◄─── Function name                        │
│        "data": [ ... ],           ◄─── Variables in that function           │
│        ...                                                                  │
│      },                                                                     │
│      ...                                                                    │
│    }                                                                        │
│  }                                                                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Field Descriptions

The `VariableTypeInfo` structure captures comprehensive type metadata for each variable binding:

**Identity Fields**

The `name` field contains the variable name exactly as it appears in source code. For simple bindings like `let x = ...`, this is `"x"`. For pattern bindings like `let (a, b) = ...`, the current implementation captures the entire pattern text; future versions may decompose patterns into individual bindings.

The `file`, `line`, and `column` fields provide the precise source location of the binding. Line numbers are 1-indexed to match editor conventions. Column numbers indicate the start of the pattern within the line. These coordinates enable the macro to look up type information by source location.

**Type String**

The `ty` field contains the fully resolved type as a string. This includes generic parameters with their concrete types and any allocator parameters. Examples:

| Source Expression | Resolved Type |
|-------------------|---------------|
| `let x = 42;` | `i32` |
| `let s = String::from("hello");` | `String` |
| `let v = vec![1, 2, 3];` | `Vec<i32, Global>` |
| `let rc = Rc::new(value);` | `Rc<MyStruct, Global>` |
| `let nested = Rc::new(RefCell::new(vec![]));` | `Rc<RefCell<Vec<i32, Global>>, Global>` |
| `let ptr: *const i32 = &x;` | `*const i32` |
| `let closure = \|x\| x + 1;` | `impl Fn(i32) -> i32` |
| `let guard = mutex.lock().unwrap();` | `MutexGuard<'_, i32>` |
| `let future = async { 42 };` | `impl Future<Output = i32>` |

The `Global` allocator parameter appears because rust-analyzer displays the full type including default generic parameters. This verbosity is intentional—it provides complete type information without ambiguity.

**Classification Flags**

Boolean flags provide quick classification using semantic analysis (no string parsing):

| Flag | Detection Method | Purpose |
|------|------------------|---------|
| `is_copy` | `ty.is_copy(db)` | Copy trait - determines move vs copy semantics |
| `is_clone` | `ty.impls_trait(Clone)` | Clone trait implementation |
| `is_send` | `ty.impls_trait(Send)` via import_map | Thread-safe ownership transfer |
| `is_sync` | `ty.impls_trait(Sync)` | Thread-safe shared reference |
| `is_drop` | `ty.impls_trait(Drop)` | Custom destructor |
| `is_sized` | `ty.impls_trait(Sized)` | Compile-time known size |
| `is_future` | `ty.impls_trait(Future)` | Async/Future type |
| `is_iterator` | `ty.impls_trait(Iterator)` | Iterator type |
| `is_primitive` | `ty.as_builtin()` | i32, bool, char, f64, etc. |
| `is_reference` | `ty.is_reference()` | &T or &mut T |
| `is_mutable_reference` | `ty.is_mutable_reference()` | &mut T |
| `is_raw_ptr` | `ty.is_raw_ptr()` | *const T or *mut T |
| `is_slice` | `ty.is_slice()` + inner type check | [T], &[T], Box<[T]> |
| `is_str` | `ty.as_builtin().is_str()` | str type |
| `is_closure` | `ty.is_closure()` | Closure type |
| `is_fn_ptr` | `ty.is_fn()` | fn(...) -> ... |
| `is_dyn_trait` | `ty.as_dyn_trait()` + inner type check | dyn Trait, &dyn T, Box<dyn T> |
| `is_union` | `ty.as_adt()` + `Adt::Union` | Union types including MaybeUninit |
| `is_rc` | ADT path == `alloc::rc::Rc` | Reference-counted pointer |
| `is_arc` | ADT path == `alloc::sync::Arc` | Atomic reference-counted pointer |
| `is_box` | ADT path == `alloc::boxed::Box` | Heap allocation |
| `is_weak` | ADT path == `alloc::rc::Weak` or `alloc::sync::Weak` | Weak reference |
| `is_refcell` | ADT path == `core::cell::RefCell` | Runtime borrow checking |
| `is_cell` | ADT path == `core::cell::Cell` | Copy-based interior mutability |
| `is_mutex` | ADT path == `std::sync::Mutex` | Thread-safe lock |
| `is_rwlock` | ADT path == `std::sync::RwLock` | Reader-writer lock |
| `is_guard` | ADT path matches guard types | MutexGuard, Ref, RefMut, etc. |
| `is_vec` | ADT path == `alloc::vec::Vec` | Dynamic array |
| `is_string` | ADT path == `alloc::string::String` | Owned string |
| `is_option` | ADT path == `core::option::Option` | Optional value |
| `is_result` | ADT path == `core::result::Result` | Result type |
| `is_pin` | ADT path == `core::pin::Pin` | Pinned pointer |
| `is_cow` | ADT path == `alloc::borrow::Cow` | Clone-on-write |
| `is_once_cell` | ADT path == `core::cell::OnceCell` or `std::sync::OnceLock` | Lazy initialization |
| `is_maybe_uninit` | ADT path == `core::mem::MaybeUninit` | Uninitialized memory |
| `is_channel` | ADT path matches mpsc Sender/Receiver | Channel endpoints |
| `is_extern_type` | ADT path matches FFI types | c_void, CStr, CString, OsStr, etc. |
| `is_static` | Declaration syntax | static declaration |
| `is_const` | Declaration syntax | const declaration |
| `is_tuple_binding` | Pattern syntax | let (a, b) = ... |
| `is_mut_binding` | Pattern syntax | let mut x = ... |
| `is_impl_trait` | Type annotation syntax | impl Trait type |

**Initializer Pattern (v2.1+)**

The `initializer_kind` field captures the semantic pattern of the variable's initializer expression. This enables the macro to select the most appropriate tracking function based on how the variable was created, not just its type. The analyzer uses fully semantic type resolution to classify initializers into 78 distinct categories.

#### Expression-Level Classification

The top-level expression type determines the initial classification:

| Expression Type | `initializer_kind` | Example | Description |
|-----------------|-------------------|---------|-------------|
| `Literal` | `literal` | `let x = 42;` | Numeric, string, bool, char literals |
| `CallExpr` | (see Function Calls) | `let x = foo();` | Function or constructor call |
| `MethodCallExpr` | (see Method Calls) | `let x = y.bar();` | Method invocation |
| `BlockExpr` | `block` | `let x = { ... };` | Block expression |
| `IfExpr` | `if` | `let x = if c { a } else { b };` | Conditional expression |
| `MatchExpr` | `match` | `let x = match v { ... };` | Pattern matching |
| `ClosureExpr` | `closure` | `let f = \|x\| x + 1;` | Closure definition |
| `RefExpr` | `ref` / `ref_mut` | `let r = &x;` / `let r = &mut x;` | Reference creation |
| `PathExpr` | `path` | `let x = some_var;` | Variable or constant reference |
| `MacroExpr` | (see Macros) | `let v = vec![1,2,3];` | Macro invocation |
| `AwaitExpr` | `await` | `let x = fut.await;` | Async await |
| `TryExpr` | `try` | `let x = fallible()?;` | Try operator |
| `TupleExpr` | `tuple` | `let t = (1, 2, 3);` | Tuple construction |
| `ArrayExpr` | `array` | `let a = [1, 2, 3];` | Array construction |
| `IndexExpr` | `index` | `let x = arr[0];` | Index operation |
| `FieldExpr` | `field` | `let x = s.field;` | Field access |
| `CastExpr` | `cast` | `let x = y as i32;` | Type cast |
| `RecordExpr` | `struct_literal` | `let s = Struct { ... };` | Struct literal |
| `RangeExpr` | `range` | `let r = 0..10;` | Range expression |
| `BinExpr` | `binary` | `let x = a + b;` | Binary operation |
| `PrefixExpr` (deref) | `deref` | `let x = *ptr;` | Dereference |
| `PrefixExpr` (not) | `not` | `let x = !flag;` | Logical not |
| `PrefixExpr` (neg) | `neg` | `let x = -val;` | Negation |
| `LoopExpr` | `loop` | `let x = loop { break 42; };` | Loop expression |
| `WhileExpr` | `while` | `let x = while c { ... };` | While loop |
| `ForExpr` | `for` | `let x = for i in iter { ... };` | For loop |
| `ReturnExpr` | `return` | `let x = return;` | Return expression |
| `BreakExpr` | `break` | `let x = break;` | Break expression |
| `ContinueExpr` | `continue` | `let x = continue;` | Continue expression |
| `YieldExpr` | `yield` | `let x = yield val;` | Generator yield |
| `YeetExpr` | `yeet` | `let x = yeet err;` | Yeet expression |
| `AsmExpr` | `asm` | `let x = asm!(...);` | Inline assembly |
| `FormatArgsExpr` | `format_args` | `let x = format_args!(...);` | Format arguments |
| `OffsetOfExpr` | `offset_of` | `let x = offset_of!(...);` | Offset of field |

#### Function Call Classification

When the initializer is a function call (`CallExpr`), the analyzer examines the callee path to identify specific patterns:

##### Smart Pointer Constructors

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Rc creation | `rc_new` | `Rc::new`, `std::rc::Rc::new`, `alloc::rc::Rc::new` |
| Arc creation | `arc_new` | `Arc::new`, `std::sync::Arc::new`, `alloc::sync::Arc::new` |
| Box creation | `box_new` | `Box::new`, `std::boxed::Box::new`, `alloc::boxed::Box::new` |
| Box::pin | `box_pin` | `Box::pin`, `std::boxed::Box::pin` |
| Rc clone | `rc_clone` | `Rc::clone`, `std::rc::Rc::clone` |
| Arc clone | `arc_clone` | `Arc::clone`, `std::sync::Arc::clone` |
| Weak creation | `weak_new` | `Weak::new`, `std::rc::Weak::new`, `std::sync::Weak::new` |

##### Interior Mutability Constructors

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| RefCell creation | `refcell_new` | `RefCell::new`, `std::cell::RefCell::new`, `core::cell::RefCell::new` |
| Cell creation | `cell_new` | `Cell::new`, `std::cell::Cell::new`, `core::cell::Cell::new` |
| Mutex creation | `mutex_new` | `Mutex::new`, `std::sync::Mutex::new` |
| RwLock creation | `rwlock_new` | `RwLock::new`, `std::sync::RwLock::new` |

##### Lazy Initialization

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| OnceCell creation | `once_cell_new` | `OnceCell::new`, `std::cell::OnceCell::new`, `core::cell::OnceCell::new` |
| OnceLock creation | `once_lock_new` | `OnceLock::new`, `std::sync::OnceLock::new` |

##### Uninitialized Memory

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| MaybeUninit uninit | `maybe_uninit_uninit` | `MaybeUninit::uninit`, `std::mem::MaybeUninit::uninit`, `core::mem::MaybeUninit::uninit` |
| MaybeUninit new | `maybe_uninit_new` | `MaybeUninit::new`, `std::mem::MaybeUninit::new`, `core::mem::MaybeUninit::new` |
| MaybeUninit zeroed | `maybe_uninit_zeroed` | `MaybeUninit::zeroed`, `std::mem::MaybeUninit::zeroed`, `core::mem::MaybeUninit::zeroed` |

##### Channels

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Channel creation | `channel_new` | `channel`, `std::sync::mpsc::channel` |
| Sync channel | `sync_channel_new` | `sync_channel`, `std::sync::mpsc::sync_channel` |

##### Pin

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Pin creation | `pin_new` | `Pin::new`, `std::pin::Pin::new`, `core::pin::Pin::new` |
| Pin unchecked | `pin_new_unchecked` | `Pin::new_unchecked`, `std::pin::Pin::new_unchecked` |

##### Cow (Clone-on-Write)

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Cow borrowed | `cow_borrowed` | `Cow::Borrowed`, `std::borrow::Cow::Borrowed` |
| Cow owned | `cow_owned` | `Cow::Owned`, `std::borrow::Cow::Owned` |

##### Option/Result Constructors

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Some variant | `option_some` | `Some`, `core::option::Option::Some`, `std::option::Option::Some` |
| None variant (call) | `option_none` | `None()`, `core::option::Option::None()` (as function call) |
| None variant (path) | `none` | `None`, `Option::None` (as path expression) |
| Ok variant | `result_ok` | `Ok`, `core::result::Result::Ok`, `std::result::Result::Ok` |
| Err variant | `result_err` | `Err`, `core::result::Result::Err`, `std::result::Result::Err` |

##### String Constructors

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| String::new | `string_new` | `String::new`, `std::string::String::new`, `alloc::string::String::new` |
| String::from | `string_from` | `String::from`, `std::string::String::from`, `alloc::string::String::from` |
| String::with_capacity | `string_with_capacity` | `String::with_capacity` |

##### Vec Constructors

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Vec::new | `vec_new` | `Vec::new`, `std::vec::Vec::new`, `alloc::vec::Vec::new` |
| Vec::with_capacity | `vec_with_capacity` | `Vec::with_capacity`, `std::vec::Vec::with_capacity` |

##### Collection Constructors

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| HashMap::new | `hashmap_new` | `HashMap::new`, `std::collections::HashMap::new` |
| HashSet::new | `hashset_new` | `HashSet::new`, `std::collections::HashSet::new` |
| BTreeMap::new | `btreemap_new` | `BTreeMap::new`, `std::collections::BTreeMap::new` |
| BTreeSet::new | `btreeset_new` | `BTreeSet::new`, `std::collections::BTreeSet::new` |
| VecDeque::new | `vecdeque_new` | `VecDeque::new`, `std::collections::VecDeque::new` |
| LinkedList::new | `linkedlist_new` | `LinkedList::new`, `std::collections::LinkedList::new` |
| BinaryHeap::new | `binaryheap_new` | `BinaryHeap::new`, `std::collections::BinaryHeap::new` |

##### Path and FFI Constructors

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| PathBuf::new | `pathbuf_new` | `PathBuf::new`, `std::path::PathBuf::new` |
| PathBuf::from | `pathbuf_from` | `PathBuf::from`, `std::path::PathBuf::from` |
| OsString::new | `osstring_new` | `OsString::new`, `std::ffi::OsString::new` |
| OsString::from | `osstring_from` | `OsString::from`, `std::ffi::OsString::from` |
| CString::new | `cstring_new` | `CString::new`, `std::ffi::CString::new` |

##### Raw Pointer Constructors

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Null pointer | `ptr_null` | `ptr::null`, `std::ptr::null`, `core::ptr::null` |
| Null mut pointer | `ptr_null_mut` | `ptr::null_mut`, `std::ptr::null_mut`, `core::ptr::null_mut` |
| NonNull::new | `nonnull_new` | `NonNull::new`, `std::ptr::NonNull::new`, `core::ptr::NonNull::new` |
| NonNull::dangling | `nonnull_dangling` | `NonNull::dangling`, `std::ptr::NonNull::dangling` |

##### Box Raw Pointer Operations

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Box::into_raw | `box_into_raw` | `Box::into_raw`, `std::boxed::Box::into_raw` |
| Box::from_raw | `box_from_raw` | `Box::from_raw`, `std::boxed::Box::from_raw` |

##### ManuallyDrop

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| ManuallyDrop::new | `manually_drop_new` | `ManuallyDrop::new`, `std::mem::ManuallyDrop::new`, `core::mem::ManuallyDrop::new` |
| ManuallyDrop::into_inner | `manually_drop_into_inner` | `ManuallyDrop::into_inner`, `std::mem::ManuallyDrop::into_inner` |

##### Atomics

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| AtomicBool::new | `atomic_bool_new` | `AtomicBool::new`, `std::sync::atomic::AtomicBool::new`, `core::sync::atomic::AtomicBool::new` |
| AtomicI8::new | `atomic_i8_new` | `AtomicI8::new`, `std::sync::atomic::AtomicI8::new` |
| AtomicI16::new | `atomic_i16_new` | `AtomicI16::new`, `std::sync::atomic::AtomicI16::new` |
| AtomicI32::new | `atomic_i32_new` | `AtomicI32::new`, `std::sync::atomic::AtomicI32::new` |
| AtomicI64::new | `atomic_i64_new` | `AtomicI64::new`, `std::sync::atomic::AtomicI64::new` |
| AtomicIsize::new | `atomic_isize_new` | `AtomicIsize::new`, `std::sync::atomic::AtomicIsize::new` |
| AtomicU8::new | `atomic_u8_new` | `AtomicU8::new`, `std::sync::atomic::AtomicU8::new` |
| AtomicU16::new | `atomic_u16_new` | `AtomicU16::new`, `std::sync::atomic::AtomicU16::new` |
| AtomicU32::new | `atomic_u32_new` | `AtomicU32::new`, `std::sync::atomic::AtomicU32::new` |
| AtomicU64::new | `atomic_u64_new` | `AtomicU64::new`, `std::sync::atomic::AtomicU64::new` |
| AtomicUsize::new | `atomic_usize_new` | `AtomicUsize::new`, `std::sync::atomic::AtomicUsize::new` |
| AtomicPtr::new | `atomic_ptr_new` | `AtomicPtr::new`, `std::sync::atomic::AtomicPtr::new` |

##### Time

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Duration::new | `duration_new` | `Duration::new`, `std::time::Duration::new`, `core::time::Duration::new` |
| Duration::from_secs | `duration_from_secs` | `Duration::from_secs`, `std::time::Duration::from_secs` |
| Duration::from_millis | `duration_from_millis` | `Duration::from_millis`, `std::time::Duration::from_millis` |
| Duration::from_micros | `duration_from_micros` | `Duration::from_micros`, `std::time::Duration::from_micros` |
| Duration::from_nanos | `duration_from_nanos` | `Duration::from_nanos`, `std::time::Duration::from_nanos` |
| Duration::from_secs_f32/f64 | `duration_from_secs_f` | `Duration::from_secs_f32`, `Duration::from_secs_f64` |
| Instant::now | `instant_now` | `Instant::now`, `std::time::Instant::now` |
| SystemTime::now | `system_time_now` | `SystemTime::now`, `std::time::SystemTime::now` |

##### IO

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Cursor::new | `cursor_new` | `Cursor::new`, `std::io::Cursor::new` |
| BufReader::new | `bufreader_new` | `BufReader::new`, `std::io::BufReader::new` |
| BufReader::with_capacity | `bufreader_with_capacity` | `BufReader::with_capacity`, `std::io::BufReader::with_capacity` |
| BufWriter::new | `bufwriter_new` | `BufWriter::new`, `std::io::BufWriter::new` |
| BufWriter::with_capacity | `bufwriter_with_capacity` | `BufWriter::with_capacity`, `std::io::BufWriter::with_capacity` |
| File::open | `file_open` | `File::open`, `std::fs::File::open` |
| File::create | `file_create` | `File::create`, `std::fs::File::create` |

##### Ordering (Comparison Result)

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Ordering::Less | `ordering_less` | `Ordering::Less`, `std::cmp::Ordering::Less` |
| Ordering::Equal | `ordering_equal` | `Ordering::Equal`, `std::cmp::Ordering::Equal` |
| Ordering::Greater | `ordering_greater` | `Ordering::Greater`, `std::cmp::Ordering::Greater` |

##### Poll (Async Support)

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Poll::Ready | `poll_ready` | `Poll::Ready`, `std::task::Poll::Ready` |
| Poll::Pending | `poll_pending` | `Poll::Pending`, `std::task::Poll::Pending` |

##### Panic Support

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Location::caller | `location_caller` | `Location::caller`, `std::panic::Location::caller` |

##### UnsafeCell

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| UnsafeCell::new | `unsafe_cell_new` | `UnsafeCell::new`, `std::cell::UnsafeCell::new`, `core::cell::UnsafeCell::new` |

##### Trait Methods

| Pattern | `initializer_kind` | Matched Paths |
|---------|-------------------|---------------|
| Default::default | `default` | `Default::default`, `std::default::Default::default`, `core::default::Default::default` |
| Clone (generic) | `clone` | Any path ending in `::clone` |

##### Semantic Type Classification (v2.3+)

The analyzer applies **semantic type classification** to ALL expressions using rust-analyzer's type resolution. This approach examines the resolved type to classify the initializer, with expression structure (call, method, macro, etc.) used only as context:

###### ADT Classification by Canonical Path

For types that resolve to an ADT (struct, enum, or union), the analyzer extracts the canonical module path and classifies accordingly:

| Resolved Type Path | `initializer_kind` | Example |
|-------------------|-------------------|---------|
| `alloc::rc::Rc` | `rc_new` | `let x = create_rc();` where return type is `Rc<T>` |
| `alloc::sync::Arc` | `arc_new` | Factory function returning `Arc<T>` |
| `core::option::Option` | `option_variant` | Any function returning `Option<T>` |
| `core::result::Result` | `result_variant` | Any function returning `Result<T, E>` |
| (70+ standard library types) | (type-specific) | See full list in source |

###### User-Defined Types

When the resolved type is a user-defined ADT (not from std/core/alloc), classification is by ADT kind:

| ADT Kind | `initializer_kind` | Example |
|----------|-------------------|---------|
| Struct | `user_struct` | `let p = Point::new(1, 2);` |
| Enum | `user_enum` | `let s = Status::Active;` |
| Union | `user_union` | `let u = MyUnion::new();` |

###### Tuple Types

| Type | `initializer_kind` | Example |
|------|-------------------|---------|
| Tuple | `tuple` | `let (tx, rx) = channel();` |

###### impl Trait Types

For opaque `impl Trait` return types, the analyzer extracts the primary trait bound:

| Trait Bound | `initializer_kind` | Example |
|-------------|-------------------|---------|
| `Future` | `impl_future` | `let f = async_fn();` returning `impl Future<Output = T>` |
| `Iterator` | `impl_iterator` | `let i = get_iter();` returning `impl Iterator<Item = T>` |
| `Fn`/`FnMut`/`FnOnce` | `impl_fn` | `let f = get_closure();` returning `impl Fn(T) -> U` |
| Other traits | `impl_{trait_name}` | Lowercase trait name |

###### Other Semantic Classifications

| Type Category | `initializer_kind` | Detection Method |
|---------------|-------------------|------------------|
| Primitives | `primitive` | `ty.as_builtin()` - i32, bool, char, etc. |
| str | `str` | `ty.as_builtin().is_str()` |
| Closures | `closure` | `ty.is_closure()` |
| Function pointers | `fn_ptr` | `ty.is_fn()` |
| References | `ref` / `ref_mut` | `ty.is_reference()` |
| Raw pointers | `raw_ptr` | `ty.is_raw_ptr()` |

##### Fallback

With semantic classification, the `call` fallback is now rare and only occurs when:
- The type cannot be resolved (e.g., in files outside the crate graph)
- The type is truly unknown to rust-analyzer

| Pattern | `initializer_kind` | Description |
|---------|-------------------|-------------|
| Unresolved call | `call` | Function call with unresolvable return type |

#### Method Call Classification

When the initializer is a method call (`MethodCallExpr`), the analyzer examines the method name:

##### RefCell Methods

| Method | `initializer_kind` | Example |
|--------|-------------------|---------|
| `borrow` | `refcell_borrow` | `let r = cell.borrow();` |
| `borrow_mut` | `refcell_borrow_mut` | `let r = cell.borrow_mut();` |
| `try_borrow` | `refcell_try_borrow` | `let r = cell.try_borrow();` |
| `try_borrow_mut` | `refcell_try_borrow_mut` | `let r = cell.try_borrow_mut();` |

##### Cell Methods

| Method | `initializer_kind` | Example |
|--------|-------------------|---------|
| `get` | `cell_get` | `let v = cell.get();` |
| `set` | `cell_set` | `let _ = cell.set(v);` |
| `replace` | `cell_replace` | `let old = cell.replace(new);` |
| `take` | `cell_take` | `let v = cell.take();` |

##### Mutex/RwLock Methods

| Method | `initializer_kind` | Example |
|--------|-------------------|---------|
| `lock` | `mutex_lock` | `let guard = mutex.lock().unwrap();` |
| `try_lock` | `mutex_try_lock` | `let guard = mutex.try_lock();` |
| `read` | `rwlock_read` | `let guard = rwlock.read().unwrap();` |
| `write` | `rwlock_write` | `let guard = rwlock.write().unwrap();` |
| `try_read` | `rwlock_try_read` | `let guard = rwlock.try_read();` |
| `try_write` | `rwlock_try_write` | `let guard = rwlock.try_write();` |

##### OnceCell Methods

| Method | `initializer_kind` | Example |
|--------|-------------------|---------|
| `get_or_init` | `once_cell_get_or_init` | `let v = cell.get_or_init(\|\| 42);` |
| `get_or_try_init` | `once_cell_get_or_try_init` | `let v = cell.get_or_try_init(\|\| Ok(42));` |

##### MaybeUninit Methods

| Method | `initializer_kind` | Example |
|--------|-------------------|---------|
| `assume_init` | `maybe_uninit_assume_init` | `let v = uninit.assume_init();` |
| `assume_init_read` | `maybe_uninit_assume_init_read` | `let v = uninit.assume_init_read();` |
| `assume_init_ref` | `maybe_uninit_assume_init_ref` | `let r = uninit.assume_init_ref();` |
| `assume_init_mut` | `maybe_uninit_assume_init_mut` | `let r = uninit.assume_init_mut();` |

##### Weak Pointer Methods

| Method | `initializer_kind` | Example |
|--------|-------------------|---------|
| `downgrade` | `weak_downgrade` | `let weak = Rc::downgrade(&rc);` |
| `upgrade` | `weak_upgrade` | `let strong = weak.upgrade();` |

##### Cow Methods

| Method | `initializer_kind` | Example |
|--------|-------------------|---------|
| `to_mut` | `cow_to_mut` | `let m = cow.to_mut();` |
| `into_owned` | `cow_into_owned` | `let owned = cow.into_owned();` |

##### Pin Methods

| Method | `initializer_kind` | Example |
|--------|-------------------|---------|
| `as_ref` | `pin_as_ref` | `let r = pin.as_ref();` |
| `as_mut` | `pin_as_mut` | `let r = pin.as_mut();` |
| `into_inner` | `into_inner` | `let v = pin.into_inner();` |

##### Atomic Methods

| Method | `initializer_kind` | Example |
|--------|-------------------|---------|
| `load` | `atomic_load` | `let v = atomic.load(Ordering::SeqCst);` |
| `store` | `atomic_store` | `atomic.store(v, Ordering::SeqCst);` |
| `swap` | `atomic_swap` | `let old = atomic.swap(new, Ordering::SeqCst);` |
| `compare_exchange` | `atomic_compare_exchange` | `let r = atomic.compare_exchange(...);` |
| `compare_exchange_weak` | `atomic_compare_exchange_weak` | `let r = atomic.compare_exchange_weak(...);` |
| `fetch_add` | `atomic_fetch_add` | `let old = atomic.fetch_add(1, Ordering::SeqCst);` |
| `fetch_sub` | `atomic_fetch_sub` | `let old = atomic.fetch_sub(1, Ordering::SeqCst);` |
| `fetch_and` | `atomic_fetch_and` | `let old = atomic.fetch_and(mask, Ordering::SeqCst);` |
| `fetch_or` | `atomic_fetch_or` | `let old = atomic.fetch_or(mask, Ordering::SeqCst);` |
| `fetch_xor` | `atomic_fetch_xor` | `let old = atomic.fetch_xor(mask, Ordering::SeqCst);` |
| `fetch_max` | `atomic_fetch_max` | `let old = atomic.fetch_max(val, Ordering::SeqCst);` |
| `fetch_min` | `atomic_fetch_min` | `let old = atomic.fetch_min(val, Ordering::SeqCst);` |
| `fetch_update` | `atomic_fetch_update` | `let r = atomic.fetch_update(...);` |

##### Duration/Instant Methods

| Method | `initializer_kind` | Example |
|--------|-------------------|---------|
| `as_secs` | `duration_as_secs` | `let s = duration.as_secs();` |
| `as_millis` | `duration_as_millis` | `let ms = duration.as_millis();` |
| `as_micros` | `duration_as_micros` | `let us = duration.as_micros();` |
| `as_nanos` | `duration_as_nanos` | `let ns = duration.as_nanos();` |
| `as_secs_f32`/`as_secs_f64` | `duration_as_secs_f` | `let s = duration.as_secs_f64();` |
| `elapsed` | `instant_elapsed` | `let d = instant.elapsed();` |
| `duration_since` | `instant_duration_since` | `let d = instant.duration_since(earlier);` |

##### Iterator Methods

| Method | `initializer_kind` | Example |
|--------|-------------------|---------|
| `iter` | `iter` | `let it = vec.iter();` |
| `iter_mut` | `iter_mut` | `let it = vec.iter_mut();` |
| `into_iter` | `into_iter` | `let it = vec.into_iter();` |

##### Common Combinator Methods

| Method | `initializer_kind` | Example |
|--------|-------------------|---------|
| `unwrap` | `unwrap` | `let v = opt.unwrap();` |
| `expect` | `expect` | `let v = opt.expect("msg");` |
| `map` | `map` | `let v = opt.map(\|x\| x + 1);` |
| `and_then` | `and_then` | `let v = opt.and_then(\|x\| Some(x));` |
| `ok` | `ok` | `let opt = result.ok();` |
| `err` | `err` | `let opt = result.err();` |
| `clone` | `clone` | `let c = val.clone();` |

##### Fallback

For method calls not matching known patterns, the analyzer applies semantic type classification (see above) based on the resolved return type. This means methods like `.iter().map().filter()` chains are classified by their final return type rather than falling back to a generic `method` classification.

| Method | `initializer_kind` | Description |
|--------|-------------------|-------------|
| Unknown method | (semantic) | Classified by resolved return type |
| Unresolved method | `method` | Method with unresolvable return type |

#### Macro Classification

When the initializer is a macro invocation (`MacroExpr`), the analyzer examines the macro name:

| Macro | `initializer_kind` | Example |
|-------|-------------------|---------|
| `vec!` | `vec_macro` | `let v = vec![1, 2, 3];` |
| `format!` | `format_macro` | `let s = format!("{}", x);` |
| `println!`/`print!`/`eprintln!`/`eprint!` | `print_macro` | `let _ = println!("hi");` |
| `panic!` | `panic_macro` | `let _ = panic!("error");` |
| `assert!`/`assert_eq!`/`assert_ne!` | `assert_macro` | `let _ = assert!(true);` |
| `pin!` | `pin_macro` | `let p = pin!(future);` |
| Unknown macro | `macro` | Any macro not matching above |

#### Design Rationale

The `initializer_kind` classification serves several purposes:

1. **Precise Tracking Selection**: The macro can select the exact tracking function based on how a value was created, not just its type. For example, `Rc::clone` should use `track_rc_clone` (which records the source reference count) rather than `track_rc_new`.

2. **Type Alias Handling**: When users define type aliases like `type MyRc<T> = Rc<T>`, the call `MyRc::new(x)` won't match the `Rc::new` pattern. However, the type flags (`is_rc: true`) still enable correct tracking via the fallback path.

3. **Guard Tracking**: Methods like `borrow()`, `lock()`, and `read()` create guard types that require special tracking to monitor their lifetime and detect potential deadlocks or borrow violations.

4. **Unsafe Operation Tracking**: Patterns like `MaybeUninit::assume_init()` and `Box::from_raw()` indicate unsafe operations that warrant special attention in ownership visualization.

5. **Performance Optimization**: By classifying at analysis time, the macro avoids runtime pattern matching and can generate optimal tracking code directly.

**Disambiguation Fields (v2.2+)**

These fields enable precise variable lookup when multiple variables share the same name:

| Field | Purpose |
|-------|---------|
| `function_name` | Name of the containing function (null for module-level) |
| `decl_index` | 0-based declaration order within the function |
| `scope_id` | Unique scope identifier |
| `span_start` | Byte offset of pattern start |
| `span_end` | Byte offset of pattern end |

The macro uses these fields to disambiguate variables with the same name in different functions or shadowed within the same function:

```rust
fn foo() {
    let x = Rc::new(1);  // function_name: "foo", decl_index: 0
}

fn bar() {
    let x = Arc::new(1); // function_name: "bar", decl_index: 0
    let x = x.clone();   // function_name: "bar", decl_index: 1 (shadowing)
}
```

These flags are not mutually exclusive. A type like `Rc<RefCell<Vec<String>>>` will have `is_rc`, `is_refcell`, `is_vec`, and `is_string` all set to `true`, reflecting the nested structure.

**Binding Pattern Flags (for Macro Transformation)**

The following flags help the macro make better transformation decisions based on the [battle test whitepaper](https://mehmet-ylcnky.github.io/BorrowScope/battle-test-whitepaper/) error taxonomy:

| Flag | Battle Test Error | Macro Action |
|------|-------------------|--------------|
| `is_tuple_binding` | ERR-002: Tuple destructuring | Skip tracking or handle specially |
| `is_mut_binding` | Pattern syntax | let mut x = ... |
| `is_impl_trait` | Type annotation syntax | impl Trait type |

These flags are not mutually exclusive. A type like `Rc<RefCell<Vec<String>>>` will have `is_rc`, `is_refcell`, `is_vec`, and `is_string` all set to `true`, reflecting the nested structure. Additionally, it will have `is_clone: true`, `is_drop: true`, `is_sized: true` from trait detection.

### Example Output

For a source file containing:

```rust
fn example() {
    let count = 42;
    let shared = Rc::new(RefCell::new(vec![1, 2, 3]));
    let guard = shared.borrow();
    let future = async { 42 };
}
```

The analyzer produces:

```json
{
  "version": "2.3",
  "analyzer_version": "0.1.0",
  "files": {
    "src/main.rs": [
      {
        "name": "count",
        "ty": "i32",
        "is_copy": true,
        "is_clone": true,
        "is_send": true,
        "is_sync": true,
        "is_drop": false,
        "is_sized": true,
        "is_future": false,
        "is_iterator": false,
        "is_primitive": true,
        "is_reference": false,
        "is_mutable_reference": false,
        "is_raw_ptr": false,
        "is_slice": false,
        "is_str": false,
        "is_closure": false,
        "is_fn_ptr": false,
        "is_dyn_trait": false,
        "is_union": false,
        "is_rc": false,
        "is_arc": false,
        "is_box": false,
        "is_vec": false,
        "is_string": false,
        "file": "src/main.rs",
        "line": 2,
        "column": 8
      },
      {
        "name": "shared",
        "ty": "Rc<RefCell<Vec<i32, Global>>, Global>",
        "is_copy": false,
        "is_clone": true,
        "is_send": false,
        "is_sync": false,
        "is_drop": true,
        "is_sized": true,
        "is_future": false,
        "is_iterator": false,
        "is_primitive": false,
        "is_reference": false,
        "is_mutable_reference": false,
        "is_raw_ptr": false,
        "is_slice": false,
        "is_str": false,
        "is_closure": false,
        "is_fn_ptr": false,
        "is_dyn_trait": false,
        "is_union": false,
        "is_rc": true,
        "is_arc": false,
        "is_box": false,
        "is_refcell": true,
        "is_vec": true,
        "is_string": false,
        "file": "src/main.rs",
        "line": 3,
        "column": 8
      },
      {
        "name": "guard",
        "ty": "Ref<'_, Vec<i32, Global>>",
        "is_copy": false,
        "is_clone": false,
        "is_send": false,
        "is_sync": true,
        "is_drop": true,
        "is_sized": true,
        "is_guard": true,
        "is_vec": true,
        "file": "src/main.rs",
        "line": 4,
        "column": 8
      },
      {
        "name": "future",
        "ty": "impl Future<Output = i32>",
        "is_copy": false,
        "is_clone": false,
        "is_send": true,
        "is_sync": false,
        "is_drop": false,
        "is_sized": true,
        "is_future": true,
        "is_closure": true,
        "file": "src/main.rs",
        "line": 5,
        "column": 8
      }
    ]
  }
}
```

Key observations:
- `count` has `is_primitive: true`, `is_copy: true`, `is_send: true`, `is_sync: true` - all detected semantically
- `shared` has `is_send: false`, `is_sync: false` because `Rc` is not thread-safe
- `guard` has `is_guard: true` and `is_sync: true` (guards are Sync but not Send)
- `future` has `is_future: true` detected via `impls_trait(Future)`
- All trait flags are determined by actual trait implementations, not string matching

---

