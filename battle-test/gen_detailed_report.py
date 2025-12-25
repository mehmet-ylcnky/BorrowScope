#!/usr/bin/env python3
"""
Generate detailed per-function results for battle test report.
"""

import subprocess
import re
import json
import sys
from pathlib import Path

def get_functions_with_lines(filepath):
    """Extract function names and line numbers from a Rust file."""
    functions = []
    fn_pattern = re.compile(
        r'^(\s*)(?:#\[trace_borrow\]\s*\n\s*)?'
        r'((?:pub(?:\s*\([^)]*\))?\s+)?'
        r'(?:default\s+)?'
        r'(?:async\s+)?'
        r'(?:unsafe\s+)?'
        r'(?:extern\s+"[^"]*"\s+)?'
        r'fn\s+(\w+))'
    )
    
    with open(filepath, 'r') as f:
        lines = f.readlines()
    
    for i, line in enumerate(lines, 1):
        if 'const fn' in line:
            continue
        match = fn_pattern.match(line)
        if match:
            fn_name = match.group(3)
            # Check if trait method (no body)
            is_trait = False
            for j in range(i-1, min(i+9, len(lines))):
                if '{' in lines[j]:
                    break
                if lines[j].strip().endswith(';'):
                    is_trait = True
                    break
            if not is_trait:
                functions.append({'name': fn_name, 'line': i})
    
    return functions

def map_error_to_id(code, message):
    """Map Rust error code to BorrowScope error ID."""
    mapping = {
        'E0507': 'ERR-009',  # Cannot move out of shared reference
        'E0596': 'ERR-003',  # Cannot borrow as mutable
        'E0515': 'ERR-001',  # Cannot return reference to parameter
        'E0425': 'ERR-002',  # Cannot find value (tuple destructuring)
        'E0015': 'ERR-004',  # Cannot call non-const in const
        'E0433': 'ERR-005',  # Unresolved import
        'E0716': 'ERR-006',  # Temporary dropped
        'E0308': 'ERR-007',  # Type mismatch
        'E0277': 'ERR-008',  # Trait bound not satisfied
        'E0282': 'ERR-008',  # Type annotations needed
        'E0061': 'ERR-010',  # Wrong argument count
        'E0609': 'ERR-011',  # No field on type
        'E0599': 'ERR-012',  # No method found
        'E0407': 'ERR-012',  # Method not in trait
        'E0597': 'ERR-013',  # Lifetime mismatch
    }
    if code == 'unknown':
        if 'cannot find attribute' in message or 'cannot find macro' in message:
            return 'ERR-005'
        return 'unknown'
    return mapping.get(code, code)

def main():
    if len(sys.argv) < 2:
        print("Usage: python gen_detailed_report.py <results_json>")
        sys.exit(1)
    
    with open(sys.argv[1]) as f:
        data = json.load(f)
    
    repo_path = sys.argv[2] if len(sys.argv) > 2 else '.'
    
    for filepath, info in sorted(data.items()):
        if info['functions'] == 0:
            continue
            
        full_path = Path(repo_path) / filepath
        if not full_path.exists():
            continue
        
        functions = get_functions_with_lines(full_path)
        errors_by_line = {}
        for e in info['errors']:
            line = e['line']
            err_id = map_error_to_id(e['code'], e.get('message', ''))
            if line not in errors_by_line:
                errors_by_line[line] = set()
            errors_by_line[line].add(err_id)
        
        # Map errors to functions (error line is usually at or after function line)
        fn_errors = {}
        for fn in functions:
            fn_errors[fn['name']] = set()
        
        for err_line, err_ids in errors_by_line.items():
            # Find the function containing this error
            containing_fn = None
            for fn in sorted(functions, key=lambda x: x['line'], reverse=True):
                if fn['line'] <= err_line:
                    containing_fn = fn['name']
                    break
            if containing_fn:
                fn_errors[containing_fn].update(err_ids)
        
        # Print markdown table
        short_path = filepath.replace('tokio/src/', '')
        pass_count = sum(1 for fn in functions if not fn_errors.get(fn['name']))
        fail_count = len(functions) - pass_count
        
        print(f"\n### {short_path} ({len(functions)} functions)")
        print("| Function | Status | Error | Notes |")
        print("|----------|--------|-------|-------|")
        
        for fn in functions:
            errs = fn_errors.get(fn['name'], set())
            if errs:
                err_str = ','.join(sorted(errs))
                print(f"| `{fn['name']}` | ❌ Fail | {err_str} | |")
            else:
                print(f"| `{fn['name']}` | ✅ Pass | | |")

if __name__ == '__main__':
    main()
