# Section 83: File Instrumentation Engine

## Learning Objectives

By the end of this section, you will:
- Implement source code instrumentation
- Parse Rust files with syn
- Transform AST with visitor pattern
- Handle multiple files and directories
- Preserve source formatting
- Generate instrumented code

## Prerequisites

- Section 82 (Init and Check Commands)
- Deep understanding of syn and quote
- Familiarity with AST transformation from Chapter 4

---

## Instrumentation Purpose

The instrumentation engine:
1. Walks project directory tree
2. Parses Rust source files
3. Applies ownership tracking transformations
4. Writes instrumented code to temporary location
5. Preserves original code structure

---

## Complete Implementation

**src/instrumentation.rs:**

```rust
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use syn::{visit_mut::VisitMut, File};
use quote::ToTokens;

pub struct Instrumenter {
    source_dir: PathBuf,
    output_dir: PathBuf,
    config: InstrumentationConfig,
}

#[derive(Clone)]
pub struct InstrumentationConfig {
    pub track_smart_pointers: bool,
    pub track_async: bool,
    pub track_unsafe: bool,
    pub ignore_patterns: Vec<String>,
}

impl Default for InstrumentationConfig {
    fn default() -> Self {
        Self {
            track_smart_pointers: true,
            track_async: true,
            track_unsafe: false,
            ignore_patterns: vec![
                "target/**".into(),
                "**/*_test.rs".into(),
                "**/test_*.rs".into(),
            ],
        }
    }
}

impl Instrumenter {
    pub fn new(source_dir: PathBuf, config: InstrumentationConfig) -> Self {
        let output_dir = source_dir.join("target/borrowscope/instrumented");
        Self {
            source_dir,
            output_dir,
            config,
        }
    }
    
    pub fn instrument_project(&self) -> Result<InstrumentationResult> {
        std::fs::create_dir_all(&self.output_dir)?;
        
        let mut result = InstrumentationResult::default();
        
        for entry in WalkDir::new(&self.source_dir)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| !self.should_ignore(e.path()))
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                match self.instrument_file(path) {
                    Ok(output_path) => {
                        result.instrumented_files.push(output_path);
                        result.success_count += 1;
                    }
                    Err(e) => {
                        result.errors.push((path.to_path_buf(), e.to_string()));
                        result.error_count += 1;
                    }
                }
            }
        }
        
        Ok(result)
    }
    
    fn should_ignore(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        for pattern in &self.config.ignore_patterns {
            if glob_match(&path_str, pattern) {
                return true;
            }
        }
        
        false
    }
    
    fn instrument_file(&self, path: &Path) -> Result<PathBuf> {
        // Read source file
        let content = std::fs::read_to_string(path)
            .context("Failed to read source file")?;
        
        // Parse with syn
        let mut syntax = syn::parse_file(&content)
            .context("Failed to parse Rust file")?;
        
        // Apply instrumentation
        let mut visitor = InstrumentationVisitor::new(self.config.clone());
        visitor.visit_file_mut(&mut syntax);
        
        // Generate instrumented code
        let instrumented = syntax.into_token_stream().to_string();
        
        // Format with rustfmt if available
        let formatted = format_code(&instrumented).unwrap_or(instrumented);
        
        // Write to output directory
        let relative = path.strip_prefix(&self.source_dir)?;
        let output_path = self.output_dir.join(relative);
        
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(&output_path, formatted)?;
        
        Ok(output_path)
    }
}

#[derive(Default)]
pub struct InstrumentationResult {
    pub instrumented_files: Vec<PathBuf>,
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<(PathBuf, String)>,
}

struct InstrumentationVisitor {
    config: InstrumentationConfig,
    next_id: usize,
    scope_depth: usize,
}

impl InstrumentationVisitor {
    fn new(config: InstrumentationConfig) -> Self {
        Self {
            config,
            next_id: 1,
            scope_depth: 0,
        }
    }
    
    fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

impl VisitMut for InstrumentationVisitor {
    fn visit_file_mut(&mut self, file: &mut File) {
        // Add imports at the top
        let import: syn::ItemUse = syn::parse_quote! {
            use borrowscope_runtime::{track_new, track_borrow, track_borrow_mut, track_drop};
        };
        file.items.insert(0, syn::Item::Use(import));
        
        // Visit all items
        for item in &mut file.items {
            self.visit_item_mut(item);
        }
    }
    
    fn visit_item_fn_mut(&mut self, func: &mut syn::ItemFn) {
        // Skip test functions
        if has_test_attribute(&func.attrs) {
            return;
        }
        
        // Visit function body
        syn::visit_mut::visit_item_fn_mut(self, func);
    }
    
    fn visit_local_mut(&mut self, local: &mut syn::Local) {
        // Transform: let x = value;
        // Into: let x = track_new(ID, "x", value);
        
        if let Some(init) = &mut local.init {
            let id = self.next_id();
            let name = extract_pattern_name(&local.pat);
            
            let new_init: syn::Expr = syn::parse_quote! {
                track_new(#id, #name, #init.expr)
            };
            
            init.expr = Box::new(new_init);
        }
        
        syn::visit_mut::visit_local_mut(self, local);
    }
    
    fn visit_expr_mut(&mut self, expr: &mut syn::Expr) {
        match expr {
            syn::Expr::Reference(ref_expr) => {
                // Transform: &x or &mut x
                let id = self.next_id();
                let is_mut = ref_expr.mutability.is_some();
                
                let track_fn = if is_mut {
                    quote::quote! { track_borrow_mut }
                } else {
                    quote::quote! { track_borrow }
                };
                
                let inner = &ref_expr.expr;
                
                *expr = syn::parse_quote! {
                    #track_fn(#id, #inner)
                };
            }
            syn::Expr::Block(block_expr) => {
                self.scope_depth += 1;
                syn::visit_mut::visit_expr_mut(self, expr);
                self.scope_depth -= 1;
            }
            _ => {
                syn::visit_mut::visit_expr_mut(self, expr);
            }
        }
    }
}

fn has_test_attribute(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("test") || attr.path().is_ident("tokio::test")
    })
}

fn extract_pattern_name(pat: &syn::Pat) -> String {
    match pat {
        syn::Pat::Ident(ident) => ident.ident.to_string(),
        syn::Pat::Type(pat_type) => extract_pattern_name(&pat_type.pat),
        _ => "<complex>".to_string(),
    }
}

fn glob_match(path: &str, pattern: &str) -> bool {
    // Simple glob matching (simplified)
    if pattern.contains("**") {
        let parts: Vec<&str> = pattern.split("**").collect();
        if parts.len() == 2 {
            return path.contains(parts[0]) && path.ends_with(parts[1]);
        }
    }
    
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        return parts.iter().all(|part| path.contains(part));
    }
    
    path == pattern
}

fn format_code(code: &str) -> Result<String> {
    use std::process::{Command, Stdio};
    use std::io::Write;
    
    let mut child = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(code.as_bytes())?;
    }
    
    let output = child.wait_with_output()?;
    
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        Ok(code.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match() {
        assert!(glob_match("target/debug/main.rs", "target/**"));
        assert!(glob_match("src/test_foo.rs", "**/test_*.rs"));
        assert!(!glob_match("src/main.rs", "**/test_*.rs"));
    }

    #[test]
    fn test_extract_pattern_name() {
        let pat: syn::Pat = syn::parse_quote! { x };
        assert_eq!(extract_pattern_name(&pat), "x");
        
        let pat: syn::Pat = syn::parse_quote! { x: i32 };
        assert_eq!(extract_pattern_name(&pat), "x");
    }
}
```

**Progress:** 8/13
