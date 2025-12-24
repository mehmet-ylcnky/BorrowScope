# tracker.rs Restructuring Plan

## Current State
- **File**: `src/tracker.rs` - 4,603 lines
- **125 public functions** + Tracker struct with ~70 methods
- All tracking logic in single file

## Proposed Module Structure

```
src/
├── tracker/
│   ├── mod.rs              # Re-exports, TRACKER static, Tracker struct
│   ├── core.rs             # Basic: track_new, track_borrow, track_move, track_drop
│   ├── smart_pointers.rs   # Rc, Arc, Box, Weak, Pin, Cow
│   ├── interior_mut.rs     # RefCell, Cell, OnceCell, OnceLock
│   ├── unsafe_code.rs      # Raw pointers, unsafe blocks, FFI, transmute, unions
│   ├── concurrency.rs      # Threads, channels, lock guards
│   ├── async_tracking.rs   # Async blocks, await, futures
│   ├── control_flow.rs     # Loops, match, branches, break, continue, return
│   ├── expressions.rs      # Index, field access, calls, binary ops, casts
│   ├── statics.rs          # Static init/access, const eval
│   ├── sampling.rs         # should_sample, *_sampled variants
│   ├── query.rs            # get_events, get_summary, filters
│   └── maybe_uninit.rs     # MaybeUninit operations
```

## Function Distribution

### mod.rs (~200 lines)
- `TRACKER` static, `TIMESTAMP` atomic
- `Tracker` struct definition
- `Tracker::new()`, `Default` impl
- Re-exports from all submodules

### core.rs (~400 lines)
- `track_new`, `track_new_with_id`
- `track_borrow`, `track_borrow_with_id`
- `track_borrow_mut`, `track_borrow_mut_with_id`
- `track_move`, `track_move_with_id`
- `track_drop`, `track_drop_with_id`, `track_drop_batch`
- `Tracker::record_new`, `record_borrow`, `record_move`, `record_drop`

### smart_pointers.rs (~500 lines)
- `track_rc_new`, `track_rc_clone` (+ `_with_id` variants)
- `track_arc_new`, `track_arc_clone` (+ `_with_id` variants)
- `track_weak_new`, `track_weak_clone`, `track_weak_upgrade` (+ sync variants)
- `track_box_new`, `track_box_into_raw`, `track_box_from_raw`
- `track_pin_new`, `track_pin_into_inner`
- `track_cow_borrowed`, `track_cow_owned`, `track_cow_to_mut`
- Corresponding `Tracker::record_*` methods

### interior_mut.rs (~350 lines)
- `track_refcell_new`, `track_refcell_borrow`, `track_refcell_borrow_mut`, `track_refcell_drop`
- `track_cell_new`, `track_cell_get`, `track_cell_set`
- `track_once_cell_new`, `track_once_cell_set`, `track_once_cell_get`, `track_once_cell_get_or_init`
- `track_once_lock_new`
- Corresponding `Tracker::record_*` methods

### unsafe_code.rs (~300 lines)
- `track_raw_ptr`, `track_raw_ptr_mut`, `track_raw_ptr_deref`
- `track_unsafe_block_enter`, `track_unsafe_block_exit`
- `track_unsafe_fn_call`, `track_ffi_call`
- `track_transmute`, `track_union_field_access`
- Corresponding `Tracker::record_*` methods

### concurrency.rs (~250 lines)
- `track_thread_spawn`, `track_thread_join`
- `track_channel`, `track_channel_send`, `track_channel_recv`, `track_channel_try_recv`
- `track_lock`, `track_lock_guard_acquire`, `track_lock_guard_drop`
- Corresponding `Tracker::record_*` methods

### async_tracking.rs (~200 lines)
- `track_async_block_enter`, `track_async_block_exit`
- `track_await_start`, `track_await_end`
- `track_future_create`, `track_future_poll`
- Corresponding `Tracker::record_*` methods

### control_flow.rs (~350 lines)
- `track_loop_enter`, `track_loop_iteration`, `track_loop_exit`
- `track_match_enter`, `track_match_arm`, `track_match_exit`
- `track_branch`, `track_return`, `track_try`
- `track_break`, `track_continue`
- `track_fn_enter`, `track_fn_exit`
- `track_region_enter`, `track_region_exit`
- Corresponding `Tracker::record_*` methods

### expressions.rs (~300 lines)
- `track_index_access`, `track_field_access`
- `track_call`, `track_unwrap`, `track_clone`, `track_deref`
- `track_closure_create`, `track_closure_capture`
- `track_struct_create`, `track_tuple_create`, `track_array_create`
- `track_let_else`, `track_range`, `track_binary_op`, `track_type_cast`
- Corresponding `Tracker::record_*` methods

### statics.rs (~150 lines)
- `track_static_init`, `track_static_access`
- `track_const_eval`
- Corresponding `Tracker::record_*` methods

### sampling.rs (~200 lines)
- `should_sample`
- `track_new_sampled`, `track_borrow_sampled`, `track_borrow_mut_sampled`
- `track_drop_sampled`, `track_move_sampled`
- `track_new_with_id_sampled`, etc.

### query.rs (~300 lines)
- `reset`, `get_events`, `get_events_filtered`
- `get_new_events`, `get_borrow_events`, `get_drop_events`, `get_move_events`
- `get_events_for_var`, `get_event_counts`
- `TrackingSummary` struct, `get_summary`, `print_summary`

### maybe_uninit.rs (~150 lines)
- `track_maybe_uninit_uninit`, `track_maybe_uninit_new`
- `track_maybe_uninit_write`, `track_maybe_uninit_assume_init`
- `track_maybe_uninit_assume_init_read`, `track_maybe_uninit_assume_init_drop`
- Corresponding `Tracker::record_*` methods

## Implementation Strategy

### Phase 1: Create module structure
1. Create `src/tracker/` directory
2. Create `mod.rs` with Tracker struct and re-exports
3. Move functions incrementally, one module at a time
4. Keep `src/tracker.rs` as fallback until complete

### Phase 2: Extract modules (order)
1. `query.rs` - No dependencies on other modules
2. `sampling.rs` - Only depends on core tracking
3. `statics.rs` - Simple, few functions
4. `maybe_uninit.rs` - Self-contained
5. `control_flow.rs` - Self-contained
6. `expressions.rs` - Self-contained
7. `async_tracking.rs` - Self-contained
8. `concurrency.rs` - Self-contained
9. `unsafe_code.rs` - Self-contained
10. `interior_mut.rs` - Self-contained
11. `smart_pointers.rs` - Larger but self-contained
12. `core.rs` - Last, as others may reference patterns

### Phase 3: Cleanup
1. Delete old `src/tracker.rs`
2. Update `src/lib.rs` to use `mod tracker;`
3. Verify all tests pass
4. Update documentation

## Key Considerations

### Tracker Methods
The `Tracker` struct has ~70 `record_*` methods. Options:
1. **Keep in mod.rs** - Single impl block, methods call into submodules
2. **Split impl blocks** - Each module has `impl Tracker` for its methods
3. **Trait-based** - Define traits per category, implement on Tracker

Recommendation: **Option 2** - Split impl blocks. Rust allows multiple impl blocks for the same struct across modules. Each module owns its `record_*` methods.

### Visibility
- `TRACKER` static: `pub(crate)` in mod.rs
- `Tracker` struct: `pub(crate)` 
- Public functions: `pub` with re-export from `lib.rs`
- `record_*` methods: `pub(crate)`

### Testing
- Keep existing tests in `tests/` directory
- They test public API, should work unchanged
- Add module-specific unit tests in each file

## Estimated Effort
- **Phase 1**: 30 minutes
- **Phase 2**: 2-3 hours (incremental, test after each module)
- **Phase 3**: 30 minutes
- **Total**: ~4 hours

## Benefits
- Easier navigation (12 focused files vs 1 massive file)
- Better code organization by domain
- Easier to add new tracking categories
- Clearer ownership of functionality
- IDE performance improvement
