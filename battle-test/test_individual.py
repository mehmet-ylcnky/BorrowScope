#!/usr/bin/env python3
"""
Test each Rust file individually by:
1. Starting with clean repo (no trace_borrow)
2. Adding trace_borrow to ONE file
3. Running cargo check
4. Recording results
5. Reverting and moving to next file
"""

import subprocess
import re
import json
import sys
import os
import shutil
from pathlib import Path

def add_trace_borrow_to_file(filepath):
    """Add trace_borrow import and annotations to a single file."""
    with open(filepath, 'r') as f:
        content = f.read()

    # Add import
    if 'borrowscope_macro::trace_borrow' not in content:
        lines = content.split('\n')
        # Find first use statement
        insert_idx = 0
        for i, line in enumerate(lines):
            if line.strip().startswith('use ') or line.strip().startswith('pub use '):
                insert_idx = i
                break
        lines.insert(insert_idx, 'use borrowscope_macro::trace_borrow;')
        content = '\n'.join(lines)

    # Add #[trace_borrow] to functions
    fn_pattern = re.compile(
        r'^(\s*)((?:pub(?:\s*\([^)]*\))?\s+)?(?:default\s+)?(?:async\s+)?(?:unsafe\s+)?(?:extern\s+"[^"]*"\s+)?fn\s+\w+)'
    )

    lines = content.split('\n')
    result = []
    i = 0
    while i < len(lines):
        line = lines[i]
        match = fn_pattern.match(line)
        if match and 'const fn' not in line:
            indent = match.group(1)
            # Check if already has trace_borrow
            if i > 0 and '#[trace_borrow]' in lines[i-1]:
                result.append(line)
            else:
                # Check if trait method (ends with ;)
                is_trait = False
                for j in range(i, min(i+10, len(lines))):
                    if '{' in lines[j]:
                        break
                    if lines[j].strip().endswith(';'):
                        is_trait = True
                        break
                if not is_trait:
                    result.append(f'{indent}#[trace_borrow]')
        result.append(lines[i])
        i += 1

    with open(filepath, 'w') as f:
        f.write('\n'.join(result))

def run_cargo_check(repo_path, package, features=None):
    """Run cargo check and return (success, errors)."""
    cmd = ['cargo', 'check', '-p', package, '--message-format=json']
    if features:
        cmd.extend(['--features', features])

    result = subprocess.run(cmd, cwd=repo_path, capture_output=True, text=True)

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
                                'message': message.get('message', '')[:100]
                            })
        except:
            pass

    return len(errors) == 0, errors

def get_functions_in_file(filepath):
    """Count functions that would get trace_borrow."""
    count = 0
    fn_pattern = re.compile(r'^\s*(?:pub(?:\s*\([^)]*\))?\s+)?(?:default\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+(\w+)')

    with open(filepath, 'r') as f:
        lines = f.readlines()

    for i, line in enumerate(lines):
        if 'const fn' in line:
            continue
        match = fn_pattern.match(line)
        if match:
            # Check if trait method
            is_trait = False
            for j in range(i, min(i+10, len(lines))):
                if '{' in lines[j]:
                    break
                if lines[j].strip().endswith(';'):
                    is_trait = True
                    break
            if not is_trait:
                count += 1
    return count

def main():
    if len(sys.argv) < 4:
        print("Usage: python test_individual.py <repo_path> <src_dir> <package> [features]")
        sys.exit(1)

    repo_path = sys.argv[1]
    src_dir = sys.argv[2]
    package = sys.argv[3]
    features = sys.argv[4] if len(sys.argv) > 4 else None

    src_path = Path(repo_path) / src_dir
    rust_files = sorted([f for f in src_path.rglob('*.rs')
                        if '/tests/' not in str(f) and '/test/' not in str(f)])

    print(f"Found {len(rust_files)} files to test", file=sys.stderr)

    results = {}

    for idx, filepath in enumerate(rust_files):
        rel_path = str(filepath.relative_to(repo_path))
        fn_count = get_functions_in_file(filepath)

        if fn_count == 0:
            print(f"[{idx+1}/{len(rust_files)}] {rel_path}: 0 functions, skipping", file=sys.stderr)
            continue

        # Save original
        with open(filepath, 'r') as f:
            original = f.read()

        try:
            # Add trace_borrow
            add_trace_borrow_to_file(filepath)

            # Run cargo check
            print(f"[{idx+1}/{len(rust_files)}] Testing {rel_path} ({fn_count} functions)...", file=sys.stderr)
            success, errors = run_cargo_check(repo_path, package, features)

            # Filter errors to this file only
            file_errors = [e for e in errors if rel_path in e['file']]

            results[rel_path] = {
                'functions': fn_count,
                'success': len(file_errors) == 0,
                'errors': file_errors
            }

            status = "✅" if len(file_errors) == 0 else f"❌ ({len(file_errors)} errors)"
            print(f"  -> {status}", file=sys.stderr)

        finally:
            # Restore original
            with open(filepath, 'w') as f:
                f.write(original)

    print(json.dumps(results, indent=2))

if __name__ == '__main__':
    main()
