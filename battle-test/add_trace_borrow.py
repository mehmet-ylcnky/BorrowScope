#!/usr/bin/env python3
"""
Add #[trace_borrow] macro to all functions in Rust files.
Handles imports properly and avoids doc comments.
"""

import re
import sys
import os
from pathlib import Path

def find_real_code_start(content):
    """Find where actual code starts (after doc comments and attributes)."""
    lines = content.split('\n')
    in_block_comment = False
    
    for i, line in enumerate(lines):
        stripped = line.strip()
        
        # Track block comments
        if '/*' in stripped and '*/' not in stripped:
            in_block_comment = True
            continue
        if '*/' in stripped:
            in_block_comment = False
            continue
        if in_block_comment:
            continue
            
        # Skip line comments
        if stripped.startswith('//'):
            continue
            
        # Skip empty lines
        if not stripped:
            continue
            
        # Found real code - check if it's an attribute or use/mod statement
        if stripped.startswith('#![') or stripped.startswith('#['):
            continue
        if stripped.startswith('use ') or stripped.startswith('pub use '):
            return i
        if stripped.startswith('mod ') or stripped.startswith('pub mod '):
            return i
        if stripped.startswith('extern '):
            return i
        # Any other code
        return i
    
    return 0

def add_import(content, filepath):
    """Add borrowscope import after inner attributes AND doc comments."""
    if 'borrowscope_macro::trace_borrow' in content:
        return content
    
    lines = content.split('\n')
    insert_line = 0
    in_multiline_attr = False
    
    for i, line in enumerate(lines):
        stripped = line.strip()
        
        # Track multi-line attributes like #![allow(\n...\n)]
        if stripped.startswith('#![') and not stripped.endswith(']'):
            in_multiline_attr = True
            insert_line = i + 1
            continue
        if in_multiline_attr:
            if ']' in stripped:
                in_multiline_attr = False
                insert_line = i + 1
            continue
        
        if stripped.startswith('#!['):  # Single-line inner attribute
            insert_line = i + 1
        elif stripped.startswith('//!'):  # Inner doc comment
            insert_line = i + 1
        elif stripped.startswith('/*!'):  # Block inner doc comment
            insert_line = i + 1
        elif stripped:  # Non-empty = stop
            break
    
    lines.insert(insert_line, 'use borrowscope_macro::trace_borrow;')
    return '\n'.join(lines)

def add_trace_borrow(content):
    """Add #[trace_borrow] to all function definitions with bodies."""
    lines = content.split('\n')
    result = []
    
    # Pattern for function definitions
    fn_pattern = re.compile(
        r'^(\s*)'  # Leading whitespace
        r'((?:pub(?:\s*\([^)]*\))?\s+)?'  # Optional pub/pub(crate)/etc
        r'(?:default\s+)?'  # Optional default
        r'(?:async\s+)?'  # Optional async
        r'(?:unsafe\s+)?'  # Optional unsafe
        r'(?:extern\s+"[^"]*"\s+)?'  # Optional extern "C"
        r'(?:const\s+)?'  # Optional const
        r'fn\s+\w+)'  # fn keyword and name
    )
    
    i = 0
    while i < len(lines):
        line = lines[i]
        match = fn_pattern.match(line)
        
        if match:
            indent = match.group(1)
            # Check if previous non-empty line already has trace_borrow
            prev_idx = i - 1
            while prev_idx >= 0 and not lines[prev_idx].strip():
                prev_idx -= 1
            
            if prev_idx >= 0 and '#[trace_borrow]' in lines[prev_idx]:
                result.append(line)
            else:
                # Skip const fn - can't be tracked
                if 'const fn' in line:
                    result.append(line)
                else:
                    # Check if this is a trait method declaration (ends with ; not {)
                    # Look ahead to find if function has a body or ends with semicolon
                    is_trait_method = False
                    check_line = line
                    check_idx = i
                    # Accumulate lines until we find { or ;
                    while check_idx < len(lines):
                        check_line = lines[check_idx].strip()
                        if '{' in check_line:
                            break
                        if check_line.endswith(';'):
                            is_trait_method = True
                            break
                        check_idx += 1
                    
                    if is_trait_method:
                        result.append(line)
                    else:
                        result.append(f'{indent}#[trace_borrow]')
                        result.append(line)
        else:
            result.append(line)
        i += 1
    
    return '\n'.join(result)

def process_file(filepath):
    """Process a single Rust file."""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Add import
        content = add_import(content, filepath)
        
        # Add trace_borrow to functions
        content = add_trace_borrow(content)
        
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)
        
        return True
    except Exception as e:
        print(f"Error processing {filepath}: {e}", file=sys.stderr)
        return False

def main():
    if len(sys.argv) < 2:
        print("Usage: python add_trace_borrow.py <directory_or_file> [--exclude pattern1,pattern2]")
        sys.exit(1)
    
    target = sys.argv[1]
    excludes = ['target', 'tests', 'test', 'benches', 'examples', 'build.rs']
    
    # Parse --exclude argument
    if '--exclude' in sys.argv:
        idx = sys.argv.index('--exclude')
        if idx + 1 < len(sys.argv):
            excludes.extend(sys.argv[idx + 1].split(','))
    
    if os.path.isfile(target):
        if process_file(target):
            print(f"Processed: {target}")
    else:
        # Process directory
        processed = 0
        skipped = 0
        for filepath in Path(target).rglob('*.rs'):
            # Check excludes
            skip = False
            for exc in excludes:
                if exc in str(filepath):
                    skip = True
                    break
            
            if skip:
                skipped += 1
                continue
            
            if process_file(str(filepath)):
                processed += 1
                print(f"Processed: {filepath}")
        
        print(f"\nTotal: {processed} files processed, {skipped} skipped")

if __name__ == '__main__':
    main()
