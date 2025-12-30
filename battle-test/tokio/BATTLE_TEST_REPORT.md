# BorrowScope Battle Test: tokio

**Project:** [tokio](https://github.com/tokio-rs/tokio)  
**Version:** Latest (cloned on 2024-12-26)  
**Stars:** ~27k  
**Description:** Async runtime for Rust - provides async I/O, timers, channels, and task scheduling  

**Note:** "Pass" means the macro transformation compiles without errors. Each file was tested individually with `cargo check` to get accurate per-file results.

---

## Phase 1: Reconnaissance

### Lines of Code
```
~168,045 lines total (tokio/src only: ~68,000 lines)
```

### Key Modules to Test
| Module | Description | Ownership Patterns |
|--------|-------------|-------------------|
| `tokio/src/sync/` | Synchronization primitives | Mutex, RwLock, channels, Arc patterns |
| `tokio/src/task/` | Task spawning and management | Future ownership, JoinHandle |
| `tokio/src/runtime/` | Runtime scheduler | Worker threads, task queues |
| `tokio/src/io/` | Async I/O traits and utilities | ReadBuf, AsyncRead/Write |
| `tokio/src/net/` | Async networking | TcpStream, UdpSocket |
| `tokio/src/time/` | Timers and delays | Sleep, Interval |
| `tokio/src/fs/` | Async filesystem | File handles, paths |

---

## Phase 2: Error Log

### ERR-009: Self-consuming functions - cannot move out of shared reference (E0507)

**Frequency:** 196 occurrences (most common error)

**Location Examples:**
- `tokio/src/sync/mutex.rs` - MutexGuard methods
- `tokio/src/sync/rwlock/*.rs` - Guard map/downgrade methods
- `tokio/src/future/maybe_done.rs` - Pin<&mut Self> methods
- `tokio/src/task/local.rs` - LocalSet methods

**Error Message:**
```
error[E0507]: cannot move out of a shared reference
  --> tokio/src/sync/mutex.rs:590
   |
   | #[trace_borrow]
   | ^^^^^^^^^^^^^^^ move occurs because value has type `MutexGuard<'_, T>`, which does not implement `Copy`
```

**Root Cause:**
The macro wraps `self` in a borrow before the function body executes. When the function consumes `self` (like guard methods that call `.map()` or `.downgrade()`), the macro's borrow prevents the move.

---

### ERR-003: Mutable method chains - cannot borrow as mutable (E0596)

**Frequency:** 193 occurrences (second most common)

**Location Examples:**
- `tokio/src/io/read_buf.rs` - `filled_mut`, `unfilled_mut`, `clear`
- `tokio/src/sync/once_cell.rs` - initialization methods
- `tokio/src/task/join_set.rs` - spawn methods
- `tokio/src/runtime/builder.rs` - builder methods

**Error Message:**
```
error[E0596]: cannot borrow data in a `&` reference as mutable
  --> tokio/src/io/read_buf.rs:79
   |
   | #[trace_borrow]
   | ^^^^^^^^^^^^^^^ cannot borrow as mutable
```

**Root Cause:**
The macro wraps `self` with `track_borrow("self", &self)` which returns an immutable reference. Methods requiring `&mut self` then fail because they receive `&self` instead.

---

### ERR-005: Macro scope issues - cannot find attribute/macro (unknown)

**Frequency:** 78 occurrences

**Location Examples:**
- `tokio/src/sync/watch.rs` - cfg_sync! macro scope
- `tokio/src/sync/oneshot.rs` - cfg_sync! macro scope
- `tokio/src/net/addr.rs` - cfg_net! macro scope

**Error Message:**
```
error: cannot find attribute `trace_borrow` in this scope
```

**Root Cause:**
Import placement breaks tokio's internal `cfg_*` macro definitions.

---

### ERR-012: Trait impl methods - no method found (E0599)

**Frequency:** 37 occurrences

**Root Cause:**
The macro changes the type of `self`, breaking method resolution for trait implementations.

---

### ERR-002: Tuple/struct destructuring - cannot find value (E0425)

**Frequency:** 32 occurrences

**Root Cause:**
The macro fails to properly handle struct field destructuring patterns.

---

### ERR-010: Wrong argument count (E0061)

**Frequency:** 21 occurrences

**Root Cause:**
The macro incorrectly transforms method calls, dropping arguments in certain patterns.

---

## Phase 3: Compilation Results


### blocking.rs (5 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `spawn_blocking` | ✅ Pass | | |
| `spawn_mandatory_blocking` | ✅ Pass | | |
| `poll` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `assert_send_sync` | ✅ Pass | | |

### doc/mod.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `register` | ✅ Pass | | |
| `reregister` | ✅ Pass | | |
| `deregister` | ✅ Pass | | |

### fs/canonicalize.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `canonicalize` | ✅ Pass | | |

### fs/copy.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `copy` | ✅ Pass | | |

### fs/create_dir.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `create_dir` | ✅ Pass | | |

### fs/create_dir_all.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `create_dir_all` | ✅ Pass | | |

### fs/dir_builder.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `recursive` | ✅ Pass | | |
| `create` | ❌ Fail | ERR-003 | |
| `mode` | ✅ Pass | | |

### fs/file.rs (34 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `open` | ✅ Pass | | |
| `create` | ✅ Pass | | |
| `create_new` | ✅ Pass | | |
| `options` | ✅ Pass | | |
| `from_std` | ✅ Pass | | |
| `sync_all` | ✅ Pass | | |
| `sync_data` | ✅ Pass | | |
| `set_len` | ✅ Pass | | |
| `metadata` | ✅ Pass | | |
| `try_clone` | ✅ Pass | | |
| `into_std` | ✅ Pass | | |
| `try_into_std` | ✅ Pass | | |
| `set_permissions` | ✅ Pass | | |
| `set_max_buf_size` | ✅ Pass | | |
| `max_buf_size` | ✅ Pass | | |
| `poll_read` | ✅ Pass | | |
| `start_seek` | ✅ Pass | | |
| `poll_complete` | ✅ Pass | | |
| `poll_write` | ✅ Pass | | |
| `poll_write_vectored` | ✅ Pass | | |
| `is_write_vectored` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ✅ Pass | | |
| `from` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `as_raw_fd` | ❌ Fail | ERR-009 | |
| `as_fd` | ✅ Pass | | |
| `from_raw_fd` | ❌ Fail | ERR-003 | |
| `as_raw_handle` | ✅ Pass | | |
| `as_handle` | ✅ Pass | | |
| `from_raw_handle` | ✅ Pass | | |
| `complete_inflight` | ✅ Pass | | |
| `poll_complete_inflight` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |

### fs/file/tests.rs (28 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `open_read` | ✅ Pass | | |
| `read_twice_before_dispatch` | ✅ Pass | | |
| `read_with_smaller_buf` | ✅ Pass | | |
| `read_with_bigger_buf` | ✅ Pass | | |
| `read_err_then_read_success` | ✅ Pass | | |
| `open_write` | ✅ Pass | | |
| `flush_while_idle` | ✅ Pass | | |
| `read_with_buffer_larger_than_max` | ✅ Pass | | |
| `write_with_buffer_larger_than_max` | ✅ Pass | | |
| `write_twice_before_dispatch` | ✅ Pass | | |
| `incomplete_read_followed_by_write` | ✅ Pass | | |
| `incomplete_partial_read_followed_by_write` | ✅ Pass | | |
| `incomplete_read_followed_by_flush` | ✅ Pass | | |
| `incomplete_flush_followed_by_write` | ✅ Pass | | |
| `read_err` | ✅ Pass | | |
| `write_write_err` | ✅ Pass | | |
| `write_read_write_err` | ✅ Pass | | |
| `write_read_flush_err` | ✅ Pass | | |
| `write_seek_write_err` | ✅ Pass | | |
| `write_seek_flush_err` | ✅ Pass | | |
| `sync_all_ordered_after_write` | ✅ Pass | | |
| `sync_all_err_ordered_after_write` | ✅ Pass | | |
| `sync_data_ordered_after_write` | ✅ Pass | | |
| `sync_data_err_ordered_after_write` | ✅ Pass | | |
| `open_set_len_ok` | ✅ Pass | | |
| `open_set_len_err` | ✅ Pass | | |
| `partial_read_set_len_ok` | ✅ Pass | | |
| `busy_file_seek_error` | ✅ Pass | | |

### fs/hard_link.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `hard_link` | ✅ Pass | | |

### fs/metadata.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `metadata` | ✅ Pass | | |

### fs/mocks.rs (11 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `read` | ✅ Pass | | |
| `read` | ✅ Pass | | |
| `seek` | ✅ Pass | | |
| `write` | ✅ Pass | | |
| `flush` | ✅ Pass | | |
| `from` | ✅ Pass | | |
| `spawn_blocking` | ✅ Pass | | |
| `spawn_mandatory_blocking` | ✅ Pass | | |
| `poll` | ✅ Pass | | |
| `len` | ✅ Pass | | |
| `run_one` | ✅ Pass | | |

### fs/mod.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `asyncify` | ✅ Pass | | |

### fs/open_options.rs (19 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `read` | ❌ Fail | ERR-012 | |
| `write` | ❌ Fail | ERR-012 | |
| `append` | ❌ Fail | ERR-012 | |
| `truncate` | ❌ Fail | ERR-012 | |
| `create` | ❌ Fail | ERR-012 | |
| `create_new` | ❌ Fail | ERR-012 | |
| `open` | ❌ Fail | ERR-012 | |
| `std_open` | ❌ Fail | ERR-005 | |
| `as_inner_mut` | ✅ Pass | | |
| `mode` | ❌ Fail | ERR-012 | |
| `custom_flags` | ❌ Fail | ERR-012 | |
| `access_mode` | ✅ Pass | | |
| `share_mode` | ✅ Pass | | |
| `custom_flags` | ❌ Fail | ERR-012 | |
| `attributes` | ✅ Pass | | |
| `security_qos_flags` | ✅ Pass | | |
| `from` | ✅ Pass | | |
| `default` | ✅ Pass | | |

### fs/open_options/uring_open_options.rs (12 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `append` | ✅ Pass | | |
| `create` | ✅ Pass | | |
| `create_new` | ✅ Pass | | |
| `read` | ✅ Pass | | |
| `write` | ✅ Pass | | |
| `truncate` | ✅ Pass | | |
| `mode` | ✅ Pass | | |
| `custom_flags` | ✅ Pass | | |
| `access_mode` | ✅ Pass | | |
| `creation_mode` | ✅ Pass | | |
| `from` | ✅ Pass | | |

### fs/read.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `read` | ✅ Pass | | |

### fs/read_dir.rs (10 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `read_dir` | ✅ Pass | | |
| `next_entry` | ✅ Pass | | |
| `poll_next_entry` | ✅ Pass | | |
| `next_chunk` | ✅ Pass | | |
| `ino` | ✅ Pass | | |
| `path` | ✅ Pass | | |
| `file_name` | ✅ Pass | | |
| `metadata` | ✅ Pass | | |
| `file_type` | ✅ Pass | | |
| `as_inner` | ✅ Pass | | |

### fs/read_link.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `read_link` | ✅ Pass | | |

### fs/read_to_string.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `read_to_string` | ✅ Pass | | |

### fs/read_uring.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `read_uring` | ✅ Pass | | |
| `read_to_end_uring` | ✅ Pass | | |
| `small_probe_read` | ✅ Pass | | |
| `op_read` | ✅ Pass | | |

### fs/remove_dir.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `remove_dir` | ✅ Pass | | |

### fs/remove_dir_all.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `remove_dir_all` | ✅ Pass | | |

### fs/remove_file.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `remove_file` | ✅ Pass | | |

### fs/rename.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `rename` | ✅ Pass | | |

### fs/set_permissions.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `set_permissions` | ✅ Pass | | |

### fs/symlink.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `symlink` | ✅ Pass | | |

### fs/symlink_dir.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `symlink_dir` | ✅ Pass | | |

### fs/symlink_file.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `symlink_file` | ✅ Pass | | |

### fs/symlink_metadata.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `symlink_metadata` | ✅ Pass | | |

### fs/try_exists.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `try_exists` | ✅ Pass | | |

### fs/write.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `write` | ✅ Pass | | |
| `write_uring` | ✅ Pass | | |
| `write_spawn_blocking` | ❌ Fail | E0521 | |

### future/block_on.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `block_on` | ✅ Pass | | |
| `block_on` | ✅ Pass | | |

### future/maybe_done.rs (7 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `maybe_done` | ✅ Pass | | |
| `output_mut` | ✅ Pass | | |
| `take_output` | ❌ Fail | ERR-009 | |
| `poll` | ✅ Pass | | |
| `poll` | ✅ Pass | | |
| `maybe_done_miri` | ✅ Pass | | |
| `wake` | ✅ Pass | | |

### future/trace.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `id` | ✅ Pass | | |

### future/try_join.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `try_join3` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-009 | |

### io/async_buf_read.rs (8 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `poll_fill_buf` | ✅ Pass | | |
| `consume` | ✅ Pass | | |
| `poll_fill_buf` | ✅ Pass | | |
| `consume` | ✅ Pass | | |
| `poll_fill_buf` | ✅ Pass | | |
| `consume` | ✅ Pass | | |
| `poll_fill_buf` | ✅ Pass | | |
| `consume` | ✅ Pass | | |

### io/async_fd.rs (51 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `with_interest` | ✅ Pass | | |
| `new_with_handle_and_interest` | ✅ Pass | | |
| `try_new` | ✅ Pass | | |
| `try_with_interest` | ✅ Pass | | |
| `try_new_with_handle_and_interest` | ✅ Pass | | |
| `get_ref` | ✅ Pass | | |
| `get_mut` | ✅ Pass | | |
| `take_inner` | ✅ Pass | | |
| `into_inner` | ✅ Pass | | |
| `poll_read_ready` | ✅ Pass | | |
| `poll_read_ready_mut` | ✅ Pass | | |
| `poll_write_ready` | ✅ Pass | | |
| `poll_write_ready_mut` | ✅ Pass | | |
| `ready` | ✅ Pass | | |
| `ready_mut` | ✅ Pass | | |
| `readable` | ✅ Pass | | |
| `readable_mut` | ✅ Pass | | |
| `writable` | ✅ Pass | | |
| `writable_mut` | ✅ Pass | | |
| `async_io` | ✅ Pass | | |
| `async_io_mut` | ✅ Pass | | |
| `try_io` | ✅ Pass | | |
| `try_io_mut` | ✅ Pass | | |
| `as_raw_fd` | ✅ Pass | | |
| `as_fd` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `drop` | ❌ Fail | ERR-003 | |
| `clear_ready` | ❌ Fail | ERR-003 | |
| `clear_ready_matching` | ✅ Pass | | |
| `retain_ready` | ✅ Pass | | |
| `ready` | ✅ Pass | | |
| `try_io` | ✅ Pass | | |
| `get_ref` | ✅ Pass | | |
| `get_inner` | ✅ Pass | | |
| `clear_ready` | ❌ Fail | ERR-003 | |
| `clear_ready_matching` | ✅ Pass | | |
| `retain_ready` | ✅ Pass | | |
| `ready` | ✅ Pass | | |
| `try_io` | ✅ Pass | | |
| `get_ref` | ✅ Pass | | |
| `get_mut` | ✅ Pass | | |
| `get_inner` | ✅ Pass | | |
| `get_inner_mut` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `fmt` | ❌ Fail | ERR-003 | |
| `into_parts` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `fmt` | ❌ Fail | ERR-003 | |
| `source` | ✅ Pass | | |
| `from` | ✅ Pass | | |

### io/async_read.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `poll_read` | ❌ Fail | ERR-003 | |
| `poll_read` | ❌ Fail | ERR-003 | |
| `poll_read` | ❌ Fail | ERR-003 | |
| `poll_read` | ❌ Fail | ERR-003 | |

### io/async_seek.rs (6 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `start_seek` | ✅ Pass | | |
| `poll_complete` | ✅ Pass | | |
| `start_seek` | ✅ Pass | | |
| `poll_complete` | ✅ Pass | | |
| `start_seek` | ✅ Pass | | |
| `poll_complete` | ✅ Pass | | |

### io/async_write.rs (37 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `poll_write_vectored` | ❌ Fail | ERR-009 | |
| `is_write_vectored` | ❌ Fail | ERR-009 | |
| `poll_write` | ✅ Pass | | |
| `poll_write_vectored` | ❌ Fail | ERR-009 | |
| `is_write_vectored` | ❌ Fail | ERR-009 | |
| `poll_flush` | ❌ Fail | ERR-009 | |
| `poll_shutdown` | ❌ Fail | ERR-009 | |
| `poll_write` | ✅ Pass | | |
| `poll_write_vectored` | ❌ Fail | ERR-009 | |
| `is_write_vectored` | ❌ Fail | ERR-009 | |
| `poll_flush` | ❌ Fail | ERR-009 | |
| `poll_shutdown` | ❌ Fail | ERR-009 | |
| `poll_write` | ✅ Pass | | |
| `poll_write_vectored` | ❌ Fail | ERR-009 | |
| `is_write_vectored` | ❌ Fail | ERR-009 | |
| `poll_flush` | ❌ Fail | ERR-009 | |
| `poll_shutdown` | ❌ Fail | ERR-009 | |
| `poll_write` | ✅ Pass | | |
| `poll_write_vectored` | ❌ Fail | ERR-009 | |
| `is_write_vectored` | ❌ Fail | ERR-009 | |
| `poll_flush` | ❌ Fail | ERR-009 | |
| `poll_shutdown` | ❌ Fail | ERR-009 | |
| `poll_write` | ✅ Pass | | |
| `poll_write_vectored` | ❌ Fail | ERR-009 | |
| `is_write_vectored` | ❌ Fail | ERR-009 | |
| `poll_flush` | ❌ Fail | ERR-009 | |
| `poll_shutdown` | ❌ Fail | ERR-009 | |
| `poll_write` | ✅ Pass | | |
| `poll_write_vectored` | ❌ Fail | ERR-009 | |
| `is_write_vectored` | ❌ Fail | ERR-009 | |
| `poll_flush` | ❌ Fail | ERR-009 | |
| `poll_shutdown` | ❌ Fail | ERR-009 | |
| `poll_write` | ✅ Pass | | |
| `poll_write_vectored` | ❌ Fail | ERR-009 | |
| `is_write_vectored` | ❌ Fail | ERR-009 | |
| `poll_flush` | ❌ Fail | ERR-009 | |
| `poll_shutdown` | ❌ Fail | ERR-009 | |

### io/blocking.rs (15 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `poll_read` | ✅ Pass | | |
| `poll_write` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ✅ Pass | | |
| `with_capacity` | ✅ Pass | | |
| `is_empty` | ✅ Pass | | |
| `len` | ✅ Pass | | |
| `copy_to` | ❌ Fail | ERR-003 | |
| `copy_from` | ✅ Pass | | |
| `bytes` | ✅ Pass | | |
| `read_from` | ✅ Pass | | |
| `write_to` | ✅ Pass | | |
| `discard_read` | ❌ Fail | ERR-003 | |
| `copy_from_bufs` | ✅ Pass | | |

### io/bsd/poll_aio.rs (12 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `register` | ✅ Pass | | |
| `deregister` | ✅ Pass | | |
| `reregister` | ✅ Pass | | |
| `new_for_aio` | ✅ Pass | | |
| `new_for_lio` | ✅ Pass | | |
| `new_with_interest` | ✅ Pass | | |
| `clear_ready` | ✅ Pass | | |
| `into_inner` | ✅ Pass | | |
| `poll_ready` | ✅ Pass | | |
| `deref` | ✅ Pass | | |
| `deref_mut` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |

### io/interest.rs (7 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `remove` | ✅ Pass | | |
| `to_mio` | ✅ Pass | | |
| `mio_add` | ❌ Fail | ERR-012 | |
| `mask` | ✅ Pass | | |
| `bitor` | ❌ Fail | ERR-003 | |
| `bitor_assign` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-012 | |

### io/join.rs (16 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `join` | ✅ Pass | | |
| `into_inner` | ✅ Pass | | |
| `reader` | ✅ Pass | | |
| `writer` | ✅ Pass | | |
| `reader_mut` | ✅ Pass | | |
| `writer_mut` | ✅ Pass | | |
| `reader_pin_mut` | ✅ Pass | | |
| `writer_pin_mut` | ❌ Fail | ERR-009 | |
| `poll_read` | ❌ Fail | ERR-009 | |
| `poll_write` | ✅ Pass | | |
| `poll_flush` | ❌ Fail | ERR-009 | |
| `poll_shutdown` | ✅ Pass | | |
| `poll_write_vectored` | ❌ Fail | ERR-009 | |
| `is_write_vectored` | ❌ Fail | ERR-009 | |
| `poll_fill_buf` | ✅ Pass | | |
| `consume` | ❌ Fail | ERR-009 | |

### io/poll_evented.rs (12 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `new_with_interest` | ✅ Pass | | |
| `new_with_interest_and_handle` | ✅ Pass | | |
| `registration` | ✅ Pass | | |
| `into_inner` | ✅ Pass | | |
| `reregister` | ✅ Pass | | |
| `poll_read` | ✅ Pass | | |
| `poll_write` | ✅ Pass | | |
| `poll_write_vectored` | ✅ Pass | | |
| `deref` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `drop` | ❌ Fail | ERR-003 | |

### io/read_buf.rs (25 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `uninit` | ✅ Pass | | |
| `capacity` | ✅ Pass | | |
| `filled` | ✅ Pass | | |
| `filled_mut` | ✅ Pass | | |
| `take` | ❌ Fail | ERR-001,ERR-003 | |
| `initialized` | ✅ Pass | | |
| `initialized_mut` | ✅ Pass | | |
| `inner_mut` | ✅ Pass | | |
| `unfilled_mut` | ✅ Pass | | |
| `initialize_unfilled` | ❌ Fail | ERR-001,ERR-003 | |
| `initialize_unfilled_to` | ✅ Pass | | |
| `remaining` | ✅ Pass | | |
| `clear` | ✅ Pass | | |
| `advance` | ❌ Fail | ERR-003 | |
| `set_filled` | ✅ Pass | | |
| `assume_init` | ✅ Pass | | |
| `put_slice` | ✅ Pass | | |
| `remaining_mut` | ✅ Pass | | |
| `advance_mut` | ✅ Pass | | |
| `chunk_mut` | ❌ Fail | ERR-003 | |
| `fmt` | ❌ Fail | ERR-003 | |
| `slice_to_uninit_mut` | ❌ Fail | ERR-003 | |
| `slice_assume_init` | ✅ Pass | | |
| `slice_assume_init_mut` | ✅ Pass | | |

### io/ready.rs (19 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `from_mio` | ✅ Pass | | |
| `is_empty` | ✅ Pass | | |
| `is_readable` | ✅ Pass | | |
| `is_writable` | ✅ Pass | | |
| `is_read_closed` | ✅ Pass | | |
| `is_write_closed` | ✅ Pass | | |
| `is_priority` | ✅ Pass | | |
| `is_error` | ✅ Pass | | |
| `contains` | ❌ Fail | ERR-008 | |
| `from_usize` | ✅ Pass | | |
| `as_usize` | ✅ Pass | | |
| `from_interest` | ✅ Pass | | |
| `intersection` | ✅ Pass | | |
| `satisfies` | ✅ Pass | | |
| `bitor` | ✅ Pass | | |
| `bitor_assign` | ✅ Pass | | |
| `bitand` | ✅ Pass | | |
| `sub` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |

### io/seek.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `seek` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-009 | |

### io/split.rs (13 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `split` | ✅ Pass | | |
| `with_lock` | ✅ Pass | | |
| `is_pair_of` | ✅ Pass | | |
| `unsplit` | ✅ Pass | | |
| `is_pair_of` | ✅ Pass | | |
| `poll_read` | ❌ Fail | ERR-009 | |
| `poll_write` | ❌ Fail | ERR-009 | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ✅ Pass | | |
| `poll_write_vectored` | ❌ Fail | ERR-009 | |
| `is_write_vectored` | ❌ Fail | ERR-009 | |
| `fmt` | ❌ Fail | ERR-003 | |
| `fmt` | ❌ Fail | ERR-003 | |

### io/stderr.rs (8 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `stderr` | ✅ Pass | | |
| `as_raw_fd` | ❌ Fail | ERR-005 | |
| `as_fd` | ❌ Fail | ERR-005 | |
| `as_raw_handle` | ✅ Pass | | |
| `as_handle` | ✅ Pass | | |
| `poll_write` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ✅ Pass | | |

### io/stdin.rs (6 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `stdin` | ✅ Pass | | |
| `as_raw_fd` | ❌ Fail | ERR-005 | |
| `as_fd` | ❌ Fail | ERR-005 | |
| `as_raw_handle` | ✅ Pass | | |
| `as_handle` | ✅ Pass | | |
| `poll_read` | ✅ Pass | | |

### io/stdio_common.rs (13 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `poll_write` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ✅ Pass | | |
| `poll_write` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `poll_write` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ✅ Pass | | |
| `test_splitter` | ✅ Pass | | |
| `test_pseudo_text` | ✅ Pass | | |

### io/stdout.rs (8 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `stdout` | ✅ Pass | | |
| `as_raw_fd` | ❌ Fail | ERR-005 | |
| `as_fd` | ❌ Fail | ERR-005 | |
| `as_raw_handle` | ✅ Pass | | |
| `as_handle` | ✅ Pass | | |
| `poll_write` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ✅ Pass | | |

### io/uring/open.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `complete` | ✅ Pass | | |
| `complete_with_error` | ✅ Pass | | |
| `cancel` | ✅ Pass | | |
| `open` | ✅ Pass | | |

### io/uring/read.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `complete` | ✅ Pass | | |
| `complete_with_error` | ✅ Pass | | |
| `cancel` | ✅ Pass | | |
| `read` | ✅ Pass | | |

### io/uring/utils.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `cstr` | ✅ Pass | | |

### io/uring/write.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `complete` | ✅ Pass | | |
| `complete_with_error` | ✅ Pass | | |
| `cancel` | ✅ Pass | | |
| `write_at` | ✅ Pass | | |

### io/util/async_buf_read_ext.rs (6 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `read_until` | ✅ Pass | | |
| `read_line` | ✅ Pass | | |
| `split` | ✅ Pass | | |
| `fill_buf` | ✅ Pass | | |
| `consume` | ✅ Pass | | |
| `lines` | ✅ Pass | | |

### io/util/async_read_ext.rs (7 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `chain` | ✅ Pass | | |
| `read` | ✅ Pass | | |
| `read_buf` | ✅ Pass | | |
| `read_exact` | ✅ Pass | | |
| `read_to_end` | ✅ Pass | | |
| `read_to_string` | ✅ Pass | | |
| `take` | ✅ Pass | | |

### io/util/async_seek_ext.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `seek` | ✅ Pass | | |
| `rewind` | ❌ Fail | ERR-001,ERR-003 | |
| `stream_position` | ❌ Fail | ERR-001,ERR-003 | |

### io/util/async_write_ext.rs (7 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `write` | ✅ Pass | | |
| `write_vectored` | ✅ Pass | | |
| `write_buf` | ✅ Pass | | |
| `write_all_buf` | ✅ Pass | | |
| `write_all` | ✅ Pass | | |
| `flush` | ✅ Pass | | |
| `shutdown` | ✅ Pass | | |

### io/util/buf_reader.rs (20 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `with_capacity` | ✅ Pass | | |
| `get_ref` | ✅ Pass | | |
| `get_mut` | ✅ Pass | | |
| `get_pin_mut` | ❌ Fail | ERR-009 | |
| `into_inner` | ✅ Pass | | |
| `buffer` | ✅ Pass | | |
| `discard_buffer` | ✅ Pass | | |
| `poll_read` | ❌ Fail | ERR-003,ERR-009 | |
| `poll_fill_buf` | ❌ Fail | ERR-009 | |
| `consume` | ❌ Fail | ERR-009 | |
| `start_seek` | ✅ Pass | | |
| `poll_complete` | ❌ Fail | ERR-009 | |
| `poll_write` | ✅ Pass | | |
| `poll_write_vectored` | ❌ Fail | ERR-009 | |
| `is_write_vectored` | ✅ Pass | | |
| `poll_flush` | ❌ Fail | ERR-009 | |
| `poll_shutdown` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-009 | |
| `assert_unpin` | ❌ Fail | ERR-003 | |

### io/util/buf_stream.rs (19 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `with_capacity` | ✅ Pass | | |
| `get_ref` | ✅ Pass | | |
| `get_mut` | ✅ Pass | | |
| `get_pin_mut` | ❌ Fail | ERR-009 | |
| `into_inner` | ✅ Pass | | |
| `from` | ✅ Pass | | |
| `from` | ✅ Pass | | |
| `poll_write` | ✅ Pass | | |
| `poll_write_vectored` | ❌ Fail | ERR-009 | |
| `is_write_vectored` | ❌ Fail | ERR-009 | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ✅ Pass | | |
| `poll_read` | ❌ Fail | ERR-009 | |
| `start_seek` | ✅ Pass | | |
| `poll_complete` | ✅ Pass | | |
| `poll_fill_buf` | ✅ Pass | | |
| `consume` | ❌ Fail | ERR-009 | |
| `assert_unpin` | ❌ Fail | ERR-009 | |

### io/util/buf_writer.rs (20 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `with_capacity` | ✅ Pass | | |
| `flush_buf` | ❌ Fail | ERR-009 | |
| `get_ref` | ✅ Pass | | |
| `get_mut` | ✅ Pass | | |
| `get_pin_mut` | ❌ Fail | ERR-009 | |
| `into_inner` | ✅ Pass | | |
| `buffer` | ✅ Pass | | |
| `poll_write` | ❌ Fail | ERR-009 | |
| `poll_write_vectored` | ✅ Pass | | |
| `is_write_vectored` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ❌ Fail | ERR-009 | |
| `start_seek` | ✅ Pass | | |
| `poll_complete` | ❌ Fail | ERR-009 | |
| `poll_read` | ✅ Pass | | |
| `poll_fill_buf` | ✅ Pass | | |
| `consume` | ❌ Fail | ERR-009 | |
| `fmt` | ❌ Fail | ERR-009 | |
| `assert_unpin` | ❌ Fail | ERR-003 | |

### io/util/chain.rs (10 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `chain` | ✅ Pass | | |
| `get_ref` | ✅ Pass | | |
| `get_mut` | ✅ Pass | | |
| `get_pin_mut` | ❌ Fail | ERR-009 | |
| `into_inner` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `poll_read` | ❌ Fail | ERR-009 | |
| `poll_fill_buf` | ❌ Fail | ERR-009 | |
| `consume` | ❌ Fail | ERR-009 | |
| `assert_unpin` | ✅ Pass | | |

### io/util/copy.rs (6 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `poll_fill_buf` | ❌ Fail | ERR-009 | |
| `poll_write_buf` | ✅ Pass | | |
| `poll_copy` | ✅ Pass | | |
| `copy` | ✅ Pass | | |
| `poll` | ✅ Pass | | |

### io/util/copy_bidirectional.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `transfer_one_direction` | ✅ Pass | | |
| `copy_bidirectional` | ✅ Pass | | |
| `copy_bidirectional_with_sizes` | ✅ Pass | | |
| `copy_bidirectional_impl` | ✅ Pass | | |

### io/util/copy_buf.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `copy_buf` | ✅ Pass | | |
| `poll` | ✅ Pass | | |
| `assert_unpin` | ✅ Pass | | |

### io/util/empty.rs (13 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `empty` | ✅ Pass | | |
| `poll_read` | ✅ Pass | | |
| `poll_fill_buf` | ✅ Pass | | |
| `consume` | ✅ Pass | | |
| `poll_write` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ✅ Pass | | |
| `is_write_vectored` | ✅ Pass | | |
| `poll_write_vectored` | ✅ Pass | | |
| `start_seek` | ✅ Pass | | |
| `poll_complete` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `assert_unpin` | ❌ Fail | ERR-003 | |

### io/util/fill_buf.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `fill_buf` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-009 | |

### io/util/flush.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `flush` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-009 | |

### io/util/lines.rs (7 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `lines` | ✅ Pass | | |
| `next_line` | ✅ Pass | | |
| `get_mut` | ✅ Pass | | |
| `get_ref` | ✅ Pass | | |
| `into_inner` | ✅ Pass | | |
| `poll_next_line` | ❌ Fail | ERR-009 | |
| `assert_unpin` | ✅ Pass | | |

### io/util/mem.rs (24 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `duplex` | ✅ Pass | | |
| `poll_read` | ❌ Fail | ERR-009 | |
| `poll_write` | ❌ Fail | ERR-009 | |
| `poll_write_vectored` | ❌ Fail | ERR-009 | |
| `is_write_vectored` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ❌ Fail | ERR-003 | |
| `drop` | ✅ Pass | | |
| `simplex` | ✅ Pass | | |
| `new_unsplit` | ✅ Pass | | |
| `close_write` | ✅ Pass | | |
| `close_read` | ✅ Pass | | |
| `poll_read_internal` | ✅ Pass | | |
| `poll_write_internal` | ✅ Pass | | |
| `poll_write_vectored_internal` | ✅ Pass | | |
| `poll_read` | ❌ Fail | ERR-009 | |
| `poll_read` | ❌ Fail | ERR-009 | |
| `poll_write` | ❌ Fail | ERR-009 | |
| `poll_write` | ❌ Fail | ERR-009 | |
| `poll_write_vectored` | ❌ Fail | ERR-009 | |
| `poll_write_vectored` | ❌ Fail | ERR-009 | |
| `is_write_vectored` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ❌ Fail | ERR-003 | |

### io/util/mod.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `poll_proceed_and_make_progress` | ✅ Pass | | |
| `poll_proceed_and_make_progress` | ✅ Pass | | |

### io/util/read.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `read` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-009 | |

### io/util/read_buf.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `read_buf` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-003,ERR-009 | |

### io/util/read_exact.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `read_exact` | ✅ Pass | | |
| `eof` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-009 | |

### io/util/read_int.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-009 | |
| `new` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-009 | |

### io/util/read_line.rs (5 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `read_line` | ✅ Pass | | |
| `put_back_original_data` | ✅ Pass | | |
| `finish_string_read` | ✅ Pass | | |
| `read_line_internal` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-009 | |

### io/util/read_to_end.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `read_to_end` | ✅ Pass | | |
| `read_to_end_internal` | ✅ Pass | | |
| `poll_read_to_end` | ❌ Fail | ERR-003 | |
| `poll` | ❌ Fail | ERR-009 | |

### io/util/read_to_string.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `read_to_string` | ✅ Pass | | |
| `read_to_string_internal` | ❌ Fail | ERR-003 | |
| `poll` | ❌ Fail | ERR-009 | |

### io/util/read_until.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `read_until` | ✅ Pass | | |
| `read_until_internal` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-009 | |

### io/util/repeat.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `repeat` | ✅ Pass | | |
| `poll_read` | ❌ Fail | ERR-003 | |
| `assert_unpin` | ✅ Pass | | |

### io/util/shutdown.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `shutdown` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-009 | |

### io/util/sink.rs (6 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `sink` | ✅ Pass | | |
| `poll_write` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `assert_unpin` | ✅ Pass | | |

### io/util/split.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `split` | ✅ Pass | | |
| `next_segment` | ✅ Pass | | |
| `poll_next_segment` | ❌ Fail | ERR-009 | |
| `assert_unpin` | ✅ Pass | | |

### io/util/take.rs (11 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `take` | ✅ Pass | | |
| `limit` | ✅ Pass | | |
| `set_limit` | ✅ Pass | | |
| `get_ref` | ✅ Pass | | |
| `get_mut` | ✅ Pass | | |
| `get_pin_mut` | ✅ Pass | | |
| `into_inner` | ❌ Fail | ERR-009 | |
| `poll_read` | ❌ Fail | ERR-003,ERR-009 | |
| `poll_fill_buf` | ❌ Fail | ERR-009 | |
| `consume` | ❌ Fail | ERR-009 | |
| `assert_unpin` | ✅ Pass | | |

### io/util/vec_with_initialized.rs (8 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `take` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `reserve` | ❌ Fail | ERR-003 | |
| `is_empty` | ✅ Pass | | |
| `get_read_buf` | ❌ Fail | ERR-003 | |
| `apply_read_buf` | ❌ Fail | ERR-003 | |
| `try_small_read_first` | ✅ Pass | | |
| `into_read_buf_parts` | ✅ Pass | | |

### io/util/write.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `write` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-009 | |

### io/util/write_all.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `write_all` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-009 | |

### io/util/write_all_buf.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `write_all_buf` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-009 | |

### io/util/write_buf.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `write_buf` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-009 | |

### io/util/write_int.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-009 | |
| `new` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-009 | |

### io/util/write_vectored.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `write_vectored` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-009 | |

### lib.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `trace_leaf` | ❌ Fail | ERR-005 | |
| `async_trace_leaf` | ❌ Fail | ERR-005 | |
| `poll` | ❌ Fail | ERR-005 | |
| `is_unpin` | ✅ Pass | | |

### loom/mocked.rs (10 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `lock` | ✅ Pass | | |
| `try_lock` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `read` | ✅ Pass | | |
| `try_read` | ✅ Pass | | |
| `write` | ✅ Pass | | |
| `try_write` | ✅ Pass | | |
| `seed` | ✅ Pass | | |
| `num_cpus` | ✅ Pass | | |

### loom/std/atomic_u16.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `unsync_load` | ✅ Pass | | |
| `deref` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |

### loom/std/atomic_u32.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `unsync_load` | ✅ Pass | | |
| `deref` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |

### loom/std/atomic_u64_as_mutex.rs (7 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `load` | ✅ Pass | | |
| `store` | ✅ Pass | | |
| `fetch_add` | ✅ Pass | | |
| `fetch_or` | ✅ Pass | | |
| `compare_exchange` | ✅ Pass | | |
| `compare_exchange_weak` | ✅ Pass | | |
| `default` | ✅ Pass | | |

### loom/std/atomic_u64_static_once_cell.rs (5 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `load` | ✅ Pass | | |
| `fetch_add` | ✅ Pass | | |
| `compare_exchange_weak` | ✅ Pass | | |
| `inner` | ✅ Pass | | |

### loom/std/atomic_usize.rs (5 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `unsync_load` | ✅ Pass | | |
| `with_mut` | ✅ Pass | | |
| `deref` | ✅ Pass | | |
| `deref_mut` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |

### loom/std/barrier.rs (6 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `fmt` | ❌ Fail | ERR-003 | |
| `new` | ✅ Pass | | |
| `wait` | ✅ Pass | | |
| `wait_timeout` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `is_leader` | ✅ Pass | | |

### loom/std/mod.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `seed` | ✅ Pass | | |
| `num_cpus` | ❌ Fail | ERR-005 | |
| `num_cpus` | ❌ Fail | ERR-005 | |
| `yield_now` | ❌ Fail | ERR-005 | |

### loom/std/mutex.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `lock` | ✅ Pass | | |
| `try_lock` | ✅ Pass | | |

### loom/std/parking_lot.rs (22 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `lock` | ✅ Pass | | |
| `try_lock` | ✅ Pass | | |
| `get_mut` | ✅ Pass | | |
| `deref` | ✅ Pass | | |
| `deref_mut` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `read` | ✅ Pass | | |
| `try_read` | ✅ Pass | | |
| `write` | ✅ Pass | | |
| `try_write` | ✅ Pass | | |
| `deref` | ✅ Pass | | |
| `deref` | ✅ Pass | | |
| `deref_mut` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `notify_one` | ✅ Pass | | |
| `notify_all` | ✅ Pass | | |
| `wait` | ✅ Pass | | |
| `wait_timeout` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |

### loom/std/rwlock.rs (5 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `read` | ✅ Pass | | |
| `try_read` | ✅ Pass | | |
| `write` | ✅ Pass | | |
| `try_write` | ✅ Pass | | |

### loom/std/unsafe_cell.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `with` | ✅ Pass | | |
| `with_mut` | ✅ Pass | | |

### macros/join.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `num_skip` | ✅ Pass | | |
| `num_skip` | ✅ Pass | | |

### macros/support.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `thread_rng_n` | ✅ Pass | | |
| `poll_budget_available` | ✅ Pass | | |
| `poll_budget_available` | ✅ Pass | | |

### net/addr.rs (17 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `to_socket_addrs` | ❌ Fail | E0521,ERR-002,ERR-008 | |
| `to_socket_addrs` | ❌ Fail | E0521,ERR-002,ERR-008 | |
| `to_socket_addrs` | ❌ Fail | E0521,ERR-002,ERR-008 | |
| `to_socket_addrs` | ❌ Fail | E0521,ERR-002,ERR-008 | |
| `to_socket_addrs` | ❌ Fail | E0521,ERR-002,ERR-008 | |
| `to_socket_addrs` | ❌ Fail | E0521,ERR-002,ERR-008 | |
| `to_socket_addrs` | ❌ Fail | E0521,ERR-002,ERR-008 | |
| `to_socket_addrs` | ❌ Fail | E0521,ERR-002,ERR-008 | |
| `to_socket_addrs` | ❌ Fail | E0521,ERR-002,ERR-008 | |
| `slice_to_vec` | ✅ Pass | | |
| `to_socket_addrs` | ❌ Fail | E0521,ERR-002,ERR-008 | |
| `to_socket_addrs` | ❌ Fail | E0521,ERR-002,ERR-008 | |
| `to_socket_addrs` | ❌ Fail | E0521,ERR-002,ERR-008 | |
| `to_socket_addrs` | ❌ Fail | E0521,ERR-002,ERR-008 | |
| `poll` | ❌ Fail | ERR-005 | |
| `next` | ✅ Pass | | |
| `size_hint` | ❌ Fail | ERR-005 | |

### net/lookup_host.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `lookup_host` | ✅ Pass | | |

### net/tcp/listener.rs (18 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `bind` | ✅ Pass | | |
| `bind_addr` | ✅ Pass | | |
| `accept` | ✅ Pass | | |
| `poll_accept` | ✅ Pass | | |
| `from_std` | ✅ Pass | | |
| `into_std` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `local_addr` | ✅ Pass | | |
| `ttl` | ✅ Pass | | |
| `set_ttl` | ✅ Pass | | |
| `try_from` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `as_raw_fd` | ❌ Fail | ERR-005 | |
| `as_fd` | ❌ Fail | ERR-005 | |
| `as_raw_fd` | ❌ Fail | ERR-005 | |
| `as_fd` | ❌ Fail | ERR-005 | |
| `as_raw_socket` | ✅ Pass | | |
| `as_socket` | ✅ Pass | | |

### net/tcp/socket.rs (37 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new_v4` | ✅ Pass | | |
| `new_v6` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `set_keepalive` | ✅ Pass | | |
| `keepalive` | ✅ Pass | | |
| `set_reuseaddr` | ✅ Pass | | |
| `reuseaddr` | ✅ Pass | | |
| `set_reuseport` | ✅ Pass | | |
| `reuseport` | ✅ Pass | | |
| `set_send_buffer_size` | ✅ Pass | | |
| `send_buffer_size` | ✅ Pass | | |
| `set_recv_buffer_size` | ✅ Pass | | |
| `recv_buffer_size` | ✅ Pass | | |
| `set_linger` | ✅ Pass | | |
| `linger` | ✅ Pass | | |
| `set_nodelay` | ✅ Pass | | |
| `nodelay` | ✅ Pass | | |
| `tos` | ✅ Pass | | |
| `set_tos` | ✅ Pass | | |
| `device` | ✅ Pass | | |
| `bind_device` | ✅ Pass | | |
| `local_addr` | ✅ Pass | | |
| `take_error` | ✅ Pass | | |
| `bind` | ❌ Fail | ERR-008 | |
| `connect` | ❌ Fail | ERR-008 | |
| `listen` | ✅ Pass | | |
| `from_std_stream` | ✅ Pass | | |
| `convert_address` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `as_raw_fd` | ✅ Pass | | |
| `as_fd` | ✅ Pass | | |
| `from_raw_fd` | ✅ Pass | | |
| `into_raw_fd` | ✅ Pass | | |
| `into_raw_socket` | ✅ Pass | | |
| `as_raw_socket` | ✅ Pass | | |
| `as_socket` | ✅ Pass | | |
| `from_raw_socket` | ✅ Pass | | |

### net/tcp/split.rs (24 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `split` | ✅ Pass | | |
| `poll_peek` | ✅ Pass | | |
| `peek` | ✅ Pass | | |
| `ready` | ✅ Pass | | |
| `readable` | ✅ Pass | | |
| `try_read` | ✅ Pass | | |
| `try_read_vectored` | ✅ Pass | | |
| `try_read_buf` | ✅ Pass | | |
| `peer_addr` | ✅ Pass | | |
| `local_addr` | ✅ Pass | | |
| `ready` | ✅ Pass | | |
| `writable` | ✅ Pass | | |
| `try_write` | ✅ Pass | | |
| `try_write_vectored` | ✅ Pass | | |
| `peer_addr` | ✅ Pass | | |
| `local_addr` | ✅ Pass | | |
| `poll_read` | ✅ Pass | | |
| `poll_write` | ✅ Pass | | |
| `poll_write_vectored` | ✅ Pass | | |
| `is_write_vectored` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ✅ Pass | | |
| `as_ref` | ✅ Pass | | |
| `as_ref` | ✅ Pass | | |

### net/tcp/split_owned.rs (30 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `split_owned` | ✅ Pass | | |
| `reunite` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `reunite` | ✅ Pass | | |
| `poll_peek` | ✅ Pass | | |
| `peek` | ✅ Pass | | |
| `ready` | ✅ Pass | | |
| `readable` | ✅ Pass | | |
| `try_read` | ✅ Pass | | |
| `try_read_vectored` | ✅ Pass | | |
| `try_read_buf` | ✅ Pass | | |
| `peer_addr` | ✅ Pass | | |
| `local_addr` | ✅ Pass | | |
| `poll_read` | ✅ Pass | | |
| `reunite` | ✅ Pass | | |
| `forget` | ✅ Pass | | |
| `ready` | ✅ Pass | | |
| `writable` | ✅ Pass | | |
| `try_write` | ✅ Pass | | |
| `try_write_vectored` | ✅ Pass | | |
| `peer_addr` | ✅ Pass | | |
| `local_addr` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `poll_write` | ✅ Pass | | |
| `poll_write_vectored` | ✅ Pass | | |
| `is_write_vectored` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ✅ Pass | | |
| `as_ref` | ❌ Fail | ERR-008 | |
| `as_ref` | ❌ Fail | ERR-008 | |

### net/tcp/stream.rs (52 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `connect` | ✅ Pass | | |
| `connect_addr` | ✅ Pass | | |
| `connect_mio` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `from_std` | ✅ Pass | | |
| `into_std` | ✅ Pass | | |
| `local_addr` | ✅ Pass | | |
| `take_error` | ✅ Pass | | |
| `peer_addr` | ✅ Pass | | |
| `poll_peek` | ✅ Pass | | |
| `ready` | ✅ Pass | | |
| `readable` | ✅ Pass | | |
| `poll_read_ready` | ✅ Pass | | |
| `try_read` | ❌ Fail | ERR-010 | |
| `try_read_vectored` | ✅ Pass | | |
| `try_read_buf` | ❌ Fail | ERR-003 | |
| `writable` | ✅ Pass | | |
| `poll_write_ready` | ✅ Pass | | |
| `try_write` | ❌ Fail | ERR-010 | |
| `try_write_vectored` | ✅ Pass | | |
| `try_io` | ✅ Pass | | |
| `async_io` | ✅ Pass | | |
| `peek` | ✅ Pass | | |
| `shutdown_std` | ✅ Pass | | |
| `nodelay` | ✅ Pass | | |
| `set_nodelay` | ✅ Pass | | |
| `quickack` | ✅ Pass | | |
| `set_quickack` | ✅ Pass | | |
| `linger` | ✅ Pass | | |
| `set_linger` | ✅ Pass | | |
| `ttl` | ✅ Pass | | |
| `set_ttl` | ✅ Pass | | |
| `split` | ✅ Pass | | |
| `into_split` | ✅ Pass | | |
| `poll_read_priv` | ✅ Pass | | |
| `poll_write_priv` | ✅ Pass | | |
| `poll_write_vectored_priv` | ✅ Pass | | |
| `try_from` | ✅ Pass | | |
| `poll_read` | ✅ Pass | | |
| `poll_write` | ✅ Pass | | |
| `poll_write_vectored` | ✅ Pass | | |
| `is_write_vectored` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `as_ref` | ✅ Pass | | |
| `as_raw_fd` | ✅ Pass | | |
| `as_fd` | ❌ Fail | ERR-005 | |
| `as_raw_socket` | ✅ Pass | | |
| `as_socket` | ✅ Pass | | |
| `as_raw_fd` | ✅ Pass | | |
| `as_fd` | ❌ Fail | ERR-005 | |

### net/udp.rs (68 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `bind` | ✅ Pass | | |
| `bind_addr` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `from_std` | ✅ Pass | | |
| `into_std` | ✅ Pass | | |
| `as_socket` | ❌ Fail | ERR-005 | |
| `local_addr` | ✅ Pass | | |
| `peer_addr` | ✅ Pass | | |
| `connect` | ✅ Pass | | |
| `ready` | ✅ Pass | | |
| `writable` | ✅ Pass | | |
| `poll_send_ready` | ✅ Pass | | |
| `send` | ✅ Pass | | |
| `poll_send` | ✅ Pass | | |
| `try_send` | ✅ Pass | | |
| `readable` | ✅ Pass | | |
| `poll_recv_ready` | ✅ Pass | | |
| `recv` | ✅ Pass | | |
| `poll_recv` | ❌ Fail | ERR-003 | |
| `try_recv` | ✅ Pass | | |
| `try_recv_buf` | ❌ Fail | ERR-003 | |
| `recv_buf` | ✅ Pass | | |
| `try_recv_buf_from` | ❌ Fail | ERR-002 | |
| `recv_buf_from` | ✅ Pass | | |
| `send_to` | ✅ Pass | | |
| `poll_send_to` | ✅ Pass | | |
| `try_send_to` | ✅ Pass | | |
| `send_to_addr` | ✅ Pass | | |
| `recv_from` | ✅ Pass | | |
| `poll_recv_from` | ❌ Fail | ERR-002 | |
| `try_recv_from` | ✅ Pass | | |
| `try_io` | ✅ Pass | | |
| `async_io` | ✅ Pass | | |
| `peek` | ✅ Pass | | |
| `poll_peek` | ❌ Fail | ERR-003 | |
| `try_peek` | ✅ Pass | | |
| `peek_from` | ✅ Pass | | |
| `poll_peek_from` | ✅ Pass | | |
| `try_peek_from` | ❌ Fail | ERR-002 | |
| `peek_sender` | ✅ Pass | | |
| `poll_peek_sender` | ✅ Pass | | |
| `try_peek_sender` | ✅ Pass | | |
| `peek_sender_inner` | ✅ Pass | | |
| `broadcast` | ✅ Pass | | |
| `set_broadcast` | ✅ Pass | | |
| `multicast_loop_v4` | ✅ Pass | | |
| `set_multicast_loop_v4` | ✅ Pass | | |
| `multicast_ttl_v4` | ✅ Pass | | |
| `set_multicast_ttl_v4` | ✅ Pass | | |
| `multicast_loop_v6` | ✅ Pass | | |
| `set_multicast_loop_v6` | ✅ Pass | | |
| `ttl` | ✅ Pass | | |
| `set_ttl` | ✅ Pass | | |
| `tos` | ✅ Pass | | |
| `set_tos` | ✅ Pass | | |
| `device` | ✅ Pass | | |
| `bind_device` | ✅ Pass | | |
| `join_multicast_v4` | ✅ Pass | | |
| `join_multicast_v6` | ✅ Pass | | |
| `leave_multicast_v4` | ✅ Pass | | |
| `leave_multicast_v6` | ✅ Pass | | |
| `take_error` | ✅ Pass | | |
| `try_from` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `as_raw_fd` | ✅ Pass | | |
| `as_fd` | ✅ Pass | | |
| `as_raw_socket` | ✅ Pass | | |
| `as_socket` | ❌ Fail | ERR-005 | |

### net/unix/datagram/socket.rs (39 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `from_mio` | ✅ Pass | | |
| `ready` | ✅ Pass | | |
| `writable` | ✅ Pass | | |
| `poll_send_ready` | ✅ Pass | | |
| `readable` | ✅ Pass | | |
| `poll_recv_ready` | ✅ Pass | | |
| `bind` | ✅ Pass | | |
| `pair` | ✅ Pass | | |
| `from_std` | ✅ Pass | | |
| `into_std` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `unbound` | ✅ Pass | | |
| `connect` | ✅ Pass | | |
| `send` | ✅ Pass | | |
| `try_send` | ✅ Pass | | |
| `try_send_to` | ✅ Pass | | |
| `recv` | ✅ Pass | | |
| `try_recv` | ✅ Pass | | |
| `try_recv_buf_from` | ✅ Pass | | |
| `recv_buf_from` | ✅ Pass | | |
| `try_recv_buf` | ❌ Fail | ERR-003 | |
| `recv_buf` | ✅ Pass | | |
| `send_to` | ✅ Pass | | |
| `recv_from` | ✅ Pass | | |
| `poll_recv_from` | ✅ Pass | | |
| `poll_send_to` | ❌ Fail | ERR-002 | |
| `poll_send` | ✅ Pass | | |
| `poll_recv` | ❌ Fail | ERR-003 | |
| `try_recv_from` | ✅ Pass | | |
| `try_io` | ✅ Pass | | |
| `async_io` | ✅ Pass | | |
| `local_addr` | ✅ Pass | | |
| `peer_addr` | ✅ Pass | | |
| `take_error` | ✅ Pass | | |
| `shutdown` | ✅ Pass | | |
| `try_from` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `as_raw_fd` | ✅ Pass | | |
| `as_fd` | ✅ Pass | | |

### net/unix/listener.rs (12 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `bind` | ✅ Pass | | |
| `from_std` | ✅ Pass | | |
| `into_std` | ✅ Pass | | |
| `local_addr` | ✅ Pass | | |
| `take_error` | ✅ Pass | | |
| `accept` | ✅ Pass | | |
| `poll_accept` | ✅ Pass | | |
| `try_from` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `as_raw_fd` | ✅ Pass | | |
| `as_fd` | ✅ Pass | | |

### net/unix/pipe.rs (49 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `pipe` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `read_write` | ✅ Pass | | |
| `unchecked` | ✅ Pass | | |
| `open_receiver` | ✅ Pass | | |
| `open_sender` | ✅ Pass | | |
| `open` | ❌ Fail | ERR-010 | |
| `default` | ✅ Pass | | |
| `from_mio` | ✅ Pass | | |
| `from_file` | ❌ Fail | ERR-008 | |
| `from_owned_fd` | ❌ Fail | ERR-008 | |
| `from_file_unchecked` | ❌ Fail | ERR-008 | |
| `from_owned_fd_unchecked` | ❌ Fail | ERR-008 | |
| `ready` | ✅ Pass | | |
| `writable` | ✅ Pass | | |
| `poll_write_ready` | ✅ Pass | | |
| `try_write` | ❌ Fail | ERR-010 | |
| `try_write_vectored` | ✅ Pass | | |
| `into_blocking_fd` | ✅ Pass | | |
| `into_nonblocking_fd` | ❌ Fail | ERR-003 | |
| `poll_write` | ✅ Pass | | |
| `poll_write_vectored` | ✅ Pass | | |
| `is_write_vectored` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ✅ Pass | | |
| `as_raw_fd` | ✅ Pass | | |
| `as_fd` | ✅ Pass | | |
| `from_mio` | ✅ Pass | | |
| `from_file` | ❌ Fail | ERR-008 | |
| `from_owned_fd` | ❌ Fail | ERR-008 | |
| `from_file_unchecked` | ❌ Fail | ERR-008 | |
| `from_owned_fd_unchecked` | ❌ Fail | ERR-008 | |
| `ready` | ✅ Pass | | |
| `readable` | ✅ Pass | | |
| `poll_read_ready` | ✅ Pass | | |
| `try_read` | ❌ Fail | ERR-010 | |
| `try_read_vectored` | ✅ Pass | | |
| `try_read_buf` | ✅ Pass | | |
| `into_blocking_fd` | ✅ Pass | | |
| `into_nonblocking_fd` | ❌ Fail | ERR-003 | |
| `poll_read` | ✅ Pass | | |
| `as_raw_fd` | ✅ Pass | | |
| `as_fd` | ✅ Pass | | |
| `is_pipe` | ✅ Pass | | |
| `get_file_flags` | ✅ Pass | | |
| `has_read_access` | ✅ Pass | | |
| `has_write_access` | ✅ Pass | | |
| `set_nonblocking` | ✅ Pass | | |
| `set_blocking` | ✅ Pass | | |

### net/unix/socket.rs (12 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `ty` | ✅ Pass | | |
| `new_datagram` | ✅ Pass | | |
| `new_stream` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `bind` | ✅ Pass | | |
| `listen` | ✅ Pass | | |
| `connect` | ✅ Pass | | |
| `datagram` | ✅ Pass | | |
| `as_raw_fd` | ✅ Pass | | |
| `as_fd` | ✅ Pass | | |
| `from_raw_fd` | ✅ Pass | | |
| `into_raw_fd` | ✅ Pass | | |

### net/unix/socketaddr.rs (6 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `is_unnamed` | ✅ Pass | | |
| `as_pathname` | ✅ Pass | | |
| `as_abstract_name` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `from` | ✅ Pass | | |
| `from` | ✅ Pass | | |

### net/unix/split.rs (22 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `split` | ✅ Pass | | |
| `ready` | ✅ Pass | | |
| `readable` | ✅ Pass | | |
| `try_read` | ✅ Pass | | |
| `try_read_buf` | ✅ Pass | | |
| `try_read_vectored` | ✅ Pass | | |
| `peer_addr` | ✅ Pass | | |
| `local_addr` | ✅ Pass | | |
| `ready` | ✅ Pass | | |
| `writable` | ✅ Pass | | |
| `try_write` | ✅ Pass | | |
| `try_write_vectored` | ✅ Pass | | |
| `peer_addr` | ✅ Pass | | |
| `local_addr` | ✅ Pass | | |
| `poll_read` | ✅ Pass | | |
| `poll_write` | ✅ Pass | | |
| `poll_write_vectored` | ✅ Pass | | |
| `is_write_vectored` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ✅ Pass | | |
| `as_ref` | ✅ Pass | | |
| `as_ref` | ✅ Pass | | |

### net/unix/split_owned.rs (28 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `split_owned` | ✅ Pass | | |
| `reunite` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `reunite` | ✅ Pass | | |
| `ready` | ✅ Pass | | |
| `readable` | ✅ Pass | | |
| `try_read` | ✅ Pass | | |
| `try_read_buf` | ✅ Pass | | |
| `try_read_vectored` | ✅ Pass | | |
| `peer_addr` | ✅ Pass | | |
| `local_addr` | ✅ Pass | | |
| `poll_read` | ✅ Pass | | |
| `reunite` | ✅ Pass | | |
| `forget` | ✅ Pass | | |
| `ready` | ✅ Pass | | |
| `writable` | ✅ Pass | | |
| `try_write` | ✅ Pass | | |
| `try_write_vectored` | ✅ Pass | | |
| `peer_addr` | ✅ Pass | | |
| `local_addr` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `poll_write` | ✅ Pass | | |
| `poll_write_vectored` | ✅ Pass | | |
| `is_write_vectored` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ✅ Pass | | |
| `as_ref` | ❌ Fail | ERR-008 | |
| `as_ref` | ❌ Fail | ERR-008 | |

### net/unix/stream.rs (39 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `connect_mio` | ✅ Pass | | |
| `connect` | ✅ Pass | | |
| `ready` | ✅ Pass | | |
| `readable` | ✅ Pass | | |
| `poll_read_ready` | ✅ Pass | | |
| `try_read` | ❌ Fail | ERR-010 | |
| `try_read_vectored` | ✅ Pass | | |
| `try_read_buf` | ❌ Fail | ERR-003 | |
| `writable` | ✅ Pass | | |
| `poll_write_ready` | ✅ Pass | | |
| `try_write` | ❌ Fail | ERR-010 | |
| `try_write_vectored` | ✅ Pass | | |
| `try_io` | ✅ Pass | | |
| `async_io` | ✅ Pass | | |
| `from_std` | ✅ Pass | | |
| `into_std` | ✅ Pass | | |
| `pair` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `local_addr` | ✅ Pass | | |
| `peer_addr` | ✅ Pass | | |
| `peer_cred` | ✅ Pass | | |
| `take_error` | ✅ Pass | | |
| `shutdown_std` | ✅ Pass | | |
| `split` | ✅ Pass | | |
| `into_split` | ✅ Pass | | |
| `try_from` | ✅ Pass | | |
| `poll_read` | ✅ Pass | | |
| `poll_write` | ✅ Pass | | |
| `poll_write_vectored` | ✅ Pass | | |
| `is_write_vectored` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ✅ Pass | | |
| `poll_read_priv` | ✅ Pass | | |
| `poll_write_priv` | ✅ Pass | | |
| `poll_write_vectored_priv` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `as_ref` | ✅ Pass | | |
| `as_raw_fd` | ✅ Pass | | |
| `as_fd` | ✅ Pass | | |

### net/unix/ucred.rs (10 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `uid` | ✅ Pass | | |
| `gid` | ✅ Pass | | |
| `pid` | ✅ Pass | | |
| `get_peer_cred` | ❌ Fail | ERR-005 | |
| `get_peer_cred` | ❌ Fail | ERR-005 | |
| `get_peer_cred` | ❌ Fail | ERR-005 | |
| `get_peer_cred` | ❌ Fail | ERR-005 | |
| `get_peer_cred` | ❌ Fail | ERR-005 | |
| `get_peer_cred` | ❌ Fail | ERR-005 | |
| `get_peer_cred` | ❌ Fail | ERR-005 | |

### net/windows/named_pipe.rs (68 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `from_raw_handle` | ✅ Pass | | |
| `info` | ✅ Pass | | |
| `connect` | ✅ Pass | | |
| `disconnect` | ✅ Pass | | |
| `ready` | ✅ Pass | | |
| `readable` | ✅ Pass | | |
| `poll_read_ready` | ✅ Pass | | |
| `try_read` | ✅ Pass | | |
| `try_read_vectored` | ✅ Pass | | |
| `try_read_buf` | ✅ Pass | | |
| `writable` | ✅ Pass | | |
| `poll_write_ready` | ✅ Pass | | |
| `try_write` | ✅ Pass | | |
| `try_write_vectored` | ✅ Pass | | |
| `try_io` | ✅ Pass | | |
| `async_io` | ✅ Pass | | |
| `poll_read` | ✅ Pass | | |
| `poll_write` | ✅ Pass | | |
| `poll_write_vectored` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ✅ Pass | | |
| `as_raw_handle` | ✅ Pass | | |
| `as_handle` | ✅ Pass | | |
| `from_raw_handle` | ✅ Pass | | |
| `info` | ✅ Pass | | |
| `ready` | ✅ Pass | | |
| `readable` | ✅ Pass | | |
| `poll_read_ready` | ✅ Pass | | |
| `try_read` | ✅ Pass | | |
| `try_read_vectored` | ✅ Pass | | |
| `try_read_buf` | ✅ Pass | | |
| `writable` | ✅ Pass | | |
| `poll_write_ready` | ✅ Pass | | |
| `try_write` | ✅ Pass | | |
| `try_write_vectored` | ✅ Pass | | |
| `try_io` | ✅ Pass | | |
| `async_io` | ✅ Pass | | |
| `poll_read` | ✅ Pass | | |
| `poll_write` | ✅ Pass | | |
| `poll_write_vectored` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ✅ Pass | | |
| `as_raw_handle` | ✅ Pass | | |
| `as_handle` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `pipe_mode` | ✅ Pass | | |
| `access_inbound` | ✅ Pass | | |
| `access_outbound` | ✅ Pass | | |
| `first_pipe_instance` | ✅ Pass | | |
| `write_dac` | ✅ Pass | | |
| `write_owner` | ✅ Pass | | |
| `access_system_security` | ✅ Pass | | |
| `reject_remote_clients` | ✅ Pass | | |
| `max_instances` | ✅ Pass | | |
| `out_buffer_size` | ✅ Pass | | |
| `in_buffer_size` | ✅ Pass | | |
| `create` | ✅ Pass | | |
| `create_with_security_attributes_raw` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `read` | ✅ Pass | | |
| `write` | ✅ Pass | | |
| `security_qos_flags` | ✅ Pass | | |
| `pipe_mode` | ✅ Pass | | |
| `open` | ✅ Pass | | |
| `open_with_security_attributes_raw` | ✅ Pass | | |
| `get_flags` | ✅ Pass | | |
| `encode_addr` | ✅ Pass | | |
| `named_pipe_info` | ✅ Pass | | |

### process/kill.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `kill` | ✅ Pass | | |

### process/mod.rs (70 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `as_std` | ✅ Pass | | |
| `as_std_mut` | ✅ Pass | | |
| `into_std` | ✅ Pass | | |
| `arg` | ✅ Pass | | |
| `args` | ✅ Pass | | |
| `raw_arg` | ✅ Pass | | |
| `env` | ✅ Pass | | |
| `envs` | ✅ Pass | | |
| `env_remove` | ✅ Pass | | |
| `env_clear` | ✅ Pass | | |
| `current_dir` | ✅ Pass | | |
| `stdin` | ✅ Pass | | |
| `stdout` | ✅ Pass | | |
| `stderr` | ✅ Pass | | |
| `kill_on_drop` | ✅ Pass | | |
| `creation_flags` | ✅ Pass | | |
| `uid` | ✅ Pass | | |
| `gid` | ✅ Pass | | |
| `arg0` | ✅ Pass | | |
| `pre_exec` | ✅ Pass | | |
| `process_group` | ✅ Pass | | |
| `spawn` | ✅ Pass | | |
| `spawn_with` | ✅ Pass | | |
| `build_child` | ✅ Pass | | |
| `status` | ❌ Fail | ERR-003 | |
| `output` | ✅ Pass | | |
| `get_kill_on_drop` | ❌ Fail | ERR-003 | |
| `from` | ✅ Pass | | |
| `kill` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `poll` | ✅ Pass | | |
| `id` | ✅ Pass | | |
| `raw_handle` | ✅ Pass | | |
| `start_kill` | ✅ Pass | | |
| `kill` | ✅ Pass | | |
| `wait` | ✅ Pass | | |
| `try_wait` | ✅ Pass | | |
| `wait_with_output` | ✅ Pass | | |
| `read_to_end` | ❌ Fail | ERR-003 | |
| `from_std` | ✅ Pass | | |
| `from_std` | ✅ Pass | | |
| `from_std` | ✅ Pass | | |
| `poll_write` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ✅ Pass | | |
| `poll_write_vectored` | ✅ Pass | | |
| `is_write_vectored` | ✅ Pass | | |
| `poll_read` | ✅ Pass | | |
| `poll_read` | ✅ Pass | | |
| `try_into` | ✅ Pass | | |
| `try_into` | ✅ Pass | | |
| `try_into` | ✅ Pass | | |
| `into_owned_fd` | ✅ Pass | | |
| `as_raw_fd` | ✅ Pass | | |
| `as_fd` | ✅ Pass | | |
| `into_owned_handle` | ❌ Fail | ERR-005 | |
| `as_raw_handle` | ❌ Fail | ERR-005 | |
| `as_handle` | ❌ Fail | ERR-005 | |
| `into_owned_handle` | ❌ Fail | ERR-005 | |
| `as_raw_handle` | ❌ Fail | ERR-005 | |
| `as_handle` | ❌ Fail | ERR-005 | |
| `new` | ✅ Pass | | |
| `with_result` | ✅ Pass | | |
| `kill` | ✅ Pass | | |
| `poll` | ✅ Pass | | |
| `kills_on_drop_if_specified` | ✅ Pass | | |
| `no_kill_on_drop_by_default` | ✅ Pass | | |
| `no_kill_if_already_killed` | ✅ Pass | | |
| `no_kill_if_reaped` | ✅ Pass | | |

### process/unix/mod.rs (39 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `id` | ✅ Pass | | |
| `try_wait` | ❌ Fail | ERR-003 | |
| `kill` | ❌ Fail | ERR-003 | |
| `get_orphan_queue` | ✅ Pass | | |
| `get_orphan_queue` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `reap_orphans` | ✅ Pass | | |
| `push_orphan` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `build_child` | ❌ Fail | ERR-003 | |
| `id` | ✅ Pass | | |
| `std_child` | ✅ Pass | | |
| `try_wait` | ❌ Fail | ERR-003 | |
| `kill` | ❌ Fail | ERR-003 | |
| `poll` | ❌ Fail | ERR-003 | |
| `from` | ✅ Pass | | |
| `read` | ✅ Pass | | |
| `write` | ✅ Pass | | |
| `flush` | ✅ Pass | | |
| `write_vectored` | ❌ Fail | ERR-010 | |
| `as_raw_fd` | ✅ Pass | | |
| `as_fd` | ❌ Fail | ERR-010 | |
| `convert_to_blocking_file` | ✅ Pass | | |
| `convert_to_stdio` | ✅ Pass | | |
| `register` | ✅ Pass | | |
| `reregister` | ✅ Pass | | |
| `deregister` | ✅ Pass | | |
| `into_owned_fd` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `as_raw_fd` | ✅ Pass | | |
| `as_fd` | ❌ Fail | ERR-010 | |
| `poll_write` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ✅ Pass | | |
| `poll_write_vectored` | ✅ Pass | | |
| `is_write_vectored` | ✅ Pass | | |
| `poll_read` | ✅ Pass | | |
| `set_nonblocking` | ✅ Pass | | |
| `stdio` | ✅ Pass | | |

### process/unix/orphan.rs (19 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `id` | ✅ Pass | | |
| `try_wait` | ✅ Pass | | |
| `push_orphan` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `len` | ✅ Pass | | |
| `push_orphan` | ✅ Pass | | |
| `reap_orphans` | ✅ Pass | | |
| `drain_orphan_queue` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `push_orphan` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `with_err` | ✅ Pass | | |
| `id` | ✅ Pass | | |
| `try_wait` | ✅ Pass | | |
| `drain_attempts_a_single_reap_of_all_queued_orphans` | ✅ Pass | | |
| `no_reap_if_no_signal_received` | ✅ Pass | | |
| `no_reap_if_signal_lock_held` | ✅ Pass | | |
| `does_not_register_signal_if_queue_empty` | ✅ Pass | | |
| `does_nothing_if_signal_could_not_be_registered` | ✅ Pass | | |

### process/unix/pidfd_reaper.rs (21 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `open` | ✅ Pass | | |
| `as_raw_fd` | ✅ Pass | | |
| `register` | ✅ Pass | | |
| `reregister` | ✅ Pass | | |
| `deregister` | ✅ Pass | | |
| `display_eq` | ✅ Pass | | |
| `write_str` | ✅ Pass | | |
| `is_rt_shutdown_err` | ✅ Pass | | |
| `poll` | ✅ Pass | | |
| `deref` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `inner_mut` | ✅ Pass | | |
| `poll` | ✅ Pass | | |
| `kill` | ✅ Pass | | |
| `drop` | ❌ Fail | ERR-003 | |
| `create_runtime` | ✅ Pass | | |
| `run_test` | ✅ Pass | | |
| `is_pidfd_available` | ✅ Pass | | |
| `test_pidfd_reaper_poll` | ✅ Pass | | |
| `test_pidfd_reaper_kill` | ✅ Pass | | |
| `test_pidfd_reaper_drop` | ✅ Pass | | |

### process/unix/reap.rs (17 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `deref` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `inner` | ✅ Pass | | |
| `inner_mut` | ✅ Pass | | |
| `poll` | ✅ Pass | | |
| `kill` | ❌ Fail | ERR-003 | |
| `drop` | ❌ Fail | ERR-003 | |
| `new` | ✅ Pass | | |
| `id` | ✅ Pass | | |
| `try_wait` | ✅ Pass | | |
| `kill` | ❌ Fail | ERR-003 | |
| `new` | ✅ Pass | | |
| `poll_recv` | ✅ Pass | | |
| `reaper` | ✅ Pass | | |
| `kill` | ❌ Fail | ERR-003 | |
| `drop_reaps_if_possible` | ✅ Pass | | |
| `drop_enqueues_orphan_if_wait_fails` | ✅ Pass | | |

### process/windows.rs (22 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `fmt` | ✅ Pass | | |
| `build_child` | ✅ Pass | | |
| `id` | ✅ Pass | | |
| `try_wait` | ✅ Pass | | |
| `kill` | ✅ Pass | | |
| `poll` | ✅ Pass | | |
| `as_raw_handle` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `callback` | ✅ Pass | | |
| `read` | ✅ Pass | | |
| `write` | ✅ Pass | | |
| `flush` | ✅ Pass | | |
| `into_owned_handle` | ✅ Pass | | |
| `as_raw_handle` | ✅ Pass | | |
| `poll_read` | ✅ Pass | | |
| `poll_write` | ✅ Pass | | |
| `poll_flush` | ✅ Pass | | |
| `poll_shutdown` | ✅ Pass | | |
| `stdio` | ✅ Pass | | |
| `convert_to_file` | ✅ Pass | | |
| `convert_to_stdio` | ✅ Pass | | |
| `duplicate_handle` | ✅ Pass | | |

### runtime/blocking/mod.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `create_blocking_pool` | ✅ Pass | | |

### runtime/blocking/pool.rs (31 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `num_threads` | ❌ Fail | ERR-009 | |
| `num_idle_threads` | ✅ Pass | | |
| `queue_depth` | ✅ Pass | | |
| `inc_num_threads` | ✅ Pass | | |
| `dec_num_threads` | ✅ Pass | | |
| `inc_num_idle_threads` | ✅ Pass | | |
| `dec_num_idle_threads` | ✅ Pass | | |
| `inc_queue_depth` | ✅ Pass | | |
| `dec_queue_depth` | ✅ Pass | | |
| `from` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `run` | ✅ Pass | | |
| `shutdown_or_run_if_mandatory` | ✅ Pass | | |
| `spawn_blocking` | ❌ Fail | ERR-003 | |
| `spawn_mandatory_blocking` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `spawner` | ✅ Pass | | |
| `shutdown` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `spawn_blocking` | ❌ Fail | ERR-003 | |
| `spawn_mandatory_blocking` | ✅ Pass | | |
| `spawn_blocking_inner` | ✅ Pass | | |
| `spawn_task` | ✅ Pass | | |
| `spawn_thread` | ✅ Pass | | |
| `num_threads` | ❌ Fail | ERR-009 | |
| `num_idle_threads` | ✅ Pass | | |
| `queue_depth` | ✅ Pass | | |
| `is_temporary_os_thread_error` | ✅ Pass | | |
| `run` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |

### runtime/blocking/schedule.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ❌ Fail | ERR-005 | |
| `release` | ❌ Fail | ERR-005 | |
| `schedule` | ❌ Fail | ERR-005 | |
| `hooks` | ❌ Fail | ERR-005 | |

### runtime/blocking/shutdown.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `channel` | ✅ Pass | | |
| `wait` | ✅ Pass | | |

### runtime/blocking/task.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `poll` | ✅ Pass | | |

### runtime/builder.rs (44 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new_current_thread` | ✅ Pass | | |
| `new_multi_thread` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `enable_all` | ❌ Fail | ERR-003 | |
| `enable_alt_timer` | ✅ Pass | | |
| `worker_threads` | ✅ Pass | | |
| `max_blocking_threads` | ✅ Pass | | |
| `thread_name` | ❌ Fail | ERR-008 | |
| `thread_name_fn` | ✅ Pass | | |
| `thread_stack_size` | ✅ Pass | | |
| `on_thread_start` | ✅ Pass | | |
| `on_thread_stop` | ✅ Pass | | |
| `on_thread_park` | ✅ Pass | | |
| `on_thread_unpark` | ✅ Pass | | |
| `on_task_spawn` | ✅ Pass | | |
| `on_before_task_poll` | ✅ Pass | | |
| `on_after_task_poll` | ✅ Pass | | |
| `on_task_terminate` | ✅ Pass | | |
| `build` | ✅ Pass | | |
| `build_local` | ✅ Pass | | |
| `get_cfg` | ✅ Pass | | |
| `thread_keep_alive` | ✅ Pass | | |
| `global_queue_interval` | ✅ Pass | | |
| `event_interval` | ✅ Pass | | |
| `unhandled_panic` | ✅ Pass | | |
| `disable_lifo_slot` | ✅ Pass | | |
| `rng_seed` | ✅ Pass | | |
| `enable_metrics_poll_time_histogram` | ✅ Pass | | |
| `enable_metrics_poll_count_histogram` | ✅ Pass | | |
| `metrics_poll_count_histogram_scale` | ✅ Pass | | |
| `metrics_poll_time_histogram_configuration` | ✅ Pass | | |
| `metrics_poll_count_histogram_resolution` | ✅ Pass | | |
| `metrics_poll_count_histogram_buckets` | ✅ Pass | | |
| `build_current_thread_runtime` | ✅ Pass | | |
| `build_current_thread_local_runtime` | ✅ Pass | | |
| `build_current_thread_runtime_components` | ✅ Pass | | |
| `metrics_poll_count_histogram_builder` | ✅ Pass | | |
| `enable_io` | ✅ Pass | | |
| `max_io_events_per_tick` | ✅ Pass | | |
| `enable_time` | ✅ Pass | | |
| `enable_io_uring` | ✅ Pass | | |
| `start_paused` | ✅ Pass | | |
| `build_threaded_runtime` | ❌ Fail | ERR-009 | |
| `fmt` | ❌ Fail | ERR-003 | |

### runtime/context.rs (9 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `thread_rng_n` | ❌ Fail | ERR-003 | |
| `budget` | ✅ Pass | | |
| `thread_id` | ✅ Pass | | |
| `set_current_task_id` | ✅ Pass | | |
| `current_task_id` | ✅ Pass | | |
| `defer` | ✅ Pass | | |
| `set_scheduler` | ✅ Pass | | |
| `with_scheduler` | ✅ Pass | | |
| `with_trace` | ✅ Pass | | |

### runtime/context/blocking.rs (6 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `try_enter_blocking_region` | ✅ Pass | | |
| `disallow_block_in_place` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `block_on` | ❌ Fail | ERR-003 | |
| `block_on_timeout` | ✅ Pass | | |
| `drop` | ✅ Pass | | |

### runtime/context/current.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `try_set_current` | ❌ Fail | ERR-003 | |
| `with_current` | ✅ Pass | | |
| `set_current` | ✅ Pass | | |
| `drop` | ✅ Pass | | |

### runtime/context/runtime.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `enter_runtime` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `drop` | ❌ Fail | ERR-003 | |
| `is_entered` | ✅ Pass | | |

### runtime/context/runtime_mt.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `current_enter_context` | ✅ Pass | | |
| `exit_runtime` | ✅ Pass | | |
| `drop` | ✅ Pass | | |

### runtime/context/scoped.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `set` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `with` | ✅ Pass | | |

### runtime/driver.rs (32 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `park` | ✅ Pass | | |
| `park_timeout` | ✅ Pass | | |
| `shutdown` | ✅ Pass | | |
| `unpark` | ✅ Pass | | |
| `io` | ✅ Pass | | |
| `signal` | ✅ Pass | | |
| `time` | ✅ Pass | | |
| `with_time` | ✅ Pass | | |
| `clock` | ✅ Pass | | |
| `create_io_stack` | ✅ Pass | | |
| `park` | ✅ Pass | | |
| `park_timeout` | ✅ Pass | | |
| `shutdown` | ✅ Pass | | |
| `unpark` | ✅ Pass | | |
| `as_ref` | ✅ Pass | | |
| `create_io_stack` | ✅ Pass | | |
| `park` | ✅ Pass | | |
| `park_timeout` | ✅ Pass | | |
| `shutdown` | ✅ Pass | | |
| `is_enabled` | ✅ Pass | | |
| `create_signal_driver` | ✅ Pass | | |
| `create_signal_driver` | ✅ Pass | | |
| `create_process_driver` | ✅ Pass | | |
| `create_process_driver` | ✅ Pass | | |
| `create_clock` | ✅ Pass | | |
| `create_time_driver` | ✅ Pass | | |
| `park` | ✅ Pass | | |
| `park_timeout` | ✅ Pass | | |
| `shutdown` | ✅ Pass | | |
| `create_clock` | ✅ Pass | | |
| `create_time_driver` | ✅ Pass | | |

### runtime/driver/op.rs (5 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `take_data` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `from` | ✅ Pass | | |
| `poll` | ✅ Pass | | |

### runtime/dump.rs (22 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `from_backtrace_symbol` | ✅ Pass | | |
| `name_raw` | ✅ Pass | | |
| `name_demangled` | ✅ Pass | | |
| `addr` | ✅ Pass | | |
| `filename` | ✅ Pass | | |
| `lineno` | ✅ Pass | | |
| `colno` | ✅ Pass | | |
| `from_resolved_backtrace_frame` | ✅ Pass | | |
| `ip` | ✅ Pass | | |
| `symbol_address` | ✅ Pass | | |
| `symbols` | ✅ Pass | | |
| `frames` | ✅ Pass | | |
| `resolve_backtraces` | ✅ Pass | | |
| `capture` | ✅ Pass | | |
| `root` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `tasks` | ✅ Pass | | |
| `iter` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `id` | ✅ Pass | | |
| `trace` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |

### runtime/handle.rs (21 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `enter` | ✅ Pass | | |
| `current` | ✅ Pass | | |
| `try_current` | ✅ Pass | | |
| `spawn` | ✅ Pass | | |
| `spawn_blocking` | ✅ Pass | | |
| `block_on` | ✅ Pass | | |
| `block_on_inner` | ✅ Pass | | |
| `spawn_named` | ✅ Pass | | |
| `spawn_local_named` | ✅ Pass | | |
| `runtime_flavor` | ✅ Pass | | |
| `id` | ✅ Pass | | |
| `metrics` | ✅ Pass | | |
| `dump` | ✅ Pass | | |
| `is_tracing` | ✅ Pass | | |
| `spawn_thread` | ✅ Pass | | |
| `new_no_context` | ✅ Pass | | |
| `new_thread_local_destroyed` | ✅ Pass | | |
| `is_missing_context` | ✅ Pass | | |
| `is_thread_local_destroyed` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |

### runtime/id.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ❌ Fail | ERR-008 | |
| `fmt` | ✅ Pass | | |

### runtime/io/driver.rs (15 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `with_ready` | ❌ Fail | ERR-005 | |
| `_assert_kinds` | ✅ Pass | | |
| `_assert` | ❌ Fail | ERR-005 | |
| `new` | ❌ Fail | ERR-005 | |
| `park` | ✅ Pass | | |
| `park_timeout` | ❌ Fail | ERR-005 | |
| `shutdown` | ❌ Fail | ERR-005 | |
| `turn` | ❌ Fail | ERR-005 | |
| `fmt` | ❌ Fail | ERR-005 | |
| `unpark` | ✅ Pass | | |
| `add_source` | ❌ Fail | ERR-005 | |
| `deregister_source` | ❌ Fail | ERR-005 | |
| `release_pending_registrations` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-005 | |
| `mask` | ❌ Fail | ERR-005 | |

### runtime/io/driver/signal.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `register_signal_receiver` | ✅ Pass | | |
| `consume_signal_ready` | ✅ Pass | | |

### runtime/io/driver/uring.rs (17 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `as_usize` | ✅ Pass | | |
| `from_usize` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `ring` | ✅ Pass | | |
| `ring_mut` | ✅ Pass | | |
| `try_init` | ✅ Pass | | |
| `dispatch_completions` | ✅ Pass | | |
| `submit` | ✅ Pass | | |
| `remove_op` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `add_uring_source` | ✅ Pass | | |
| `get_uring` | ✅ Pass | | |
| `set_uring_state` | ✅ Pass | | |
| `check_and_init` | ✅ Pass | | |
| `try_init` | ✅ Pass | | |
| `register_op` | ✅ Pass | | |
| `cancel_op` | ✅ Pass | | |

### runtime/io/metrics.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `incr_fd_count` | ✅ Pass | | |
| `dec_fd_count` | ✅ Pass | | |
| `incr_ready_count_by` | ✅ Pass | | |

### runtime/io/registration.rs (15 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new_with_interest_and_handle` | ✅ Pass | | |
| `deregister` | ✅ Pass | | |
| `clear_readiness` | ✅ Pass | | |
| `poll_read_ready` | ✅ Pass | | |
| `poll_write_ready` | ✅ Pass | | |
| `poll_read_io` | ✅ Pass | | |
| `poll_write_io` | ✅ Pass | | |
| `poll_ready` | ✅ Pass | | |
| `poll_io` | ✅ Pass | | |
| `try_io` | ✅ Pass | | |
| `readiness` | ✅ Pass | | |
| `async_io` | ✅ Pass | | |
| `handle` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `gone` | ✅ Pass | | |

### runtime/io/registration_set.rs (11 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `is_shutdown` | ✅ Pass | | |
| `needs_release` | ✅ Pass | | |
| `allocate` | ✅ Pass | | |
| `deregister` | ✅ Pass | | |
| `shutdown` | ✅ Pass | | |
| `release` | ✅ Pass | | |
| `remove` | ✅ Pass | | |
| `as_raw` | ✅ Pass | | |
| `from_raw` | ✅ Pass | | |
| `pointers` | ✅ Pass | | |

### runtime/io/scheduled_io.rs (18 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `addr_of_pointers` | ✅ Pass | | |
| `default` | ✅ Pass | | |
| `token` | ✅ Pass | | |
| `shutdown` | ✅ Pass | | |
| `set_readiness` | ❌ Fail | ERR-004 | |
| `wake` | ❌ Fail | ERR-003 | |
| `ready_event` | ✅ Pass | | |
| `poll_readiness` | ✅ Pass | | |
| `clear_readiness` | ✅ Pass | | |
| `clear_wakers` | ❌ Fail | ERR-003 | |
| `drop` | ✅ Pass | | |
| `readiness` | ✅ Pass | | |
| `readiness_fut` | ✅ Pass | | |
| `as_raw` | ✅ Pass | | |
| `from_raw` | ✅ Pass | | |
| `pointers` | ✅ Pass | | |
| `poll` | ✅ Pass | | |
| `drop` | ✅ Pass | | |

### runtime/local_runtime/runtime.rs (12 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `from_parts` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `handle` | ✅ Pass | | |
| `spawn_local` | ✅ Pass | | |
| `spawn_blocking` | ✅ Pass | | |
| `block_on` | ✅ Pass | | |
| `block_on_inner` | ✅ Pass | | |
| `enter` | ✅ Pass | | |
| `shutdown_timeout` | ✅ Pass | | |
| `shutdown_background` | ✅ Pass | | |
| `metrics` | ✅ Pass | | |
| `drop` | ✅ Pass | | |

### runtime/metrics/batch.rs (25 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `new_unstable` | ✅ Pass | | |
| `new_unstable` | ✅ Pass | | |
| `submit` | ❌ Fail | ERR-003 | |
| `submit_unstable` | ✅ Pass | | |
| `submit_unstable` | ✅ Pass | | |
| `about_to_park` | ✅ Pass | | |
| `about_to_park` | ✅ Pass | | |
| `unparked` | ✅ Pass | | |
| `start_processing_scheduled_tasks` | ✅ Pass | | |
| `end_processing_scheduled_tasks` | ✅ Pass | | |
| `start_poll` | ✅ Pass | | |
| `start_poll` | ✅ Pass | | |
| `end_poll` | ✅ Pass | | |
| `end_poll` | ✅ Pass | | |
| `inc_local_schedule_count` | ✅ Pass | | |
| `inc_local_schedule_count` | ✅ Pass | | |
| `incr_steal_count` | ✅ Pass | | |
| `incr_steal_count` | ✅ Pass | | |
| `incr_steal_operations` | ✅ Pass | | |
| `incr_steal_operations` | ✅ Pass | | |
| `incr_overflow_count` | ✅ Pass | | |
| `incr_overflow_count` | ✅ Pass | | |
| `duration_as_u64` | ✅ Pass | | |
| `now` | ✅ Pass | | |

### runtime/metrics/histogram.rs (24 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `default` | ✅ Pass | | |
| `linear` | ✅ Pass | | |
| `log` | ✅ Pass | | |
| `num_buckets` | ✅ Pass | | |
| `value_to_bucket` | ✅ Pass | | |
| `bucket_range` | ✅ Pass | | |
| `num_buckets` | ✅ Pass | | |
| `get` | ✅ Pass | | |
| `bucket_range` | ✅ Pass | | |
| `from_histogram` | ✅ Pass | | |
| `measure` | ✅ Pass | | |
| `submit` | ✅ Pass | | |
| `value_to_bucket` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `legacy_mut` | ✅ Pass | | |
| `build` | ✅ Pass | | |
| `default` | ✅ Pass | | |
| `linear` | ✅ Pass | | |
| `test_legacy_builder` | ✅ Pass | | |
| `log_scale_resolution_1` | ✅ Pass | | |
| `log_scale_resolution_2` | ✅ Pass | | |
| `linear_scale_resolution_1` | ✅ Pass | | |
| `linear_scale_resolution_100` | ✅ Pass | | |
| `inc_by_more_than_one` | ✅ Pass | | |

### runtime/metrics/histogram/h2_histogram.rs (26 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `default` | ✅ Pass | | |
| `from_n_p` | ✅ Pass | | |
| `truncate_to_max_value` | ✅ Pass | | |
| `builder` | ✅ Pass | | |
| `max_value` | ✅ Pass | | |
| `value_to_bucket` | ✅ Pass | | |
| `bucket_range` | ✅ Pass | | |
| `from` | ✅ Pass | | |
| `max_error` | ✅ Pass | | |
| `precision_exact` | ✅ Pass | | |
| `min_value` | ✅ Pass | | |
| `max_value` | ✅ Pass | | |
| `max_buckets` | ✅ Pass | | |
| `build` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `bucket_index` | ✅ Pass | | |
| `valid_log_histogram_strategy` | ✅ Pass | | |
| `log_histogram_settings` | ✅ Pass | | |
| `log_histogram_settings_maintain_invariants` | ✅ Pass | | |
| `proptest_log_histogram_invariants` | ✅ Pass | | |
| `bucket_ranges_are_correct` | ✅ Pass | | |
| `bucket_computation_spot_check` | ✅ Pass | | |
| `last_bucket_goes_to_infinity` | ✅ Pass | | |
| `bucket_offset` | ✅ Pass | | |
| `max_buckets_enforcement` | ✅ Pass | | |
| `default_configuration_size` | ✅ Pass | | |

### runtime/metrics/io.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `incr_fd_count` | ✅ Pass | | |
| `dec_fd_count` | ✅ Pass | | |
| `incr_ready_count_by` | ✅ Pass | | |

### runtime/metrics/mock.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `inc_remote_schedule_count` | ✅ Pass | | |

### runtime/metrics/runtime.rs (36 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `num_workers` | ✅ Pass | | |
| `num_alive_tasks` | ✅ Pass | | |
| `global_queue_depth` | ✅ Pass | | |
| `worker_total_busy_duration` | ✅ Pass | | |
| `worker_park_count` | ✅ Pass | | |
| `worker_park_unpark_count` | ✅ Pass | | |
| `num_blocking_threads` | ✅ Pass | | |
| `active_tasks_count` | ✅ Pass | | |
| `num_idle_blocking_threads` | ✅ Pass | | |
| `worker_thread_id` | ✅ Pass | | |
| `injection_queue_depth` | ✅ Pass | | |
| `worker_local_queue_depth` | ✅ Pass | | |
| `poll_time_histogram_enabled` | ✅ Pass | | |
| `poll_count_histogram_enabled` | ✅ Pass | | |
| `poll_time_histogram_num_buckets` | ✅ Pass | | |
| `poll_count_histogram_num_buckets` | ✅ Pass | | |
| `poll_time_histogram_bucket_range` | ✅ Pass | | |
| `poll_count_histogram_bucket_range` | ✅ Pass | | |
| `blocking_queue_depth` | ✅ Pass | | |
| `spawned_tasks_count` | ✅ Pass | | |
| `remote_schedule_count` | ✅ Pass | | |
| `budget_forced_yield_count` | ✅ Pass | | |
| `worker_noop_count` | ✅ Pass | | |
| `worker_steal_count` | ✅ Pass | | |
| `worker_steal_operations` | ✅ Pass | | |
| `worker_poll_count` | ✅ Pass | | |
| `worker_local_schedule_count` | ✅ Pass | | |
| `worker_overflow_count` | ✅ Pass | | |
| `poll_time_histogram_bucket_count` | ✅ Pass | | |
| `poll_count_histogram_bucket_count` | ✅ Pass | | |
| `worker_mean_poll_time` | ✅ Pass | | |
| `io_driver_fd_registered_count` | ✅ Pass | | |
| `io_driver_fd_deregistered_count` | ✅ Pass | | |
| `io_driver_ready_count` | ✅ Pass | | |
| `with_io_driver_metrics` | ✅ Pass | | |

### runtime/metrics/scheduler.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `inc_remote_schedule_count` | ✅ Pass | | |
| `inc_budget_forced_yield_count` | ✅ Pass | | |

### runtime/metrics/worker.rs (7 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `set_queue_depth` | ✅ Pass | | |
| `set_thread_id` | ✅ Pass | | |
| `from_config` | ✅ Pass | | |
| `from_config` | ✅ Pass | | |
| `queue_depth` | ✅ Pass | | |
| `thread_id` | ✅ Pass | | |

### runtime/mod.rs (9 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ❌ Fail | ERR-005,ERR-012 | |
| `deadline` | ❌ Fail | ERR-012 | |
| `is_elapsed` | ✅ Pass | | |
| `flavor` | ❌ Fail | ERR-012 | |
| `reset` | ❌ Fail | ERR-012 | |
| `poll_elapsed` | ❌ Fail | ERR-012 | |
| `scheduler_handle` | ✅ Pass | | |
| `driver` | ✅ Pass | | |
| `clock` | ✅ Pass | | |

### runtime/park.rs (27 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `unpark` | ✅ Pass | | |
| `park` | ✅ Pass | | |
| `park_timeout` | ✅ Pass | | |
| `shutdown` | ✅ Pass | | |
| `park` | ✅ Pass | | |
| `park_timeout` | ✅ Pass | | |
| `unpark` | ✅ Pass | | |
| `shutdown` | ✅ Pass | | |
| `default` | ✅ Pass | | |
| `unpark` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `waker` | ✅ Pass | | |
| `unpark` | ✅ Pass | | |
| `park` | ✅ Pass | | |
| `park_timeout` | ✅ Pass | | |
| `with_current` | ✅ Pass | | |
| `block_on` | ✅ Pass | | |
| `into_waker` | ✅ Pass | | |
| `into_raw` | ✅ Pass | | |
| `from_raw` | ✅ Pass | | |
| `unparker_to_raw_waker` | ✅ Pass | | |
| `clone` | ✅ Pass | | |
| `drop_waker` | ✅ Pass | | |
| `wake` | ✅ Pass | | |
| `wake_by_ref` | ✅ Pass | | |
| `current_thread_park_count` | ✅ Pass | | |

### runtime/process.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `park` | ✅ Pass | | |
| `park_timeout` | ✅ Pass | | |
| `shutdown` | ✅ Pass | | |

### runtime/runtime.rs (12 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `from_parts` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `handle` | ✅ Pass | | |
| `spawn` | ✅ Pass | | |
| `spawn_blocking` | ✅ Pass | | |
| `block_on` | ✅ Pass | | |
| `block_on_inner` | ✅ Pass | | |
| `enter` | ✅ Pass | | |
| `shutdown_timeout` | ✅ Pass | | |
| `shutdown_background` | ✅ Pass | | |
| `metrics` | ❌ Fail | ERR-009 | |
| `drop` | ✅ Pass | | |

### runtime/scheduler/block_in_place.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `block_in_place` | ✅ Pass | | |

### runtime/scheduler/current_thread/mod.rs (44 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ❌ Fail | ERR-007 | |
| `block_on` | ❌ Fail | ERR-009 | |
| `take_core` | ✅ Pass | | |
| `shutdown` | ✅ Pass | | |
| `shutdown2` | ❌ Fail | ERR-003 | |
| `fmt` | ❌ Fail | ERR-003 | |
| `tick` | ✅ Pass | | |
| `next_task` | ✅ Pass | | |
| `next_local_task` | ✅ Pass | | |
| `push_task` | ✅ Pass | | |
| `submit_metrics` | ✅ Pass | | |
| `wake_deferred_tasks_and_free` | ✅ Pass | | |
| `run_task` | ✅ Pass | | |
| `park` | ✅ Pass | | |
| `park_yield` | ✅ Pass | | |
| `park_internal` | ❌ Fail | ERR-003 | |
| `enter` | ✅ Pass | | |
| `defer` | ✅ Pass | | |
| `spawn` | ✅ Pass | | |
| `spawn_local` | ✅ Pass | | |
| `dump` | ✅ Pass | | |
| `next_remote_task` | ✅ Pass | | |
| `waker_ref` | ✅ Pass | | |
| `reset_woken` | ✅ Pass | | |
| `num_alive_tasks` | ✅ Pass | | |
| `injection_queue_depth` | ✅ Pass | | |
| `worker_metrics` | ✅ Pass | | |
| `scheduler_metrics` | ✅ Pass | | |
| `worker_local_queue_depth` | ✅ Pass | | |
| `num_blocking_threads` | ✅ Pass | | |
| `num_idle_blocking_threads` | ✅ Pass | | |
| `blocking_queue_depth` | ✅ Pass | | |
| `spawned_tasks_count` | ✅ Pass | | |
| `owned_id` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `release` | ✅ Pass | | |
| `schedule` | ❌ Fail | ERR-003 | |
| `hooks` | ✅ Pass | | |
| `unhandled_panic` | ✅ Pass | | |
| `wake` | ✅ Pass | | |
| `wake_by_ref` | ✅ Pass | | |
| `block_on` | ❌ Fail | ERR-009 | |
| `enter` | ✅ Pass | | |
| `drop` | ✅ Pass | | |

### runtime/scheduler/defer.rs (5 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `defer` | ✅ Pass | | |
| `is_empty` | ✅ Pass | | |
| `wake` | ✅ Pass | | |
| `take_deferred` | ✅ Pass | | |

### runtime/scheduler/inject.rs (5 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `is_closed` | ✅ Pass | | |
| `close` | ✅ Pass | | |
| `push` | ✅ Pass | | |
| `pop` | ✅ Pass | | |

### runtime/scheduler/inject/metrics.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `len` | ✅ Pass | | |

### runtime/scheduler/inject/pop.rs (5 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `next` | ✅ Pass | | |
| `size_hint` | ✅ Pass | | |
| `len` | ✅ Pass | | |
| `drop` | ✅ Pass | | |

### runtime/scheduler/inject/rt_multi_thread.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `lock` | ✅ Pass | | |
| `as_mut` | ✅ Pass | | |
| `push_batch` | ❌ Fail | ERR-003,ERR-009 | |
| `push_batch_inner` | ❌ Fail | ERR-003 | |

### runtime/scheduler/inject/shared.rs (8 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `is_empty` | ✅ Pass | | |
| `is_closed` | ✅ Pass | | |
| `close` | ✅ Pass | | |
| `len` | ✅ Pass | | |
| `push` | ✅ Pass | | |
| `pop` | ❌ Fail | ERR-003 | |
| `pop_n` | ✅ Pass | | |

### runtime/scheduler/inject/synced.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `pop` | ❌ Fail | ERR-003 | |

### runtime/scheduler/mod.rs (31 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `driver` | ❌ Fail | ERR-012 | |
| `current` | ✅ Pass | | |
| `blocking_spawner` | ✅ Pass | | |
| `is_local` | ✅ Pass | | |
| `timer_flavor` | ✅ Pass | | |
| `is_same_runtime` | ✅ Pass | | |
| `is_shutdown` | ✅ Pass | | |
| `push_remote_timer` | ✅ Pass | | |
| `can_spawn_local_on_local_runtime` | ✅ Pass | | |
| `spawn` | ✅ Pass | | |
| `spawn_local` | ✅ Pass | | |
| `shutdown` | ✅ Pass | | |
| `seed_generator` | ✅ Pass | | |
| `as_current_thread` | ✅ Pass | | |
| `hooks` | ✅ Pass | | |
| `num_workers` | ✅ Pass | | |
| `num_alive_tasks` | ✅ Pass | | |
| `injection_queue_depth` | ✅ Pass | | |
| `worker_metrics` | ✅ Pass | | |
| `spawned_tasks_count` | ✅ Pass | | |
| `num_blocking_threads` | ✅ Pass | | |
| `num_idle_blocking_threads` | ✅ Pass | | |
| `scheduler_metrics` | ✅ Pass | | |
| `worker_local_queue_depth` | ✅ Pass | | |
| `blocking_queue_depth` | ✅ Pass | | |
| `expect_current_thread` | ✅ Pass | | |
| `defer` | ✅ Pass | | |
| `with_time_temp_local_context` | ✅ Pass | | |
| `expect_multi_thread` | ✅ Pass | | |
| `current` | ✅ Pass | | |
| `timer_flavor` | ✅ Pass | | |

### runtime/scheduler/multi_thread/counters.rs (11 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `drop` | ✅ Pass | | |
| `inc_num_inc_notify_local` | ✅ Pass | | |
| `inc_num_unparks_local` | ✅ Pass | | |
| `inc_num_maintenance` | ✅ Pass | | |
| `inc_lifo_schedules` | ✅ Pass | | |
| `inc_lifo_capped` | ❌ Fail | ERR-005 | |
| `inc_num_inc_notify_local` | ✅ Pass | | |
| `inc_num_unparks_local` | ✅ Pass | | |
| `inc_num_maintenance` | ✅ Pass | | |
| `inc_lifo_schedules` | ✅ Pass | | |
| `inc_lifo_capped` | ❌ Fail | ERR-005 | |

### runtime/scheduler/multi_thread/handle.rs (10 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `spawn` | ✅ Pass | | |
| `is_shutdown` | ✅ Pass | | |
| `shutdown` | ✅ Pass | | |
| `bind_new_task` | ✅ Pass | | |
| `release` | ✅ Pass | | |
| `schedule` | ✅ Pass | | |
| `hooks` | ✅ Pass | | |
| `yield_now` | ✅ Pass | | |
| `owned_id` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |

### runtime/scheduler/multi_thread/handle/metrics.rs (10 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `num_workers` | ✅ Pass | | |
| `num_alive_tasks` | ✅ Pass | | |
| `injection_queue_depth` | ✅ Pass | | |
| `worker_metrics` | ✅ Pass | | |
| `spawned_tasks_count` | ✅ Pass | | |
| `num_blocking_threads` | ✅ Pass | | |
| `num_idle_blocking_threads` | ✅ Pass | | |
| `scheduler_metrics` | ✅ Pass | | |
| `worker_local_queue_depth` | ✅ Pass | | |
| `blocking_queue_depth` | ✅ Pass | | |

### runtime/scheduler/multi_thread/handle/taskdump.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `dump` | ✅ Pass | | |

### runtime/scheduler/multi_thread/idle.rs (20 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `worker_to_notify` | ✅ Pass | | |
| `transition_worker_to_parked` | ✅ Pass | | |
| `transition_worker_to_searching` | ✅ Pass | | |
| `transition_worker_from_searching` | ✅ Pass | | |
| `unpark_worker_by_id` | ✅ Pass | | |
| `is_parked` | ✅ Pass | | |
| `notify_should_wakeup` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `load` | ✅ Pass | | |
| `unpark_one` | ✅ Pass | | |
| `inc_num_searching` | ✅ Pass | | |
| `dec_num_searching` | ✅ Pass | | |
| `dec_num_unparked` | ✅ Pass | | |
| `num_searching` | ✅ Pass | | |
| `num_unparked` | ✅ Pass | | |
| `from` | ✅ Pass | | |
| `from` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `test_state` | ❌ Fail | ERR-003 | |

### runtime/scheduler/multi_thread/mod.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `block_on` | ✅ Pass | | |
| `shutdown` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |

### runtime/scheduler/multi_thread/overflow.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `push` | ✅ Pass | | |
| `push_batch` | ✅ Pass | | |

### runtime/scheduler/multi_thread/park.rs (13 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `unpark` | ✅ Pass | | |
| `park` | ✅ Pass | | |
| `park_timeout` | ✅ Pass | | |
| `shutdown` | ✅ Pass | | |
| `clone` | ✅ Pass | | |
| `unpark` | ✅ Pass | | |
| `park` | ✅ Pass | | |
| `park_condvar` | ✅ Pass | | |
| `park_driver` | ✅ Pass | | |
| `unpark` | ✅ Pass | | |
| `unpark_condvar` | ✅ Pass | | |
| `shutdown` | ✅ Pass | | |

### runtime/scheduler/multi_thread/queue.rs (22 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `make_fixed_size` | ✅ Pass | | |
| `local` | ✅ Pass | | |
| `len` | ✅ Pass | | |
| `remaining_slots` | ❌ Fail | ERR-002 | |
| `max_capacity` | ❌ Fail | ERR-002 | |
| `has_tasks` | ✅ Pass | | |
| `push_back` | ❌ Fail | ERR-002 | |
| `push_back_or_overflow` | ❌ Fail | ERR-003 | |
| `push_back_finish` | ✅ Pass | | |
| `push_overflow` | ❌ Fail | ERR-003,ERR-004,ERR-009 | |
| `next` | ✅ Pass | | |
| `pop` | ✅ Pass | | |
| `len` | ✅ Pass | | |
| `is_empty` | ✅ Pass | | |
| `steal_into` | ❌ Fail | ERR-002 | |
| `steal_into2` | ✅ Pass | | |
| `clone` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `len` | ✅ Pass | | |
| `unpack` | ✅ Pass | | |
| `pack` | ✅ Pass | | |
| `test_local_queue_capacity` | ✅ Pass | | |

### runtime/scheduler/multi_thread/stats.rs (13 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `tuned_global_queue_interval` | ❌ Fail | ERR-007 | |
| `submit` | ❌ Fail | ERR-007 | |
| `about_to_park` | ✅ Pass | | |
| `unparked` | ✅ Pass | | |
| `inc_local_schedule_count` | ✅ Pass | | |
| `start_processing_scheduled_tasks` | ✅ Pass | | |
| `end_processing_scheduled_tasks` | ✅ Pass | | |
| `start_poll` | ✅ Pass | | |
| `end_poll` | ✅ Pass | | |
| `incr_steal_count` | ✅ Pass | | |
| `incr_steal_operations` | ✅ Pass | | |
| `incr_overflow_count` | ✅ Pass | | |

### runtime/scheduler/multi_thread/trace.rs (6 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `trace_requested` | ✅ Pass | | |
| `start_trace_request` | ✅ Pass | | |
| `stash_result` | ✅ Pass | | |
| `take_result` | ✅ Pass | | |
| `end_trace_request` | ✅ Pass | | |

### runtime/scheduler/multi_thread/trace_mock.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `trace_requested` | ✅ Pass | | |

### runtime/scheduler/multi_thread/worker.rs (54 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `create` | ✅ Pass | | |
| `block_in_place` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `launch` | ✅ Pass | | |
| `run` | ❌ Fail | ERR-003 | |
| `drop` | ✅ Pass | | |
| `run` | ❌ Fail | ERR-003 | |
| `run_task` | ❌ Fail | ERR-003,ERR-009 | |
| `reset_lifo_enabled` | ✅ Pass | | |
| `assert_lifo_enabled_is_correct` | ✅ Pass | | |
| `maintenance` | ✅ Pass | | |
| `park` | ❌ Fail | ERR-003 | |
| `park_yield` | ✅ Pass | | |
| `park_internal` | ❌ Fail | ERR-012 | |
| `defer` | ❌ Fail | ERR-012 | |
| `maintain_local_timers_before_parking` | ✅ Pass | | |
| `maintain_local_timers_after_parking` | ✅ Pass | | |
| `with_core` | ✅ Pass | | |
| `with_time_temp_local_context` | ✅ Pass | | |
| `tick` | ✅ Pass | | |
| `next_task` | ✅ Pass | | |
| `next_local_task` | ✅ Pass | | |
| `steal_work` | ❌ Fail | ERR-003 | |
| `transition_to_searching` | ✅ Pass | | |
| `transition_from_searching` | ✅ Pass | | |
| `has_tasks` | ✅ Pass | | |
| `should_notify_others` | ✅ Pass | | |
| `transition_to_parked` | ✅ Pass | | |
| `transition_from_parked` | ✅ Pass | | |
| `maintenance` | ✅ Pass | | |
| `pre_shutdown` | ✅ Pass | | |
| `shutdown` | ✅ Pass | | |
| `tune_global_queue_interval` | ✅ Pass | | |
| `inject` | ✅ Pass | | |
| `schedule_task` | ❌ Fail | ERR-003 | |
| `schedule_option_task_without_yield` | ✅ Pass | | |
| `schedule_local` | ❌ Fail | ERR-003 | |
| `next_remote_task` | ✅ Pass | | |
| `push_remote_task` | ✅ Pass | | |
| `push_remote_timer` | ✅ Pass | | |
| `take_remote_timers` | ✅ Pass | | |
| `close` | ✅ Pass | | |
| `notify_parked_local` | ✅ Pass | | |
| `notify_parked_remote` | ✅ Pass | | |
| `notify_all` | ✅ Pass | | |
| `notify_if_work_pending` | ✅ Pass | | |
| `transition_worker_from_searching` | ✅ Pass | | |
| `shutdown_core` | ✅ Pass | | |
| `ptr_eq` | ✅ Pass | | |
| `push` | ✅ Pass | | |
| `push_batch` | ✅ Pass | | |
| `as_mut` | ✅ Pass | | |
| `lock` | ✅ Pass | | |
| `with_current` | ❌ Fail | ERR-003 | |

### runtime/scheduler/multi_thread/worker/metrics.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `injection_queue_depth` | ✅ Pass | | |
| `worker_local_queue_depth` | ✅ Pass | | |

### runtime/scheduler/multi_thread/worker/taskdump.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `trace_core` | ✅ Pass | | |
| `steal_all` | ✅ Pass | | |

### runtime/scheduler/multi_thread/worker/taskdump_mock.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `trace_core` | ✅ Pass | | |

### runtime/scheduler/util/time_alt.rs (11 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `min_duration` | ✅ Pass | | |
| `process_registration_queue` | ✅ Pass | | |
| `insert_inject_timers` | ✅ Pass | | |
| `remove_cancelled_timers` | ✅ Pass | | |
| `next_expiration_time` | ✅ Pass | | |
| `pre_auto_advance` | ✅ Pass | | |
| `process_expired_timers` | ✅ Pass | | |
| `shutdown_local_timers` | ✅ Pass | | |
| `post_auto_advance` | ✅ Pass | | |
| `pre_auto_advance` | ✅ Pass | | |
| `post_auto_advance` | ✅ Pass | | |

### runtime/signal/mod.rs (7 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `handle` | ✅ Pass | | |
| `park` | ❌ Fail | ERR-003 | |
| `park_timeout` | ❌ Fail | ERR-003 | |
| `shutdown` | ✅ Pass | | |
| `process` | ✅ Pass | | |
| `check_inner` | ✅ Pass | | |

### runtime/task/abort.rs (7 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `abort` | ✅ Pass | | |
| `is_finished` | ✅ Pass | | |
| `id` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `drop` | ✅ Pass | | |
| `clone` | ✅ Pass | | |

### runtime/task/core.rs (27 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `addr_of_owned` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `new_header` | ✅ Pass | | |
| `check` | ✅ Pass | | |
| `with_mut` | ✅ Pass | | |
| `enter` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-009 | |
| `drop_future_or_output` | ❌ Fail | ERR-003 | |
| `store_output` | ❌ Fail | ERR-003 | |
| `take_output` | ✅ Pass | | |
| `set_stage` | ✅ Pass | | |
| `set_next` | ✅ Pass | | |
| `set_owner_id` | ✅ Pass | | |
| `get_owner_id` | ✅ Pass | | |
| `get_trailer` | ✅ Pass | | |
| `get_scheduler` | ✅ Pass | | |
| `get_id_ptr` | ✅ Pass | | |
| `get_id` | ✅ Pass | | |
| `get_spawn_location_ptr` | ✅ Pass | | |
| `get_spawn_location` | ✅ Pass | | |
| `get_tracing_id` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `set_waker` | ✅ Pass | | |
| `will_wake` | ✅ Pass | | |
| `wake_join` | ✅ Pass | | |
| `header_lte_cache_line` | ✅ Pass | | |

### runtime/task/error.rs (11 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `cancelled` | ✅ Pass | | |
| `panic` | ✅ Pass | | |
| `is_cancelled` | ✅ Pass | | |
| `is_panic` | ✅ Pass | | |
| `into_panic` | ✅ Pass | | |
| `try_into_panic` | ✅ Pass | | |
| `id` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `from` | ✅ Pass | | |
| `panic_payload_as_str` | ✅ Pass | | |

### runtime/task/harness.rs (29 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `from_raw` | ✅ Pass | | |
| `header_ptr` | ✅ Pass | | |
| `header` | ✅ Pass | | |
| `state` | ✅ Pass | | |
| `trailer` | ✅ Pass | | |
| `core` | ✅ Pass | | |
| `drop_reference` | ✅ Pass | | |
| `wake_by_val` | ✅ Pass | | |
| `wake_by_ref` | ✅ Pass | | |
| `remote_abort` | ✅ Pass | | |
| `try_set_join_waker` | ✅ Pass | | |
| `drop_reference` | ✅ Pass | | |
| `poll` | ✅ Pass | | |
| `poll_inner` | ✅ Pass | | |
| `transition_result_to_poll_future` | ✅ Pass | | |
| `shutdown` | ✅ Pass | | |
| `dealloc` | ❌ Fail | ERR-009 | |
| `try_read_output` | ✅ Pass | | |
| `drop_join_handle_slow` | ❌ Fail | ERR-009 | |
| `complete` | ✅ Pass | | |
| `release` | ❌ Fail | E0422,ERR-011 | |
| `get_new_task` | ✅ Pass | | |
| `can_read_output` | ✅ Pass | | |
| `set_join_waker` | ❌ Fail | ERR-003 | |
| `cancel_task` | ✅ Pass | | |
| `panic_result_to_join_error` | ✅ Pass | | |
| `poll_future` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `panic_to_error` | ✅ Pass | | |

### runtime/task/id.rs (5 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `id` | ✅ Pass | | |
| `try_id` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `next` | ✅ Pass | | |
| `as_u64` | ✅ Pass | | |

### runtime/task/join.rs (9 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `abort` | ✅ Pass | | |
| `is_finished` | ✅ Pass | | |
| `set_join_waker` | ✅ Pass | | |
| `abort_handle` | ✅ Pass | | |
| `id` | ✅ Pass | | |
| `poll` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |

### runtime/task/list.rs (24 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `get_next_id` | ✅ Pass | | |
| `get_next_id` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `bind` | ❌ Fail | E0423,ERR-002,ERR-007 | |
| `bind_local` | ✅ Pass | | |
| `bind_inner` | ❌ Fail | E0423,ERR-002,ERR-003,ERR-007,ERR-009 | |
| `assert_owner` | ✅ Pass | | |
| `close_and_shutdown_all` | ❌ Fail | E0423,ERR-002,ERR-007,ERR-012 | |
| `get_shard_size` | ✅ Pass | | |
| `num_alive_tasks` | ✅ Pass | | |
| `spawned_tasks_count` | ✅ Pass | | |
| `remove` | ❌ Fail | E0423,ERR-002 | |
| `is_empty` | ✅ Pass | | |
| `gen_shared_list_size` | ✅ Pass | | |
| `for_each` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `bind` | ❌ Fail | E0423,ERR-002,ERR-007 | |
| `close_and_shutdown_all` | ❌ Fail | E0423,ERR-002,ERR-007,ERR-012 | |
| `remove` | ❌ Fail | E0423,ERR-002 | |
| `assert_owner` | ✅ Pass | | |
| `with_inner` | ✅ Pass | | |
| `is_closed` | ✅ Pass | | |
| `is_empty` | ✅ Pass | | |
| `test_id_not_broken` | ✅ Pass | | |

### runtime/task/mod.rs (37 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `task_meta` | ✅ Pass | | |
| `task_meta` | ✅ Pass | | |
| `yield_now` | ✅ Pass | | |
| `unhandled_panic` | ✅ Pass | | |
| `new_task` | ✅ Pass | | |
| `unowned` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `from_raw` | ✅ Pass | | |
| `as_raw` | ✅ Pass | | |
| `header` | ✅ Pass | | |
| `header_ptr` | ✅ Pass | | |
| `id` | ✅ Pass | | |
| `spawned_at` | ✅ Pass | | |
| `task_meta` | ✅ Pass | | |
| `notify_for_tracing` | ✅ Pass | | |
| `header` | ✅ Pass | | |
| `task_id` | ✅ Pass | | |
| `from_raw` | ✅ Pass | | |
| `into_raw` | ✅ Pass | | |
| `shutdown` | ✅ Pass | | |
| `run` | ✅ Pass | | |
| `into_notified` | ✅ Pass | | |
| `into_task` | ✅ Pass | | |
| `run` | ✅ Pass | | |
| `shutdown` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `as_raw` | ✅ Pass | | |
| `from_raw` | ✅ Pass | | |
| `pointers` | ✅ Pass | | |
| `get_shard_id` | ✅ Pass | | |
| `from` | ✅ Pass | | |
| `from` | ✅ Pass | | |
| `spawn_location_is_zero_sized` | ✅ Pass | | |
| `capture` | ❌ Fail | ERR-005 | |

### runtime/task/raw.rs (25 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `vtable` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `from_raw` | ✅ Pass | | |
| `header_ptr` | ✅ Pass | | |
| `trailer_ptr` | ✅ Pass | | |
| `header` | ✅ Pass | | |
| `trailer` | ✅ Pass | | |
| `state` | ✅ Pass | | |
| `poll` | ✅ Pass | | |
| `schedule` | ✅ Pass | | |
| `dealloc` | ✅ Pass | | |
| `try_read_output` | ❌ Fail | ERR-009 | |
| `drop_join_handle_slow` | ✅ Pass | | |
| `drop_abort_handle` | ❌ Fail | ERR-009 | |
| `shutdown` | ❌ Fail | ERR-009 | |
| `ref_inc` | ✅ Pass | | |
| `get_queue_next` | ✅ Pass | | |
| `set_queue_next` | ✅ Pass | | |
| `poll` | ✅ Pass | | |
| `schedule` | ✅ Pass | | |
| `dealloc` | ✅ Pass | | |
| `try_read_output` | ❌ Fail | ERR-009 | |
| `drop_join_handle_slow` | ✅ Pass | | |
| `drop_abort_handle` | ❌ Fail | ERR-009 | |
| `shutdown` | ❌ Fail | ERR-009 | |

### runtime/task/state.rs (41 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `load` | ✅ Pass | | |
| `transition_to_running` | ✅ Pass | | |
| `transition_to_idle` | ❌ Fail | ERR-003 | |
| `transition_to_complete` | ✅ Pass | | |
| `transition_to_terminal` | ✅ Pass | | |
| `transition_to_notified_by_val` | ✅ Pass | | |
| `transition_to_notified_by_ref` | ✅ Pass | | |
| `transition_to_notified_for_tracing` | ✅ Pass | | |
| `transition_to_notified_and_cancel` | ✅ Pass | | |
| `transition_to_shutdown` | ✅ Pass | | |
| `drop_join_handle_fast` | ✅ Pass | | |
| `transition_to_join_handle_dropped` | ❌ Fail | ERR-003 | |
| `set_join_waker` | ✅ Pass | | |
| `unset_waker` | ❌ Fail | ERR-003 | |
| `unset_waker_after_complete` | ✅ Pass | | |
| `ref_inc` | ✅ Pass | | |
| `ref_dec` | ✅ Pass | | |
| `ref_dec_twice` | ✅ Pass | | |
| `fetch_update_action` | ✅ Pass | | |
| `fetch_update` | ✅ Pass | | |
| `is_idle` | ✅ Pass | | |
| `is_notified` | ✅ Pass | | |
| `unset_notified` | ✅ Pass | | |
| `set_notified` | ✅ Pass | | |
| `is_running` | ✅ Pass | | |
| `set_running` | ✅ Pass | | |
| `unset_running` | ✅ Pass | | |
| `is_cancelled` | ✅ Pass | | |
| `set_cancelled` | ✅ Pass | | |
| `is_complete` | ✅ Pass | | |
| `is_join_interested` | ✅ Pass | | |
| `unset_join_interested` | ✅ Pass | | |
| `is_join_waker_set` | ✅ Pass | | |
| `set_join_waker` | ✅ Pass | | |
| `unset_join_waker` | ✅ Pass | | |
| `ref_count` | ✅ Pass | | |
| `ref_inc` | ✅ Pass | | |
| `ref_dec` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `fmt` | ❌ Fail | ERR-003 | |

### runtime/task/trace/mod.rs (15 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `try_with_current` | ✅ Pass | | |
| `with_current_frame` | ✅ Pass | | |
| `with_current_collector` | ✅ Pass | | |
| `is_tracing` | ✅ Pass | | |
| `capture` | ✅ Pass | | |
| `root` | ✅ Pass | | |
| `backtraces` | ✅ Pass | | |
| `trace_leaf` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `defer` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `poll` | ✅ Pass | | |
| `trace_current_thread` | ✅ Pass | | |
| `trace_multi_thread` | ✅ Pass | | |
| `trace_owned` | ✅ Pass | | |

### runtime/task/trace/symbol.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `hash` | ✅ Pass | | |
| `eq` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |

### runtime/task/trace/tree.rs (5 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `from_trace` | ✅ Pass | | |
| `consequences` | ✅ Pass | | |
| `display` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `to_symboltrace` | ✅ Pass | | |

### runtime/task/waker.rs (7 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `waker_ref` | ✅ Pass | | |
| `deref` | ✅ Pass | | |
| `clone_waker` | ✅ Pass | | |
| `drop_waker` | ✅ Pass | | |
| `wake_by_val` | ✅ Pass | | |
| `wake_by_ref` | ✅ Pass | | |
| `raw_waker` | ✅ Pass | | |

### runtime/task_hooks.rs (6 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `spawn` | ✅ Pass | | |
| `from_config` | ✅ Pass | | |
| `poll_start_callback` | ✅ Pass | | |
| `poll_stop_callback` | ✅ Pass | | |
| `id` | ✅ Pass | | |
| `spawned_at` | ✅ Pass | | |

### runtime/thread_id.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `next` | ✅ Pass | | |
| `exhausted` | ✅ Pass | | |

### runtime/time/entry.rs (43 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `default` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `is_pending` | ✅ Pass | | |
| `when` | ✅ Pass | | |
| `poll` | ✅ Pass | | |
| `read_state` | ✅ Pass | | |
| `mark_pending` | ✅ Pass | | |
| `fire` | ✅ Pass | | |
| `set_expiration` | ✅ Pass | | |
| `extend_expiration` | ✅ Pass | | |
| `might_be_registered` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `addr_of_pointers` | ❌ Fail | ERR-003 | |
| `new` | ✅ Pass | | |
| `registered_when` | ✅ Pass | | |
| `sync_when` | ✅ Pass | | |
| `set_registered_when` | ✅ Pass | | |
| `true_when` | ✅ Pass | | |
| `set_expiration` | ✅ Pass | | |
| `extend_expiration` | ✅ Pass | | |
| `handle` | ✅ Pass | | |
| `might_be_registered` | ✅ Pass | | |
| `as_raw` | ✅ Pass | | |
| `from_raw` | ✅ Pass | | |
| `pointers` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `inner` | ✅ Pass | | |
| `init_inner` | ✅ Pass | | |
| `deadline` | ✅ Pass | | |
| `is_elapsed` | ✅ Pass | | |
| `cancel` | ❌ Fail | ERR-002,ERR-008 | |
| `reset` | ✅ Pass | | |
| `poll_elapsed` | ❌ Fail | ERR-002,ERR-003 | |
| `driver` | ✅ Pass | | |
| `clock` | ✅ Pass | | |
| `registered_when` | ✅ Pass | | |
| `sync_when` | ✅ Pass | | |
| `is_pending` | ✅ Pass | | |
| `set_expiration` | ✅ Pass | | |
| `mark_pending` | ✅ Pass | | |
| `fire` | ✅ Pass | | |

### runtime/time/handle.rs (5 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `time_source` | ✅ Pass | | |
| `is_shutdown` | ✅ Pass | | |
| `unpark` | ❌ Fail | E0026,ERR-012 | |
| `current` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |

### runtime/time/mod.rs (18 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `new_alt` | ✅ Pass | | |
| `park` | ❌ Fail | ERR-003 | |
| `park_timeout` | ✅ Pass | | |
| `shutdown` | ❌ Fail | ERR-003,ERR-012 | |
| `park_internal` | ✅ Pass | | |
| `park_thread_timeout` | ✅ Pass | | |
| `park_thread_timeout` | ✅ Pass | | |
| `process` | ✅ Pass | | |
| `process_at_time` | ❌ Fail | ERR-003 | |
| `process_at_time_alt` | ✅ Pass | | |
| `shutdown_alt` | ✅ Pass | | |
| `clear_entry` | ✅ Pass | | |
| `reregister` | ✅ Pass | | |
| `did_wake` | ✅ Pass | | |
| `lock` | ✅ Pass | | |
| `is_shutdown` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003,ERR-012 | |

### runtime/time/source.rs (6 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `deadline_to_tick` | ✅ Pass | | |
| `instant_to_tick` | ❌ Fail | ERR-007 | |
| `tick_to_duration` | ❌ Fail | ERR-007 | |
| `now` | ✅ Pass | | |
| `start_time` | ✅ Pass | | |

### runtime/time/wheel/level.rs (12 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `next_expiration` | ✅ Pass | | |
| `next_occupied_slot` | ✅ Pass | | |
| `add_entry` | ✅ Pass | | |
| `remove_entry` | ✅ Pass | | |
| `take_slot` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `occupied_bit` | ❌ Fail | ERR-003 | |
| `slot_range` | ✅ Pass | | |
| `level_range` | ✅ Pass | | |
| `slot_for` | ✅ Pass | | |
| `test_slot_for` | ✅ Pass | | |

### runtime/time/wheel/mod.rs (15 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `elapsed` | ✅ Pass | | |
| `insert` | ✅ Pass | | |
| `remove` | ✅ Pass | | |
| `poll_at` | ✅ Pass | | |
| `poll` | ✅ Pass | | |
| `next_expiration` | ✅ Pass | | |
| `next_expiration_time` | ✅ Pass | | |
| `no_expirations_before` | ✅ Pass | | |
| `process_expiration` | ❌ Fail | ERR-003 | |
| `set_elapsed` | ✅ Pass | | |
| `take_entries` | ✅ Pass | | |
| `level_for` | ✅ Pass | | |
| `level_for` | ✅ Pass | | |
| `test_level_for` | ✅ Pass | | |

### runtime/time_alt/cancellation_queue.rs (9 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `drop` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `push_front` | ✅ Pass | | |
| `iter` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `next` | ✅ Pass | | |
| `send` | ✅ Pass | | |
| `recv_all` | ✅ Pass | | |
| `new` | ✅ Pass | | |

### runtime/time_alt/cancellation_queue/tests.rs (5 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new_handle` | ✅ Pass | | |
| `model` | ✅ Pass | | |
| `single_thread` | ✅ Pass | | |
| `multi_thread` | ✅ Pass | | |
| `drop_iter_should_not_leak_memory` | ✅ Pass | | |

### runtime/time_alt/context.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `new_running` | ✅ Pass | | |
| `new_shutdown` | ✅ Pass | | |

### runtime/time_alt/entry.rs (22 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `as_raw` | ✅ Pass | | |
| `from_raw` | ✅ Pass | | |
| `pointers` | ✅ Pass | | |
| `as_raw` | ✅ Pass | | |
| `from_raw` | ✅ Pass | | |
| `pointers` | ✅ Pass | | |
| `as_raw` | ✅ Pass | | |
| `from_raw` | ✅ Pass | | |
| `pointers` | ✅ Pass | | |
| `as_raw` | ✅ Pass | | |
| `from_raw` | ✅ Pass | | |
| `pointers` | ✅ Pass | | |
| `from` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `wake` | ✅ Pass | | |
| `register_cancel_tx` | ✅ Pass | | |
| `register_waker` | ✅ Pass | | |
| `cancel` | ✅ Pass | | |
| `deadline` | ✅ Pass | | |
| `is_woken_up` | ✅ Pass | | |
| `is_cancelled` | ✅ Pass | | |
| `inner_strong_count` | ✅ Pass | | |

### runtime/time_alt/registration_queue.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `drop` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `push_front` | ✅ Pass | | |
| `pop_front` | ✅ Pass | | |

### runtime/time_alt/registration_queue/tests.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new_handle` | ✅ Pass | | |
| `model` | ✅ Pass | | |
| `sanity` | ✅ Pass | | |
| `drop_should_not_leak_memory` | ✅ Pass | | |

### runtime/time_alt/tests.rs (6 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new_handle` | ✅ Pass | | |
| `model` | ✅ Pass | | |
| `wake_up_in_the_same_thread` | ✅ Pass | | |
| `cancel_in_the_same_thread` | ✅ Pass | | |
| `wake_up_in_the_different_thread` | ✅ Pass | | |
| `cancel_in_the_different_thread` | ✅ Pass | | |

### runtime/time_alt/timer.rs (13 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `fmt` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `deadline` | ✅ Pass | | |
| `is_elapsed` | ✅ Pass | | |
| `register` | ✅ Pass | | |
| `poll_elapsed` | ✅ Pass | | |
| `scheduler_handle` | ✅ Pass | | |
| `driver` | ✅ Pass | | |
| `clock` | ✅ Pass | | |
| `with_current_temp_local_context` | ✅ Pass | | |
| `push_from_remote` | ✅ Pass | | |
| `deadline_to_tick` | ✅ Pass | | |

### runtime/time_alt/wake_queue.rs (5 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `drop` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `is_empty` | ✅ Pass | | |
| `push_front` | ✅ Pass | | |
| `wake_all` | ✅ Pass | | |

### runtime/time_alt/wake_queue/tests.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new_handle` | ✅ Pass | | |
| `model` | ✅ Pass | | |
| `sanity` | ✅ Pass | | |
| `drop_should_not_leak_memory` | ✅ Pass | | |

### runtime/time_alt/wheel/level.rs (12 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `next_expiration` | ✅ Pass | | |
| `next_occupied_slot` | ✅ Pass | | |
| `add_entry` | ✅ Pass | | |
| `remove_entry` | ✅ Pass | | |
| `take_slot` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `occupied_bit` | ✅ Pass | | |
| `slot_range` | ✅ Pass | | |
| `level_range` | ✅ Pass | | |
| `slot_for` | ✅ Pass | | |
| `test_slot_for` | ✅ Pass | | |

### runtime/time_alt/wheel/mod.rs (14 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `elapsed` | ✅ Pass | | |
| `insert` | ✅ Pass | | |
| `remove` | ✅ Pass | | |
| `take_expired` | ✅ Pass | | |
| `next_expiration` | ✅ Pass | | |
| `next_expiration_time` | ✅ Pass | | |
| `no_expirations_before` | ✅ Pass | | |
| `process_expiration` | ✅ Pass | | |
| `set_elapsed` | ✅ Pass | | |
| `take_entries` | ✅ Pass | | |
| `level_for` | ✅ Pass | | |
| `level_for` | ✅ Pass | | |
| `test_level_for` | ✅ Pass | | |

### signal/ctrl_c.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `ctrl_c` | ✅ Pass | | |

### signal/mod.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `make_future` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `recv` | ✅ Pass | | |
| `poll_recv` | ✅ Pass | | |

### signal/registry.rs (20 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `default` | ✅ Pass | | |
| `event_info` | ❌ Fail | ERR-010 | |
| `for_each` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `register_listener` | ✅ Pass | | |
| `record_event` | ✅ Pass | | |
| `broadcast` | ✅ Pass | | |
| `deref` | ✅ Pass | | |
| `register_listener` | ✅ Pass | | |
| `record_event` | ✅ Pass | | |
| `broadcast` | ✅ Pass | | |
| `storage` | ✅ Pass | | |
| `globals_init` | ✅ Pass | | |
| `globals` | ✅ Pass | | |
| `smoke` | ✅ Pass | | |
| `register_panics_on_invalid_input` | ✅ Pass | | |
| `record_invalid_event_does_nothing` | ✅ Pass | | |
| `broadcast_returns_if_at_least_one_event_fired` | ✅ Pass | | |
| `rt` | ✅ Pass | | |
| `collect` | ✅ Pass | | |

### signal/reusable_box.rs (13 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ❌ Fail | ERR-008 | |
| `set` | ❌ Fail | ERR-003 | |
| `try_set` | ✅ Pass | | |
| `set_same_layout` | ✅ Pass | | |
| `get_pin` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-003 | |
| `poll` | ❌ Fail | ERR-003 | |
| `drop` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `test_different_futures` | ✅ Pass | | |
| `test_different_sizes` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-003 | |
| `test_zero_sized` | ✅ Pass | | |

### signal/unix.rs (19 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `init` | ❌ Fail | ERR-009 | |
| `event_info` | ❌ Fail | ERR-010 | |
| `for_each` | ❌ Fail | ERR-011 | |
| `init` | ❌ Fail | ERR-009 | |
| `from` | ✅ Pass | | |
| `from` | ✅ Pass | | |
| `default` | ✅ Pass | | |
| `action` | ✅ Pass | | |
| `signal_enable` | ❌ Fail | ERR-010 | |
| `signal` | ✅ Pass | | |
| `signal_with_handle` | ✅ Pass | | |
| `recv` | ✅ Pass | | |
| `poll_recv` | ✅ Pass | | |
| `poll_recv` | ✅ Pass | | |
| `ctrl_c` | ✅ Pass | | |
| `signal_enable_error_on_invalid_input` | ❌ Fail | ERR-003 | |
| `signal_enable_error_on_forbidden_input` | ✅ Pass | | |
| `from_c_int` | ✅ Pass | | |
| `into_c_int` | ✅ Pass | | |

### signal/windows.rs (15 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `ctrl_c` | ✅ Pass | | |
| `recv` | ✅ Pass | | |
| `poll_recv` | ✅ Pass | | |
| `recv` | ✅ Pass | | |
| `poll_recv` | ✅ Pass | | |
| `ctrl_break` | ✅ Pass | | |
| `ctrl_close` | ✅ Pass | | |
| `recv` | ✅ Pass | | |
| `poll_recv` | ✅ Pass | | |
| `ctrl_shutdown` | ✅ Pass | | |
| `recv` | ✅ Pass | | |
| `poll_recv` | ✅ Pass | | |
| `ctrl_logoff` | ✅ Pass | | |
| `recv` | ✅ Pass | | |
| `poll_recv` | ✅ Pass | | |

### signal/windows/stub.rs (5 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `ctrl_break` | ✅ Pass | | |
| `ctrl_close` | ✅ Pass | | |
| `ctrl_c` | ✅ Pass | | |
| `ctrl_logoff` | ✅ Pass | | |
| `ctrl_shutdown` | ✅ Pass | | |

### signal/windows/sys.rs (20 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `ctrl_break` | ✅ Pass | | |
| `ctrl_close` | ✅ Pass | | |
| `ctrl_c` | ✅ Pass | | |
| `ctrl_logoff` | ✅ Pass | | |
| `ctrl_shutdown` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `event_requires_infinite_sleep_in_handler` | ✅ Pass | | |
| `init` | ✅ Pass | | |
| `event_info` | ✅ Pass | | |
| `for_each` | ✅ Pass | | |
| `init` | ✅ Pass | | |
| `global_init` | ✅ Pass | | |
| `handler` | ✅ Pass | | |
| `raise_event` | ✅ Pass | | |
| `ctrl_c` | ✅ Pass | | |
| `ctrl_break` | ✅ Pass | | |
| `ctrl_close` | ✅ Pass | | |
| `ctrl_shutdown` | ✅ Pass | | |
| `ctrl_logoff` | ✅ Pass | | |
| `rt` | ✅ Pass | | |

### sync/barrier.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ❌ Fail | ERR-002 | |
| `wait` | ❌ Fail | ERR-005,ERR-011 | |
| `wait_internal` | ✅ Pass | | |
| `is_leader` | ✅ Pass | | |

### sync/batch_semaphore.rs (28 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `addr_of_pointers` | ✅ Pass | | |
| `new` | ❌ Fail | ERR-003,ERR-005,ERR-011 | |
| `new_closed` | ✅ Pass | | |
| `available_permits` | ✅ Pass | | |
| `release` | ❌ Fail | ERR-003 | |
| `close` | ✅ Pass | | |
| `is_closed` | ✅ Pass | | |
| `try_acquire` | ✅ Pass | | |
| `acquire` | ✅ Pass | | |
| `add_permits_locked` | ✅ Pass | | |
| `forget_permits` | ✅ Pass | | |
| `poll_acquire` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `new` | ❌ Fail | ERR-003,ERR-005,ERR-011 | |
| `assign_permits` | ✅ Pass | | |
| `poll` | ✅ Pass | | |
| `new` | ❌ Fail | ERR-003,ERR-005,ERR-011 | |
| `project` | ❌ Fail | ERR-005 | |
| `is_unpin` | ❌ Fail | ERR-005,ERR-009,ERR-010,ERR-011 | |
| `drop` | ✅ Pass | | |
| `closed` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `is_closed` | ✅ Pass | | |
| `is_no_permits` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `as_raw` | ✅ Pass | | |
| `from_raw` | ✅ Pass | | |
| `pointers` | ✅ Pass | | |

### sync/broadcast.rs (58 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `fmt` | ❌ Fail | ERR-005 | |
| `fmt` | ❌ Fail | ERR-005 | |
| `fmt` | ❌ Fail | ERR-005 | |
| `new` | ❌ Fail | ERR-003 | |
| `addr_of_pointers` | ✅ Pass | | |
| `channel` | ✅ Pass | | |
| `new` | ❌ Fail | ERR-003 | |
| `new_with_receiver_count` | ✅ Pass | | |
| `send` | ✅ Pass | | |
| `subscribe` | ✅ Pass | | |
| `downgrade` | ✅ Pass | | |
| `len` | ✅ Pass | | |
| `is_empty` | ✅ Pass | | |
| `receiver_count` | ✅ Pass | | |
| `same_channel` | ✅ Pass | | |
| `closed` | ✅ Pass | | |
| `close_channel` | ✅ Pass | | |
| `strong_count` | ✅ Pass | | |
| `weak_count` | ✅ Pass | | |
| `new_receiver` | ✅ Pass | | |
| `drop` | ❌ Fail | ERR-009 | |
| `new` | ❌ Fail | ERR-003 | |
| `pop_back_locked` | ✅ Pass | | |
| `notify_rx` | ❌ Fail | ERR-003 | |
| `clone` | ✅ Pass | | |
| `drop` | ❌ Fail | ERR-009 | |
| `upgrade` | ✅ Pass | | |
| `strong_count` | ✅ Pass | | |
| `weak_count` | ✅ Pass | | |
| `clone` | ✅ Pass | | |
| `drop` | ❌ Fail | ERR-009 | |
| `len` | ✅ Pass | | |
| `is_empty` | ✅ Pass | | |
| `same_channel` | ✅ Pass | | |
| `recv_ref` | ✅ Pass | | |
| `sender_strong_count` | ✅ Pass | | |
| `sender_weak_count` | ✅ Pass | | |
| `is_closed` | ✅ Pass | | |
| `resubscribe` | ✅ Pass | | |
| `recv` | ✅ Pass | | |
| `try_recv` | ✅ Pass | | |
| `blocking_recv` | ✅ Pass | | |
| `drop` | ❌ Fail | ERR-009 | |
| `new` | ❌ Fail | ERR-003 | |
| `project` | ✅ Pass | | |
| `poll` | ✅ Pass | | |
| `drop` | ❌ Fail | ERR-009 | |
| `as_raw` | ✅ Pass | | |
| `from_raw` | ✅ Pass | | |
| `pointers` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-005 | |
| `fmt` | ❌ Fail | ERR-005 | |
| `fmt` | ❌ Fail | ERR-005 | |
| `clone_value` | ✅ Pass | | |
| `drop` | ❌ Fail | ERR-009 | |
| `is_unpin` | ✅ Pass | | |
| `receiver_count_on_sender_constructor` | ✅ Pass | | |
| `receiver_count_on_channel_constructor` | ✅ Pass | | |

### sync/mpsc/block.rs (25 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `start_index` | ✅ Pass | | |
| `offset` | ✅ Pass | | |
| `addr_of_header` | ✅ Pass | | |
| `addr_of_values` | ✅ Pass | | |
| `new` | ❌ Fail | ERR-010 | |
| `is_at_index` | ✅ Pass | | |
| `distance` | ✅ Pass | | |
| `read` | ❌ Fail | ERR-009 | |
| `has_value` | ✅ Pass | | |
| `write` | ❌ Fail | ERR-003 | |
| `tx_close` | ✅ Pass | | |
| `is_closed` | ✅ Pass | | |
| `reclaim` | ✅ Pass | | |
| `tx_release` | ✅ Pass | | |
| `set_ready` | ✅ Pass | | |
| `is_final` | ✅ Pass | | |
| `observed_tail_position` | ✅ Pass | | |
| `load_next` | ✅ Pass | | |
| `try_push` | ❌ Fail | ERR-003 | |
| `grow` | ✅ Pass | | |
| `is_ready` | ✅ Pass | | |
| `is_tx_closed` | ✅ Pass | | |
| `initialize` | ✅ Pass | | |
| `index` | ✅ Pass | | |
| `assert_no_stack_overflow` | ✅ Pass | | |

### sync/mpsc/bounded.rs (59 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `channel` | ✅ Pass | | |
| `new` | ❌ Fail | ERR-003 | |
| `recv` | ✅ Pass | | |
| `recv_many` | ✅ Pass | | |
| `try_recv` | ✅ Pass | | |
| `blocking_recv` | ❌ Fail | ERR-003 | |
| `blocking_recv_many` | ❌ Fail | ERR-003 | |
| `close` | ✅ Pass | | |
| `is_closed` | ❌ Fail | ERR-003 | |
| `is_empty` | ✅ Pass | | |
| `len` | ✅ Pass | | |
| `capacity` | ✅ Pass | | |
| `max_capacity` | ✅ Pass | | |
| `poll_recv` | ✅ Pass | | |
| `poll_recv_many` | ✅ Pass | | |
| `sender_strong_count` | ✅ Pass | | |
| `sender_weak_count` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `new` | ❌ Fail | ERR-003 | |
| `send` | ❌ Fail | ERR-003 | |
| `closed` | ✅ Pass | | |
| `try_send` | ✅ Pass | | |
| `send_timeout` | ❌ Fail | ERR-003,ERR-009 | |
| `blocking_send` | ✅ Pass | | |
| `is_closed` | ❌ Fail | ERR-003 | |
| `reserve` | ✅ Pass | | |
| `reserve_many` | ✅ Pass | | |
| `reserve_owned` | ✅ Pass | | |
| `reserve_inner` | ✅ Pass | | |
| `try_reserve` | ✅ Pass | | |
| `try_reserve_many` | ✅ Pass | | |
| `try_reserve_owned` | ✅ Pass | | |
| `same_channel` | ✅ Pass | | |
| `capacity` | ✅ Pass | | |
| `downgrade` | ✅ Pass | | |
| `max_capacity` | ✅ Pass | | |
| `strong_count` | ✅ Pass | | |
| `weak_count` | ❌ Fail | ERR-003 | |
| `clone` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `clone` | ✅ Pass | | |
| `drop` | ❌ Fail | ERR-003 | |
| `upgrade` | ✅ Pass | | |
| `strong_count` | ✅ Pass | | |
| `weak_count` | ❌ Fail | ERR-003 | |
| `fmt` | ❌ Fail | ERR-003 | |
| `send` | ❌ Fail | ERR-003 | |
| `drop` | ❌ Fail | ERR-003 | |
| `fmt` | ❌ Fail | ERR-003 | |
| `next` | ✅ Pass | | |
| `size_hint` | ✅ Pass | | |
| `drop` | ❌ Fail | ERR-003 | |
| `fmt` | ❌ Fail | ERR-003 | |
| `send` | ❌ Fail | ERR-003 | |
| `release` | ❌ Fail | ERR-003 | |
| `same_channel` | ✅ Pass | | |
| `same_channel_as_sender` | ✅ Pass | | |
| `drop` | ❌ Fail | ERR-003 | |
| `fmt` | ❌ Fail | ERR-003 | |

### sync/mpsc/chan.rs (48 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `fmt` | ❌ Fail | ERR-003 | |
| `fmt` | ❌ Fail | ERR-003 | |
| `fmt` | ❌ Fail | ERR-003 | |
| `fmt` | ❌ Fail | ERR-003 | |
| `channel` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `strong_count` | ✅ Pass | | |
| `weak_count` | ✅ Pass | | |
| `downgrade` | ✅ Pass | | |
| `upgrade` | ✅ Pass | | |
| `semaphore` | ✅ Pass | | |
| `send` | ✅ Pass | | |
| `wake_rx` | ✅ Pass | | |
| `same_channel` | ✅ Pass | | |
| `is_closed` | ✅ Pass | | |
| `closed` | ✅ Pass | | |
| `clone` | ✅ Pass | | |
| `drop` | ❌ Fail | ERR-003 | |
| `new` | ✅ Pass | | |
| `close` | ✅ Pass | | |
| `is_closed` | ✅ Pass | | |
| `is_empty` | ✅ Pass | | |
| `len` | ✅ Pass | | |
| `recv` | ✅ Pass | | |
| `recv_many` | ✅ Pass | | |
| `try_recv` | ✅ Pass | | |
| `semaphore` | ✅ Pass | | |
| `sender_strong_count` | ✅ Pass | | |
| `sender_weak_count` | ✅ Pass | | |
| `drop` | ❌ Fail | ERR-003 | |
| `drain` | ✅ Pass | | |
| `drop` | ❌ Fail | ERR-003 | |
| `send` | ✅ Pass | | |
| `decrement_weak_count` | ✅ Pass | | |
| `increment_weak_count` | ❌ Fail | ERR-003 | |
| `strong_count` | ✅ Pass | | |
| `weak_count` | ✅ Pass | | |
| `drop` | ❌ Fail | ERR-003 | |
| `add_permit` | ✅ Pass | | |
| `add_permits` | ✅ Pass | | |
| `is_idle` | ✅ Pass | | |
| `close` | ✅ Pass | | |
| `is_closed` | ✅ Pass | | |
| `add_permit` | ✅ Pass | | |
| `add_permits` | ✅ Pass | | |
| `is_idle` | ✅ Pass | | |
| `close` | ✅ Pass | | |
| `is_closed` | ✅ Pass | | |

### sync/mpsc/error.rs (11 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `fmt` | ❌ Fail | ERR-003 | |
| `fmt` | ❌ Fail | ERR-003 | |
| `into_inner` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `fmt` | ❌ Fail | ERR-003 | |
| `from` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `fmt` | ❌ Fail | ERR-003 | |
| `into_inner` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `fmt` | ❌ Fail | ERR-003 | |

### sync/mpsc/list.rs (15 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `channel` | ✅ Pass | | |
| `push` | ❌ Fail | ERR-010 | |
| `close` | ✅ Pass | | |
| `find_block` | ✅ Pass | | |
| `reclaim_block` | ❌ Fail | ERR-003 | |
| `is_closed` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `is_empty` | ✅ Pass | | |
| `len` | ✅ Pass | | |
| `pop` | ❌ Fail | ERR-010 | |
| `try_pop` | ❌ Fail | ERR-003 | |
| `try_advancing_head` | ✅ Pass | | |
| `reclaim_blocks` | ✅ Pass | | |
| `free_blocks` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |

### sync/mpsc/unbounded.rs (33 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `clone` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `fmt` | ❌ Fail | ERR-003 | |
| `unbounded_channel` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `recv` | ✅ Pass | | |
| `recv_many` | ✅ Pass | | |
| `try_recv` | ✅ Pass | | |
| `blocking_recv` | ❌ Fail | ERR-003 | |
| `blocking_recv_many` | ❌ Fail | ERR-003 | |
| `close` | ✅ Pass | | |
| `is_closed` | ✅ Pass | | |
| `is_empty` | ✅ Pass | | |
| `len` | ✅ Pass | | |
| `poll_recv` | ✅ Pass | | |
| `poll_recv_many` | ✅ Pass | | |
| `sender_strong_count` | ✅ Pass | | |
| `sender_weak_count` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `send` | ✅ Pass | | |
| `inc_num_messages` | ✅ Pass | | |
| `closed` | ✅ Pass | | |
| `is_closed` | ✅ Pass | | |
| `same_channel` | ✅ Pass | | |
| `downgrade` | ✅ Pass | | |
| `strong_count` | ✅ Pass | | |
| `weak_count` | ✅ Pass | | |
| `clone` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `upgrade` | ✅ Pass | | |
| `strong_count` | ✅ Pass | | |
| `weak_count` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |

### sync/mutex.rs (55 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `fmt` | ❌ Fail | ERR-009 | |
| `bounds` | ✅ Pass | | |
| `check_send` | ✅ Pass | | |
| `check_unpin` | ✅ Pass | | |
| `check_send_sync_val` | ✅ Pass | | |
| `check_send_sync` | ✅ Pass | | |
| `check_static` | ✅ Pass | | |
| `check_static_val` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `lock` | ✅ Pass | | |
| `blocking_lock` | ✅ Pass | | |
| `blocking_lock_owned` | ❌ Fail | ERR-009 | |
| `lock_owned` | ✅ Pass | | |
| `acquire` | ✅ Pass | | |
| `try_lock` | ✅ Pass | | |
| `get_mut` | ✅ Pass | | |
| `try_lock_owned` | ✅ Pass | | |
| `into_inner` | ✅ Pass | | |
| `from` | ✅ Pass | | |
| `default` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-009 | |
| `skip_drop` | ❌ Fail | ERR-003 | |
| `map` | ❌ Fail | ERR-009 | |
| `try_map` | ❌ Fail | ERR-009 | |
| `mutex` | ✅ Pass | | |
| `drop` | ❌ Fail | ERR-009 | |
| `deref` | ✅ Pass | | |
| `deref_mut` | ❌ Fail | ERR-009 | |
| `fmt` | ❌ Fail | ERR-009 | |
| `fmt` | ❌ Fail | ERR-009 | |
| `skip_drop` | ❌ Fail | ERR-003 | |
| `map` | ❌ Fail | ERR-009 | |
| `try_map` | ❌ Fail | ERR-009 | |
| `mutex` | ✅ Pass | | |
| `drop` | ❌ Fail | ERR-009 | |
| `deref` | ✅ Pass | | |
| `deref_mut` | ❌ Fail | ERR-009 | |
| `fmt` | ❌ Fail | ERR-009 | |
| `fmt` | ❌ Fail | ERR-009 | |
| `skip_drop` | ❌ Fail | ERR-003 | |
| `map` | ❌ Fail | ERR-009 | |
| `try_map` | ❌ Fail | ERR-009 | |
| `drop` | ❌ Fail | ERR-009 | |
| `deref` | ✅ Pass | | |
| `deref_mut` | ❌ Fail | ERR-009 | |
| `fmt` | ❌ Fail | ERR-009 | |
| `fmt` | ❌ Fail | ERR-009 | |
| `skip_drop` | ❌ Fail | ERR-003 | |
| `map` | ❌ Fail | ERR-009 | |
| `try_map` | ❌ Fail | ERR-009 | |
| `drop` | ❌ Fail | ERR-009 | |
| `deref` | ✅ Pass | | |
| `deref_mut` | ❌ Fail | ERR-009 | |
| `fmt` | ❌ Fail | ERR-009 | |
| `fmt` | ❌ Fail | ERR-009 | |

### sync/notify.rs (42 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `addr_of_pointers` | ✅ Pass | | |
| `none` | ✅ Pass | | |
| `store_release` | ✅ Pass | | |
| `load` | ✅ Pass | | |
| `clear` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `pop_back_locked` | ✅ Pass | | |
| `drop` | ❌ Fail | ERR-009 | |
| `set_state` | ✅ Pass | | |
| `get_state` | ✅ Pass | | |
| `get_num_notify_waiters_calls` | ✅ Pass | | |
| `inc_num_notify_waiters_calls` | ✅ Pass | | |
| `atomic_inc_num_notify_waiters_calls` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `notified` | ✅ Pass | | |
| `notified_owned` | ✅ Pass | | |
| `notify_one` | ✅ Pass | | |
| `notify_last` | ✅ Pass | | |
| `notify_with_strategy` | ✅ Pass | | |
| `notify_waiters` | ✅ Pass | | |
| `inner_notify_waiters` | ❌ Fail | ERR-003 | |
| `lock_waiter_list` | ✅ Pass | | |
| `default` | ✅ Pass | | |
| `notify_locked` | ✅ Pass | | |
| `enable` | ✅ Pass | | |
| `project` | ❌ Fail | ERR-009 | |
| `poll_notified` | ❌ Fail | ERR-009 | |
| `poll` | ❌ Fail | ERR-009 | |
| `drop` | ❌ Fail | ERR-009 | |
| `enable` | ✅ Pass | | |
| `project` | ❌ Fail | ERR-009 | |
| `poll_notified` | ❌ Fail | ERR-009 | |
| `poll` | ❌ Fail | ERR-009 | |
| `drop` | ❌ Fail | ERR-009 | |
| `poll_notified` | ❌ Fail | ERR-009 | |
| `drop_notified` | ✅ Pass | | |
| `as_raw` | ✅ Pass | | |
| `from_raw` | ✅ Pass | | |
| `pointers` | ✅ Pass | | |
| `is_unpin` | ✅ Pass | | |
| `notify_waiters` | ✅ Pass | | |

### sync/once_cell.rs (23 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `default` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `clone` | ✅ Pass | | |
| `eq` | ✅ Pass | | |
| `drop` | ❌ Fail | ERR-003 | |
| `from` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `new_with` | ✅ Pass | | |
| `initialized` | ✅ Pass | | |
| `initialized_mut` | ✅ Pass | | |
| `get_unchecked` | ✅ Pass | | |
| `get_unchecked_mut` | ✅ Pass | | |
| `set_value` | ❌ Fail | ERR-010 | |
| `get` | ✅ Pass | | |
| `get_mut` | ❌ Fail | ERR-003 | |
| `set` | ✅ Pass | | |
| `get_or_init` | ✅ Pass | | |
| `get_or_try_init` | ✅ Pass | | |
| `into_inner` | ✅ Pass | | |
| `take` | ❌ Fail | ERR-003 | |
| `fmt` | ❌ Fail | ERR-003 | |
| `is_already_init_err` | ✅ Pass | | |
| `is_initializing_err` | ✅ Pass | | |

### sync/oneshot.rs (41 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `fmt` | ❌ Fail | ERR-003,ERR-005 | |
| `fmt` | ❌ Fail | ERR-003,ERR-005 | |
| `will_wake` | ✅ Pass | | |
| `with_task` | ✅ Pass | | |
| `drop_task` | ✅ Pass | | |
| `set_task` | ❌ Fail | ERR-010 | |
| `channel` | ✅ Pass | | |
| `send` | ✅ Pass | | |
| `closed` | ❌ Fail | ERR-003 | |
| `is_closed` | ✅ Pass | | |
| `poll_closed` | ✅ Pass | | |
| `drop` | ❌ Fail | ERR-003 | |
| `close` | ✅ Pass | | |
| `is_terminated` | ✅ Pass | | |
| `is_empty` | ❌ Fail | ERR-002 | |
| `try_recv` | ✅ Pass | | |
| `blocking_recv` | ✅ Pass | | |
| `drop` | ❌ Fail | ERR-003 | |
| `poll` | ✅ Pass | | |
| `complete` | ✅ Pass | | |
| `poll_recv` | ✅ Pass | | |
| `close` | ✅ Pass | | |
| `consume_value` | ✅ Pass | | |
| `has_value` | ✅ Pass | | |
| `mut_load` | ✅ Pass | | |
| `drop` | ❌ Fail | ERR-003 | |
| `fmt` | ❌ Fail | ERR-003,ERR-005 | |
| `new` | ✅ Pass | | |
| `is_complete` | ✅ Pass | | |
| `set_complete` | ✅ Pass | | |
| `is_rx_task_set` | ✅ Pass | | |
| `set_rx_task` | ✅ Pass | | |
| `unset_rx_task` | ✅ Pass | | |
| `is_closed` | ✅ Pass | | |
| `set_closed` | ✅ Pass | | |
| `set_tx_task` | ✅ Pass | | |
| `unset_tx_task` | ✅ Pass | | |
| `is_tx_task_set` | ✅ Pass | | |
| `as_usize` | ✅ Pass | | |
| `load` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003,ERR-005 | |

### sync/rwlock.rs (22 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `bounds` | ✅ Pass | | |
| `check_send` | ✅ Pass | | |
| `check_sync` | ✅ Pass | | |
| `check_unpin` | ✅ Pass | | |
| `check_send_sync_val` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `with_max_readers` | ✅ Pass | | |
| `read` | ✅ Pass | | |
| `blocking_read` | ✅ Pass | | |
| `read_owned` | ✅ Pass | | |
| `try_read` | ✅ Pass | | |
| `try_read_owned` | ✅ Pass | | |
| `write` | ✅ Pass | | |
| `blocking_write` | ✅ Pass | | |
| `write_owned` | ✅ Pass | | |
| `try_write` | ✅ Pass | | |
| `try_write_owned` | ✅ Pass | | |
| `get_mut` | ✅ Pass | | |
| `into_inner` | ✅ Pass | | |
| `from` | ✅ Pass | | |
| `default` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |

### sync/rwlock/owned_read_guard.rs (8 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `skip_drop` | ✅ Pass | | |
| `map` | ❌ Fail | ERR-009 | |
| `try_map` | ❌ Fail | ERR-009 | |
| `rwlock` | ✅ Pass | | |
| `deref` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `drop` | ✅ Pass | | |

### sync/rwlock/owned_write_guard.rs (13 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `skip_drop` | ✅ Pass | | |
| `map` | ❌ Fail | ERR-009 | |
| `downgrade_map` | ❌ Fail | ERR-009 | |
| `try_map` | ❌ Fail | ERR-009 | |
| `try_downgrade_map` | ❌ Fail | ERR-009 | |
| `into_mapped` | ✅ Pass | | |
| `downgrade` | ❌ Fail | ERR-009 | |
| `rwlock` | ✅ Pass | | |
| `deref` | ✅ Pass | | |
| `deref_mut` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `drop` | ✅ Pass | | |

### sync/rwlock/owned_write_guard_mapped.rs (9 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `skip_drop` | ✅ Pass | | |
| `map` | ❌ Fail | ERR-009 | |
| `try_map` | ❌ Fail | ERR-009 | |
| `rwlock` | ✅ Pass | | |
| `deref` | ✅ Pass | | |
| `deref_mut` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `drop` | ✅ Pass | | |

### sync/rwlock/read_guard.rs (7 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `skip_drop` | ✅ Pass | | |
| `map` | ❌ Fail | ERR-009 | |
| `try_map` | ❌ Fail | ERR-009 | |
| `deref` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `drop` | ✅ Pass | | |

### sync/rwlock/write_guard.rs (12 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `skip_drop` | ✅ Pass | | |
| `map` | ❌ Fail | ERR-009 | |
| `downgrade_map` | ❌ Fail | ERR-009 | |
| `try_map` | ❌ Fail | ERR-009 | |
| `try_downgrade_map` | ❌ Fail | ERR-009 | |
| `into_mapped` | ✅ Pass | | |
| `downgrade` | ❌ Fail | ERR-009 | |
| `deref` | ✅ Pass | | |
| `deref_mut` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `drop` | ✅ Pass | | |

### sync/rwlock/write_guard_mapped.rs (8 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `skip_drop` | ✅ Pass | | |
| `map` | ❌ Fail | ERR-009 | |
| `try_map` | ❌ Fail | ERR-009 | |
| `deref` | ✅ Pass | | |
| `deref_mut` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `drop` | ✅ Pass | | |

### sync/semaphore.rs (30 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `bounds` | ✅ Pass | | |
| `check_unpin` | ✅ Pass | | |
| `check_send_sync_val` | ✅ Pass | | |
| `check_send_sync` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `new_closed` | ✅ Pass | | |
| `available_permits` | ✅ Pass | | |
| `add_permits` | ✅ Pass | | |
| `forget_permits` | ✅ Pass | | |
| `acquire` | ✅ Pass | | |
| `acquire_many` | ❌ Fail | ERR-005,ERR-011 | |
| `try_acquire` | ✅ Pass | | |
| `try_acquire_many` | ✅ Pass | | |
| `acquire_owned` | ✅ Pass | | |
| `acquire_many_owned` | ✅ Pass | | |
| `try_acquire_owned` | ✅ Pass | | |
| `try_acquire_many_owned` | ✅ Pass | | |
| `close` | ✅ Pass | | |
| `is_closed` | ✅ Pass | | |
| `forget` | ✅ Pass | | |
| `merge` | ✅ Pass | | |
| `split` | ✅ Pass | | |
| `num_permits` | ✅ Pass | | |
| `forget` | ✅ Pass | | |
| `merge` | ✅ Pass | | |
| `split` | ✅ Pass | | |
| `semaphore` | ✅ Pass | | |
| `num_permits` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `drop` | ✅ Pass | | |

### sync/set_once.rs (15 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `default` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `clone` | ✅ Pass | | |
| `eq` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `from` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `new_with` | ✅ Pass | | |
| `initialized` | ✅ Pass | | |
| `get_unchecked` | ✅ Pass | | |
| `get` | ✅ Pass | | |
| `set` | ❌ Fail | ERR-010 | |
| `into_inner` | ✅ Pass | | |
| `wait` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |

### sync/task/atomic_waker.rs (13 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `register` | ✅ Pass | | |
| `register_by_ref` | ✅ Pass | | |
| `do_register` | ✅ Pass | | |
| `catch_unwind` | ✅ Pass | | |
| `wake` | ✅ Pass | | |
| `take_waker` | ✅ Pass | | |
| `default` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `wake` | ✅ Pass | | |
| `into_waker` | ✅ Pass | | |
| `wake` | ✅ Pass | | |
| `into_waker` | ✅ Pass | | |

### sync/watch.rs (50 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `clone` | ✅ Pass | | |
| `default` | ✅ Pass | | |
| `has_changed` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003,ERR-005 | |
| `fmt` | ❌ Fail | ERR-003,ERR-005 | |
| `fmt` | ❌ Fail | ERR-003,ERR-005 | |
| `fmt` | ❌ Fail | ERR-003,ERR-005 | |
| `new` | ❌ Fail | ERR-005 | |
| `notify_waiters` | ✅ Pass | | |
| `notified` | ❌ Fail | ERR-005 | |
| `notified` | ❌ Fail | ERR-005 | |
| `decrement` | ✅ Pass | | |
| `version` | ❌ Fail | ERR-005 | |
| `is_closed` | ✅ Pass | | |
| `new` | ❌ Fail | ERR-005 | |
| `load` | ❌ Fail | ERR-005 | |
| `increment_version_while_locked` | ✅ Pass | | |
| `set_closed` | ❌ Fail | ERR-005 | |
| `channel` | ✅ Pass | | |
| `from_shared` | ✅ Pass | | |
| `borrow` | ✅ Pass | | |
| `borrow_and_update` | ✅ Pass | | |
| `has_changed` | ✅ Pass | | |
| `mark_changed` | ✅ Pass | | |
| `mark_unchanged` | ✅ Pass | | |
| `changed` | ✅ Pass | | |
| `wait_for` | ✅ Pass | | |
| `wait_for_inner` | ✅ Pass | | |
| `same_channel` | ✅ Pass | | |
| `try_has_changed` | ✅ Pass | | |
| `maybe_changed` | ✅ Pass | | |
| `changed_impl` | ✅ Pass | | |
| `clone` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `new` | ❌ Fail | ERR-005 | |
| `send` | ✅ Pass | | |
| `send_modify` | ✅ Pass | | |
| `send_if_modified` | ✅ Pass | | |
| `send_replace` | ✅ Pass | | |
| `borrow` | ✅ Pass | | |
| `is_closed` | ✅ Pass | | |
| `closed` | ✅ Pass | | |
| `subscribe` | ✅ Pass | | |
| `receiver_count` | ✅ Pass | | |
| `sender_count` | ✅ Pass | | |
| `same_channel` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `deref` | ✅ Pass | | |
| `watch_spurious_wakeup` | ✅ Pass | | |
| `watch_borrow` | ✅ Pass | | |

### task/blocking.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `block_in_place` | ✅ Pass | | |
| `spawn_blocking` | ✅ Pass | | |

### task/builder.rs (8 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `name` | ✅ Pass | | |
| `spawn` | ✅ Pass | | |
| `spawn_on` | ✅ Pass | | |
| `spawn_local` | ✅ Pass | | |
| `spawn_local_on` | ✅ Pass | | |
| `spawn_blocking` | ✅ Pass | | |
| `spawn_blocking_on` | ✅ Pass | | |

### task/coop/consume_budget.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `consume_budget` | ✅ Pass | | |

### task/coop/mod.rs (24 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `has_remaining` | ✅ Pass | | |
| `budget` | ✅ Pass | | |
| `with_unconstrained` | ✅ Pass | | |
| `with_budget` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `has_budget_remaining` | ✅ Pass | | |
| `set` | ✅ Pass | | |
| `stop` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `made_progress` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `poll_proceed` | ✅ Pass | | |
| `poll_budget_available` | ✅ Pass | | |
| `inc_budget_forced_yield_count` | ✅ Pass | | |
| `inc_budget_forced_yield_count` | ✅ Pass | | |
| `register_waker` | ✅ Pass | | |
| `inc_budget_forced_yield_count` | ✅ Pass | | |
| `register_waker` | ✅ Pass | | |
| `decrement` | ✅ Pass | | |
| `is_unconstrained` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-009 | |
| `cooperative` | ✅ Pass | | |
| `get` | ✅ Pass | | |
| `budgeting` | ✅ Pass | | |

### task/coop/unconstrained.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `poll` | ❌ Fail | ERR-009 | |
| `poll` | ❌ Fail | ERR-009 | |
| `unconstrained` | ✅ Pass | | |

### task/join_set.rs (34 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `len` | ✅ Pass | | |
| `is_empty` | ✅ Pass | | |
| `build_task` | ✅ Pass | | |
| `spawn` | ❌ Fail | ERR-003 | |
| `spawn_on` | ❌ Fail | ERR-003 | |
| `spawn_local` | ❌ Fail | ERR-003 | |
| `spawn_local_on` | ❌ Fail | ERR-003 | |
| `spawn_blocking` | ❌ Fail | ERR-003 | |
| `spawn_blocking_on` | ✅ Pass | | |
| `insert` | ❌ Fail | ERR-003 | |
| `join_next` | ✅ Pass | | |
| `join_next_with_id` | ✅ Pass | | |
| `try_join_next` | ✅ Pass | | |
| `try_join_next_with_id` | ✅ Pass | | |
| `shutdown` | ❌ Fail | ERR-003 | |
| `join_all` | ✅ Pass | | |
| `abort_all` | ✅ Pass | | |
| `detach_all` | ✅ Pass | | |
| `poll_join_next` | ❌ Fail | ERR-003 | |
| `poll_join_next_with_id` | ❌ Fail | ERR-003 | |
| `drop` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `default` | ❌ Fail | ERR-003 | |
| `from_iter` | ❌ Fail | ERR-003 | |
| `extend` | ✅ Pass | | |
| `name` | ❌ Fail | ERR-003 | |
| `spawn` | ❌ Fail | ERR-003 | |
| `spawn_on` | ❌ Fail | ERR-003 | |
| `spawn_blocking` | ❌ Fail | ERR-003 | |
| `spawn_blocking_on` | ✅ Pass | | |
| `spawn_local` | ❌ Fail | ERR-003 | |
| `spawn_local_on` | ❌ Fail | ERR-003 | |
| `fmt` | ✅ Pass | | |

### task/local.rs (42 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `enter` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `spawn_local` | ✅ Pass | | |
| `spawn_local_inner` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `new` | ✅ Pass | | |
| `enter` | ✅ Pass | | |
| `spawn_local` | ✅ Pass | | |
| `block_on` | ✅ Pass | | |
| `run_until` | ✅ Pass | | |
| `spawn_named` | ✅ Pass | | |
| `spawn_named_inner` | ✅ Pass | | |
| `tick` | ✅ Pass | | |
| `next_task` | ❌ Fail | ERR-009 | |
| `pop_local` | ✅ Pass | | |
| `with` | ✅ Pass | | |
| `with_if_possible` | ✅ Pass | | |
| `id` | ✅ Pass | | |
| `unhandled_panic` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `poll` | ❌ Fail | ERR-003,ERR-009 | |
| `default` | ❌ Fail | E0500 | |
| `drop` | ✅ Pass | | |
| `spawn` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-003,ERR-009 | |
| `schedule` | ✅ Pass | | |
| `ptr_eq` | ✅ Pass | | |
| `release` | ✅ Pass | | |
| `schedule` | ✅ Pass | | |
| `hooks` | ✅ Pass | | |
| `unhandled_panic` | ✅ Pass | | |
| `task_pop_front` | ✅ Pass | | |
| `task_push_back` | ✅ Pass | | |
| `take_local_queue` | ✅ Pass | | |
| `task_remove` | ✅ Pass | | |
| `owned_is_empty` | ✅ Pass | | |
| `assert_owner` | ✅ Pass | | |
| `close_and_shutdown_all` | ✅ Pass | | |
| `assert_called_from_owner_thread` | ✅ Pass | | |
| `local_current_thread_scheduler` | ✅ Pass | | |
| `wakes_to_local_queue` | ✅ Pass | | |

### task/spawn.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `spawn` | ✅ Pass | | |
| `spawn_inner` | ✅ Pass | | |

### task/task_local.rs (19 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `scope` | ✅ Pass | | |
| `sync_scope` | ✅ Pass | | |
| `scope_inner` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `with` | ✅ Pass | | |
| `try_with` | ✅ Pass | | |
| `get` | ✅ Pass | | |
| `try_get` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003 | |
| `drop` | ✅ Pass | | |
| `take_value` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-009 | |
| `fmt` | ❌ Fail | ERR-003 | |
| `fmt` | ❌ Fail | ERR-003 | |
| `fmt` | ❌ Fail | ERR-003 | |
| `fmt` | ❌ Fail | ERR-003 | |
| `panic` | ✅ Pass | | |
| `from` | ✅ Pass | | |
| `from` | ✅ Pass | | |

### task/yield_now.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `yield_now` | ✅ Pass | | |
| `poll` | ✅ Pass | | |

### time/clock.rs (16 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `now` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `now` | ✅ Pass | | |
| `with_clock` | ✅ Pass | | |
| `with_clock` | ✅ Pass | | |
| `pause` | ✅ Pass | | |
| `resume` | ✅ Pass | | |
| `advance` | ✅ Pass | | |
| `now` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `pause` | ✅ Pass | | |
| `inhibit_auto_advance` | ✅ Pass | | |
| `allow_auto_advance` | ✅ Pass | | |
| `can_auto_advance` | ✅ Pass | | |
| `advance` | ✅ Pass | | |
| `now` | ✅ Pass | | |

### time/error.rs (11 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `from` | ✅ Pass | | |
| `shutdown` | ✅ Pass | | |
| `is_shutdown` | ✅ Pass | | |
| `at_capacity` | ✅ Pass | | |
| `is_at_capacity` | ✅ Pass | | |
| `invalid` | ✅ Pass | | |
| `is_invalid` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `from` | ✅ Pass | | |

### time/instant.rs (20 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `now` | ❌ Fail | ERR-005 | |
| `from_std` | ✅ Pass | | |
| `far_future` | ✅ Pass | | |
| `into_std` | ✅ Pass | | |
| `duration_since` | ✅ Pass | | |
| `checked_duration_since` | ✅ Pass | | |
| `saturating_duration_since` | ✅ Pass | | |
| `elapsed` | ✅ Pass | | |
| `checked_add` | ✅ Pass | | |
| `checked_sub` | ✅ Pass | | |
| `from` | ✅ Pass | | |
| `from` | ✅ Pass | | |
| `add` | ✅ Pass | | |
| `add_assign` | ✅ Pass | | |
| `sub` | ✅ Pass | | |
| `sub` | ✅ Pass | | |
| `sub_assign` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |
| `now` | ❌ Fail | ERR-005 | |
| `now` | ❌ Fail | ERR-005 | |

### time/interval.rs (14 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `interval` | ✅ Pass | | |
| `interval_at` | ✅ Pass | | |
| `internal_interval_at` | ✅ Pass | | |
| `next_timeout` | ✅ Pass | | |
| `default` | ✅ Pass | | |
| `tick` | ❌ Fail | ERR-003 | |
| `poll_tick` | ✅ Pass | | |
| `reset` | ✅ Pass | | |
| `reset_immediately` | ✅ Pass | | |
| `reset_after` | ✅ Pass | | |
| `reset_at` | ✅ Pass | | |
| `missed_tick_behavior` | ✅ Pass | | |
| `set_missed_tick_behavior` | ✅ Pass | | |
| `period` | ✅ Pass | | |

### time/sleep.rs (11 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `sleep_until` | ✅ Pass | | |
| `sleep` | ✅ Pass | | |
| `new_timeout` | ✅ Pass | | |
| `far_future` | ✅ Pass | | |
| `deadline` | ✅ Pass | | |
| `is_elapsed` | ✅ Pass | | |
| `reset` | ❌ Fail | ERR-009 | |
| `reset_without_reregister` | ✅ Pass | | |
| `reset_inner` | ❌ Fail | ERR-012 | |
| `poll_elapsed` | ❌ Fail | ERR-005 | |
| `poll` | ✅ Pass | | |

### time/timeout.rs (8 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `timeout` | ✅ Pass | | |
| `timeout_at` | ✅ Pass | | |
| `new_with_delay` | ✅ Pass | | |
| `get_ref` | ✅ Pass | | |
| `get_mut` | ✅ Pass | | |
| `into_inner` | ✅ Pass | | |
| `poll` | ❌ Fail | ERR-009 | |
| `poll_delay` | ✅ Pass | | |

### util/as_ref.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `as_ref` | ❌ Fail | ERR-007 | |
| `upgrade` | ✅ Pass | | |

### util/atomic_cell.rs (7 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `swap` | ✅ Pass | | |
| `set` | ✅ Pass | | |
| `take` | ✅ Pass | | |
| `to_raw` | ✅ Pass | | |
| `from_raw` | ❌ Fail | ERR-009 | |
| `drop` | ✅ Pass | | |

### util/bit.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `pack` | ✅ Pass | | |
| `unpack` | ✅ Pass | | |
| `fmt` | ✅ Pass | | |

### util/blocking_check.rs (2 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `check_socket_for_blocking` | ✅ Pass | | |
| `check_socket_for_blocking` | ✅ Pass | | |

### util/cacheline.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `deref` | ✅ Pass | | |
| `deref_mut` | ✅ Pass | | |

### util/idle_notified_set.rs (22 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `addr_of_pointers` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `len` | ✅ Pass | | |
| `is_empty` | ✅ Pass | | |
| `insert_idle` | ✅ Pass | | |
| `pop_notified` | ✅ Pass | | |
| `try_pop_notified` | ✅ Pass | | |
| `for_each` | ✅ Pass | | |
| `get_ptrs` | ✅ Pass | | |
| `drain` | ✅ Pass | | |
| `pop_next` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `move_to_new_list` | ✅ Pass | | |
| `remove` | ✅ Pass | | |
| `with_value_and_context` | ✅ Pass | | |
| `drop` | ✅ Pass | | |
| `wake_by_ref` | ❌ Fail | ERR-003 | |
| `wake` | ✅ Pass | | |
| `as_raw` | ✅ Pass | | |
| `from_raw` | ✅ Pass | | |
| `pointers` | ✅ Pass | | |
| `join_set_test` | ✅ Pass | | |

### util/linked_list.rs (32 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `push_front` | ✅ Pass | | |
| `pop_front` | ✅ Pass | | |
| `pop_back` | ✅ Pass | | |
| `is_empty` | ✅ Pass | | |
| `remove` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003,ERR-010 | |
| `last` | ✅ Pass | | |
| `default` | ✅ Pass | | |
| `drain_filter` | ✅ Pass | | |
| `next` | ✅ Pass | | |
| `for_each` | ✅ Pass | | |
| `into_guarded` | ✅ Pass | | |
| `tail` | ✅ Pass | | |
| `pop_back` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `get_prev` | ✅ Pass | | |
| `get_next` | ✅ Pass | | |
| `set_prev` | ✅ Pass | | |
| `set_next` | ✅ Pass | | |
| `fmt` | ❌ Fail | ERR-003,ERR-010 | |
| `as_raw` | ✅ Pass | | |
| `from_raw` | ✅ Pass | | |
| `pointers` | ✅ Pass | | |
| `entry` | ✅ Pass | | |
| `ptr` | ✅ Pass | | |
| `collect_list` | ✅ Pass | | |
| `push_all` | ✅ Pass | | |
| `const_new` | ✅ Pass | | |
| `push_and_drain` | ✅ Pass | | |
| `push_pop_push_pop` | ✅ Pass | | |
| `remove_by_address` | ✅ Pass | | |
| `fuzz_linked_list` | ✅ Pass | | |

### util/memchr.rs (5 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `memchr` | ❌ Fail | ERR-005 | |
| `memchr` | ❌ Fail | ERR-005 | |
| `memchr_test` | ✅ Pass | | |
| `memchr_all` | ✅ Pass | | |
| `memchr_empty` | ✅ Pass | | |

### util/metric_atomics.rs (12 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `load` | ✅ Pass | | |
| `store` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `add` | ✅ Pass | | |
| `store` | ✅ Pass | | |
| `add` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `load` | ✅ Pass | | |
| `store` | ✅ Pass | | |
| `increment` | ✅ Pass | | |
| `decrement` | ✅ Pass | | |

### util/mod.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `pin_as_deref_mut` | ❌ Fail | ERR-009 | |

### util/ptr_expose.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `expose_provenance` | ✅ Pass | | |
| `from_exposed_addr` | ✅ Pass | | |
| `unexpose_provenance` | ✅ Pass | | |

### util/rand.rs (7 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `from_u64` | ✅ Pass | | |
| `from_pair` | ✅ Pass | | |
| `new` | ✅ Pass | | |
| `from_seed` | ✅ Pass | | |
| `fastrand_n` | ❌ Fail | ERR-003 | |
| `fastrand` | ✅ Pass | | |

### util/rand/rt.rs (4 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `next_seed` | ❌ Fail | ERR-003 | |
| `next_generator` | ✅ Pass | | |
| `replace_seed` | ✅ Pass | | |

### util/rand/rt_unstable.rs (1 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `from_bytes` | ✅ Pass | | |

### util/rc_cell.rs (5 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `with_inner` | ✅ Pass | | |
| `get` | ✅ Pass | | |
| `replace` | ✅ Pass | | |
| `set` | ✅ Pass | | |

### util/sharded_list.rs (11 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `pop_back` | ✅ Pass | | |
| `remove` | ✅ Pass | | |
| `lock_shard` | ✅ Pass | | |
| `len` | ✅ Pass | | |
| `added` | ✅ Pass | | |
| `is_empty` | ✅ Pass | | |
| `shard_size` | ✅ Pass | | |
| `shard_inner` | ✅ Pass | | |
| `push` | ✅ Pass | | |
| `for_each` | ✅ Pass | | |

### util/sync_wrapper.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `into_inner` | ✅ Pass | | |
| `downcast_ref_sync` | ✅ Pass | | |

### util/trace.rs (10 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `new_unnamed` | ✅ Pass | | |
| `task` | ✅ Pass | | |
| `get_span` | ✅ Pass | | |
| `blocking_task` | ✅ Pass | | |
| `async_op` | ✅ Pass | | |
| `poll` | ✅ Pass | | |
| `task` | ✅ Pass | | |
| `blocking_task` | ✅ Pass | | |
| `caller_location` | ✅ Pass | | |

### util/try_lock.rs (5 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `try_lock` | ✅ Pass | | |
| `deref` | ✅ Pass | | |
| `deref_mut` | ✅ Pass | | |
| `drop` | ✅ Pass | | |

### util/typeid.rs (3 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `try_transmute` | ✅ Pass | | |
| `nonstatic_typeid` | ✅ Pass | | |
| `get_type_id` | ✅ Pass | | |

### util/wake.rs (7 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `deref` | ✅ Pass | | |
| `waker_ref` | ✅ Pass | | |
| `waker_vtable` | ✅ Pass | | |
| `clone_arc_raw` | ✅ Pass | | |
| `wake_arc_raw` | ✅ Pass | | |
| `wake_by_ref_arc_raw` | ✅ Pass | | |
| `drop_arc_raw` | ✅ Pass | | |

### util/wake_list.rs (6 functions)
| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `new` | ✅ Pass | | |
| `can_push` | ✅ Pass | | |
| `push` | ✅ Pass | | |
| `wake_all` | ❌ Fail | unknown | |
| `drop` | ✅ Pass | | |
| `drop` | ✅ Pass | | |


---

## Summary

### Test Results

| Metric | Value |
|--------|-------|
| Total files tested | 303 |
| Files passing | 148 (49%) |
| Files failing | 155 (51%) |
| Total functions | 3,784 |
| Total errors | 621 |

### Error Distribution

| Error Code | Count | Description | BorrowScope ID |
|------------|-------|-------------|----------------|
| E0507 | 196 | Cannot move out of shared reference | ERR-009 |
| E0596 | 193 | Cannot borrow as mutable | ERR-003 |
| unknown | 78 | Macro scope issues | ERR-005 |
| E0599 | 37 | No method found | ERR-012 |
| E0425 | 32 | Cannot find value | ERR-002 |
| E0061 | 21 | Wrong argument count | ERR-010 |
| E0277 | 11 | Trait bound not satisfied | ERR-008 |
| E0308 | 10 | Type mismatch | ERR-007/009 |
| E0433 | 9 | Unresolved import | ERR-005 |
| Other | 34 | Various | - |

### Patterns Tested

| Pattern | Status | Notes |
|---------|--------|-------|
| Basic let bindings | ✅ | Works |
| Simple async functions | ✅ | Works |
| Spawn/yield | ✅ | Works |
| Error types | ✅ | Works |
| Clock/time utilities | ✅ | Works |
| Mutex/RwLock guards | ❌ | ERR-009 - guard.map() fails |
| Channel send/recv | ❌ | ERR-003, ERR-009 |
| Pin<&mut Self> methods | ❌ | ERR-009 |
| &mut self returning &mut T | ❌ | ERR-003, ERR-001 |
| Builder patterns | ❌ | ERR-003 |
| Internal cfg macros | ❌ | ERR-005 |
| Trait implementations | ❌ | ERR-012 |

### Gaps Identified

| ID | Gap | Occurrences | Severity |
|----|-----|-------------|----------|
| ERR-009 | Self-consuming methods (guard.map, Pin methods) | 196 | Critical |
| ERR-003 | Mutable method chains (&mut self) | 193 | Critical |
| ERR-005 | Macro scope/import placement | 78 | Critical |
| ERR-012 | Trait method resolution | 37 | Critical |
| ERR-002 | Struct destructuring | 32 | Critical |
| ERR-010 | Argument dropping in unsafe code | 21 | Critical |

### Comparison with Previous Projects

| Project | Files | Functions | Pass Rate |
|---------|-------|-----------|-----------|
| zoxide | 16 | 99 | 67% |
| bat | 37 | 323 | 92% |
| fd | 19 | 137 | 90% |
| ripgrep | 71 | 2,657 | 99.4% |
| **tokio** | **303** | **3,784** | **49%** |

### Key Findings

1. **Tokio has the lowest pass rate** of all tested projects (49%)
2. **Guard types are problematic** - MutexGuard, RwLockGuard methods that consume self fail
3. **Async primitives heavily use self-consuming patterns** - common in Future/Pin APIs
4. **Import placement breaks tokio's cfg macros** - need smarter import insertion
5. **ERR-009 (E0507) is the dominant error** - 196 occurrences, needs priority fix
6. **Simple async patterns work** - spawn, yield, basic futures pass
