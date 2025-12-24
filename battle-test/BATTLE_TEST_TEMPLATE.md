# BorrowScope Battle Test: {PROJECT_NAME}

**Project:** [{PROJECT_NAME}](https://github.com/{OWNER}/{PROJECT_NAME})  
**Version:** Latest (cloned on YYYY-MM-DD)  
**Stars:** ~Xk  
**Description:** {One-line project description}  

**Note:** "Pass" means the macro transformation compiles without errors. Full runtime verification is blocked by any compilation failures.

---

## Phase 1: Reconnaissance

### Lines of Code
```
~X,XXX lines total
```

### Key Modules to Test
| Module | Description | Ownership Patterns |
|--------|-------------|-------------------|
| `src/xxx.rs` | Description | Patterns to expect |

---

## Error Log

<!-- Document each unique error encountered. Use sequential ERR-XXX IDs. -->

### ERR-XXX: {Error Title}

**Location:**
- File: `src/xxx.rs`
- Line: XX
- Function: `fn function_name()`

**Error Message:**
```
error[EXXXX]: error message
  --> src/xxx.rs:XX:XX
```

**Category:** {Type Transformation Issue | Lifetime Issue | Borrow Issue | etc.}  
**Severity:** Critical  
**Component:** borrowscope-macro  

**Minimal Reproducer:**
```rust
#[borrowscope_macro::trace_borrow]
fn example() {
    // minimal code that triggers the error
}
```

**Root Cause:**
{Explanation of why the macro transformation fails}

**Proposed Solution:**
- File: `borrowscope-macro/src/xxx.rs`
- Change: {Description of fix}

**Workaround:** 
{How to avoid this error until fixed}

---

## Progress Log

<!-- Document each file tested with pass/fail counts -->

### src/xxx.rs (X functions)

| Function | Status | Error | Notes |
|----------|--------|-------|-------|
| `fn_name` | ✅ Pass | - | - |
| `fn_name2` | ❌ Fail | ERR-XXX | {brief note} |

---

## Summary

### Test Results by File

| File | Pass | Fail | Total | Pass Rate |
|------|------|------|-------|-----------|
| xxx.rs | X | X | X | XX% |
| **TOTAL** | **X** | **X** | **X** | **XX%** |

### Patterns Tested

| Pattern | Status | Notes |
|---------|--------|-------|
| Basic let bindings | ✅ | Works |
| Immutable borrows | ✅ | Works |
| Mutable borrows | ⚠️ | Partial |
| impl Into<T> parameters | ❌ | ERR-XXX |

### Gaps Identified

#### Critical (Blocks Usage)
| ID | Gap | Description | Workaround |
|----|-----|-------------|------------|
| ERR-XXX | Gap name | Description | Workaround |
