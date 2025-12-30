#!/usr/bin/env python3
"""
Test each Rust file individually by compiling with trace_borrow.
Outputs JSON with per-function pass/fail status.
"""

import subprocess
import re
import json
import sys
import os
from pathlib import Path
from collections import defaultdict

def get_functions_in_file(filepath):
    """Extract function names and line numbers from a Rust file."""
    functions = []
    fn_pattern = re.compile(
        r'^(\s*)'
        r'(?:#\[trace_borrow\]\s*\n\s*)?'
        r'((?:pub(?:\s*\([^)]*\))?\s+)?'
        r'(?:default\s+)?'
        r'(?:async\s+)?'
        r'(?:unsafe\s+)?'
        r'(?:extern\s+"[^"]*"\s+)?'
        r'(?:const\s+)?'
        r'fn\s+(\w+))'
    )
    
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            lines = f.readlines()
        
        for i, line in enumerate(lines, 1):
            match = fn_pattern.match(line)
            if match:
                fn_name = match.group(3)
                # Check if it has trace_borrow (look at previous line)
                has_trace = i > 1 and '#[trace_borrow]' in lines[i-2]
                # Check if it's a trait method (no body)
                is_trait_method = False
                for j in range(i-1, min(i+10, len(lines))):
                    if '{' in lines[j]:
                        break
                    if lines[j].strip().endswith(';'):
                        is_trait_method = True
                        break
                
                if has_trace and not is_trait_method:
                    functions.append({'name': fn_name, 'line': i})
    except Exception as e:
        print(f"Error reading {filepath}: {e}", file=sys.stderr)
    
    return functions

def run_cargo_check(repo_path, package, features=None):
    """Run cargo check and return errors."""
    cmd = ['cargo', 'check', '-p', package, '--message-format=json']
    if features:
        cmd.extend(['--features', features])
    
    result = subprocess.run(
        cmd,
        cwd=repo_path,
        capture_output=True,
        text=True
    )
    
    errors = []
    for line in result.stdout.split('\n'):
        if not line.strip():
            continue
        try:
            msg = json.loads(line)
            if msg.get('reason') == 'compiler-message':
                message = msg.get('message', {})
                if message.get('level') == 'error':
                    spans = message.get('spans', [])
                    for span in spans:
                        if span.get('is_primary'):
                            code_info = message.get('code')
                            code = code_info.get('code', 'unknown') if code_info else 'unknown'
                            errors.append({
                                'file': span.get('file_name', ''),
                                'line': span.get('line_start', 0),
                                'code': code,
                                'message': message.get('message', '')
                            })
                            break
        except json.JSONDecodeError:
            continue
    
    return errors

def map_errors_to_functions(errors, functions_by_file):
    """Map errors to specific functions."""
    error_map = defaultdict(list)
    
    for error in errors:
        filepath = error['file']
        line = error['line']
        
        if filepath in functions_by_file:
            funcs = functions_by_file[filepath]
            # Find the function containing this line
            containing_fn = None
            for fn in sorted(funcs, key=lambda x: x['line'], reverse=True):
                if fn['line'] <= line:
                    containing_fn = fn['name']
                    break
            
            if containing_fn:
                key = f"{filepath}::{containing_fn}"
                error_map[key].append({
                    'code': error['code'],
                    'message': error['message'],
                    'line': line
                })
    
    return error_map

def main():
    if len(sys.argv) < 4:
        print("Usage: python test_files.py <repo_path> <src_dir> <package> [features]")
        sys.exit(1)
    
    repo_path = sys.argv[1]
    src_dir = sys.argv[2]
    package = sys.argv[3]
    features = sys.argv[4] if len(sys.argv) > 4 else None
    
    # Get all Rust files
    src_path = Path(repo_path) / src_dir
    rust_files = list(src_path.rglob('*.rs'))
    
    # Filter out test files
    rust_files = [f for f in rust_files if '/tests/' not in str(f) and '/test/' not in str(f)]
    
    print(f"Found {len(rust_files)} Rust files", file=sys.stderr)
    
    # Get functions from each file
    functions_by_file = {}
    total_functions = 0
    for filepath in rust_files:
        rel_path = str(filepath.relative_to(repo_path))
        funcs = get_functions_in_file(filepath)
        if funcs:
            functions_by_file[rel_path] = funcs
            total_functions += len(funcs)
    
    print(f"Found {total_functions} functions with #[trace_borrow]", file=sys.stderr)
    
    # Run cargo check once
    print("Running cargo check...", file=sys.stderr)
    errors = run_cargo_check(repo_path, package, features)
    print(f"Found {len(errors)} errors", file=sys.stderr)
    
    # Map errors to functions
    error_map = map_errors_to_functions(errors, functions_by_file)
    
    # Build results
    results = {
        'files': {},
        'summary': {
            'total_files': len(functions_by_file),
            'total_functions': total_functions,
            'total_errors': len(errors),
            'functions_with_errors': len(error_map)
        }
    }
    
    for filepath, funcs in sorted(functions_by_file.items()):
        file_results = []
        for fn in funcs:
            key = f"{filepath}::{fn['name']}"
            if key in error_map:
                file_results.append({
                    'name': fn['name'],
                    'line': fn['line'],
                    'status': 'fail',
                    'errors': error_map[key]
                })
            else:
                file_results.append({
                    'name': fn['name'],
                    'line': fn['line'],
                    'status': 'pass',
                    'errors': []
                })
        results['files'][filepath] = file_results
    
    # Output JSON
    print(json.dumps(results, indent=2))

if __name__ == '__main__':
    main()
