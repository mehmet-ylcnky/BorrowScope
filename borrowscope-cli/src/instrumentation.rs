//! Source code instrumentation engine
//!
//! This module provides functionality to instrument Rust source code with
//! ownership tracking calls. It walks directory trees, parses Rust files,
//! applies AST transformations, and generates instrumented code.

use anyhow::{Context, Result};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use syn::visit_mut::VisitMut;

/// Configuration for the instrumentation process
#[derive(Clone, Debug)]
pub struct InstrumentationConfig {
    /// Track smart pointers (Box, Rc, Arc, etc.)
    #[allow(dead_code)]
    pub track_smart_pointers: bool,
    /// Track async code
    #[allow(dead_code)]
    pub track_async: bool,
    /// Track unsafe code blocks
    pub track_unsafe: bool,
    /// Additional patterns to ignore
    #[allow(dead_code)]
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
                "**/tests/**".into(),
                "**/benches/**".into(),
            ],
        }
    }
}

/// Main instrumentation engine
#[allow(dead_code)]
pub struct Instrumenter {
    source_dir: PathBuf,
    output_dir: PathBuf,
    config: InstrumentationConfig,
}

#[allow(dead_code)]
impl Instrumenter {
    /// Create a new instrumenter
    pub fn new(source_dir: PathBuf, output_dir: PathBuf, config: InstrumentationConfig) -> Self {
        Self {
            source_dir,
            output_dir,
            config,
        }
    }

    /// Instrument an entire project
    pub fn instrument_project(&self) -> Result<InstrumentationResult> {
        std::fs::create_dir_all(&self.output_dir).context("Failed to create output directory")?;

        let mut result = InstrumentationResult::default();

        let walker = WalkBuilder::new(&self.source_dir)
            .hidden(false)
            .git_ignore(true)
            .build();

        for entry in walker.filter_map(|e| e.ok()) {
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }

            if self.should_ignore(path) {
                result.skipped_files.push(path.to_path_buf());
                result.skipped_count += 1;
                continue;
            }

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

        Ok(result)
    }

    /// Instrument a single file
    pub fn instrument_file(&self, path: &Path) -> Result<PathBuf> {
        // Read source file
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        // Parse with syn
        let mut syntax = syn::parse_file(&content)
            .with_context(|| format!("Failed to parse Rust file: {}", path.display()))?;

        // Apply instrumentation
        let mut visitor = InstrumentationVisitor::new(self.config.clone());
        visitor.visit_file_mut(&mut syntax);

        // Generate instrumented code
        let instrumented = prettyplease::unparse(&syntax);

        // Determine output path
        let relative = path.strip_prefix(&self.source_dir).unwrap_or(path);
        let output_path = self.output_dir.join(relative);

        // Create parent directories
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        // Write instrumented code
        std::fs::write(&output_path, instrumented)
            .with_context(|| format!("Failed to write file: {}", output_path.display()))?;

        Ok(output_path)
    }

    fn should_ignore(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check if path contains test-related directories
        if path_str.contains("/tests/")
            || path_str.contains("/benches/")
            || path_str.contains("/target/")
        {
            return true;
        }

        // Check custom ignore patterns
        for pattern in &self.config.ignore_patterns {
            if self.matches_pattern(&path_str, pattern) {
                return true;
            }
        }

        false
    }

    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        // Simple glob matching
        if pattern.contains("**") {
            let parts: Vec<&str> = pattern.split("**").collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1].trim_start_matches('/');

                if !prefix.is_empty() && !path.starts_with(prefix) {
                    return false;
                }

                if !suffix.is_empty() {
                    // Match suffix pattern
                    if suffix.contains('*') {
                        let suffix_parts: Vec<&str> = suffix.split('*').collect();
                        return suffix_parts.iter().all(|part| path.contains(part));
                    }
                    return path.ends_with(suffix);
                }

                return true;
            }
        }

        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            let mut last_pos = 0;
            for part in parts {
                if part.is_empty() {
                    continue;
                }
                if let Some(pos) = path[last_pos..].find(part) {
                    last_pos += pos + part.len();
                } else {
                    return false;
                }
            }
            return true;
        }

        path.contains(pattern)
    }
}

/// Result of instrumentation process
#[derive(Default, Debug)]
#[allow(dead_code)]
pub struct InstrumentationResult {
    /// Successfully instrumented files
    pub instrumented_files: Vec<PathBuf>,
    /// Skipped files (tests, etc.)
    pub skipped_files: Vec<PathBuf>,
    /// Number of successful instrumentations
    pub success_count: usize,
    /// Number of skipped files
    pub skipped_count: usize,
    /// Number of errors
    pub error_count: usize,
    /// Errors encountered
    pub errors: Vec<(PathBuf, String)>,
}

#[allow(dead_code)]
impl InstrumentationResult {
    /// Check if instrumentation was successful
    pub fn is_success(&self) -> bool {
        self.error_count == 0 && self.success_count > 0
    }

    /// Get total files processed
    pub fn total_processed(&self) -> usize {
        self.success_count + self.error_count + self.skipped_count
    }
}

/// AST visitor for instrumentation
struct InstrumentationVisitor {
    config: InstrumentationConfig,
    next_id: usize,
    in_test: bool,
    in_unsafe: bool,
}

impl InstrumentationVisitor {
    fn new(config: InstrumentationConfig) -> Self {
        Self {
            config,
            next_id: 1,
            in_test: false,
            in_unsafe: false,
        }
    }

    #[allow(dead_code)]
    fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    #[allow(dead_code)]
    fn should_instrument(&self) -> bool {
        if self.in_test {
            return false;
        }
        if self.in_unsafe && !self.config.track_unsafe {
            return false;
        }
        true
    }
}

impl VisitMut for InstrumentationVisitor {
    fn visit_file_mut(&mut self, file: &mut syn::File) {
        // Add runtime import at the top
        let import: syn::ItemUse = syn::parse_quote! {
            #[allow(unused_imports)]
            use borrowscope_runtime::{track_new, track_borrow, track_borrow_mut, track_move, track_drop};
        };
        file.items.insert(0, syn::Item::Use(import));

        // Visit all items
        for item in &mut file.items {
            syn::visit_mut::visit_item_mut(self, item);
        }
    }

    fn visit_item_fn_mut(&mut self, func: &mut syn::ItemFn) {
        // Check if this is a test function
        let was_in_test = self.in_test;
        if has_test_attribute(&func.attrs) {
            self.in_test = true;
        }

        // Visit function body
        syn::visit_mut::visit_item_fn_mut(self, func);

        self.in_test = was_in_test;
    }

    fn visit_local_mut(&mut self, local: &mut syn::Local) {
        if self.in_test {
            syn::visit_mut::visit_local_mut(self, local);
            return;
        }

        // Transform: let x = value;
        // Into: let x = track_new(ID, "x", value);
        if let Some(init) = &mut local.init {
            let name = extract_pattern_name(&local.pat);

            let expr = &init.expr;
            let new_init: syn::Expr = syn::parse_quote! {
                borrowscope_runtime::track_new(0, #name, #expr)
            };

            init.expr = Box::new(new_init);
        }

        syn::visit_mut::visit_local_mut(self, local);
    }

    fn visit_expr_mut(&mut self, expr: &mut syn::Expr) {
        if self.in_test {
            syn::visit_mut::visit_expr_mut(self, expr);
            return;
        }

        match expr {
            syn::Expr::Reference(ref_expr) => {
                // Transform: &x or &mut x
                let is_mut = ref_expr.mutability.is_some();

                let inner = &ref_expr.expr;

                if is_mut {
                    *expr = syn::parse_quote! {
                        borrowscope_runtime::track_borrow_mut(0, &#inner)
                    };
                } else {
                    *expr = syn::parse_quote! {
                        borrowscope_runtime::track_borrow(0, &#inner)
                    };
                }
            }
            syn::Expr::Unsafe(_) => {
                let was_in_unsafe = self.in_unsafe;
                self.in_unsafe = true;
                syn::visit_mut::visit_expr_mut(self, expr);
                self.in_unsafe = was_in_unsafe;
            }
            _ => {
                syn::visit_mut::visit_expr_mut(self, expr);
            }
        }
    }
}

#[allow(dead_code)]
fn has_test_attribute(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if let Some(ident) = attr.path().get_ident() {
            ident == "test" || ident == "bench"
        } else {
            attr.path()
                .segments
                .iter()
                .any(|seg| seg.ident == "test" || seg.ident == "tokio" || seg.ident == "async_std")
        }
    })
}

#[allow(dead_code)]
fn extract_pattern_name(pat: &syn::Pat) -> String {
    match pat {
        syn::Pat::Ident(ident) => ident.ident.to_string(),
        syn::Pat::Type(pat_type) => extract_pattern_name(&pat_type.pat),
        syn::Pat::Reference(pat_ref) => extract_pattern_name(&pat_ref.pat),
        syn::Pat::Tuple(pat_tuple) => {
            let names: Vec<String> = pat_tuple.elems.iter().map(extract_pattern_name).collect();
            format!("({})", names.join(", "))
        }
        _ => "<pattern>".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_instrumenter_creation() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("src");
        let output = temp_dir.path().join("output");

        let instrumenter =
            Instrumenter::new(source, output.clone(), InstrumentationConfig::default());
        assert_eq!(instrumenter.output_dir, output);
    }

    #[test]
    fn test_instrument_simple_file() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("src");
        let output_dir = temp_dir.path().join("output");
        fs::create_dir_all(&source_dir).unwrap();

        let source_file = source_dir.join("main.rs");
        fs::write(
            &source_file,
            r#"
fn main() {
    let x = 42;
    let y = &x;
}
"#,
        )
        .unwrap();

        let instrumenter = Instrumenter::new(
            source_dir.clone(),
            output_dir.clone(),
            InstrumentationConfig::default(),
        );
        let result = instrumenter.instrument_file(&source_file).unwrap();

        assert!(result.exists());
        let content = fs::read_to_string(&result).unwrap();
        assert!(content.contains("borrowscope_runtime"));
        assert!(content.contains("track_new"));
        assert!(content.contains("track_borrow"));
    }

    #[test]
    fn test_instrument_project() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("src");
        let output_dir = temp_dir.path().join("output");
        fs::create_dir_all(&source_dir).unwrap();

        // Create multiple files
        fs::write(source_dir.join("main.rs"), "fn main() { let x = 1; }").unwrap();
        fs::write(source_dir.join("lib.rs"), "pub fn foo() { let y = 2; }").unwrap();

        let instrumenter = Instrumenter::new(
            source_dir,
            output_dir.clone(),
            InstrumentationConfig::default(),
        );
        let result = instrumenter.instrument_project().unwrap();

        assert_eq!(result.success_count, 2);
        assert_eq!(result.error_count, 0);
        assert!(result.is_success());
    }

    #[test]
    fn test_skip_test_files() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("src");
        let output_dir = temp_dir.path().join("output");
        fs::create_dir_all(&source_dir).unwrap();

        fs::write(source_dir.join("main.rs"), "fn main() {}").unwrap();
        fs::write(source_dir.join("test_foo.rs"), "fn test() {}").unwrap();

        let instrumenter =
            Instrumenter::new(source_dir, output_dir, InstrumentationConfig::default());
        let result = instrumenter.instrument_project().unwrap();

        assert_eq!(result.success_count, 1);
        assert_eq!(result.skipped_count, 1);
    }

    #[test]
    fn test_pattern_matching() {
        let instrumenter = Instrumenter::new(
            PathBuf::from("/src"),
            PathBuf::from("/out"),
            InstrumentationConfig::default(),
        );

        assert!(instrumenter.matches_pattern("target/debug/main.rs", "target/**"));
        assert!(instrumenter.matches_pattern("src/test_foo.rs", "**/test_*.rs"));
        assert!(!instrumenter.matches_pattern("src/main.rs", "**/test_*.rs"));
    }

    #[test]
    fn test_extract_pattern_name() {
        let pat: syn::Pat = syn::parse_quote! { x };
        assert_eq!(extract_pattern_name(&pat), "x");

        let pat: syn::Pat = syn::parse_quote! { mut x };
        assert_eq!(extract_pattern_name(&pat), "x");
    }

    #[test]
    fn test_has_test_attribute() {
        let attrs: Vec<syn::Attribute> = vec![syn::parse_quote! { #[test] }];
        assert!(has_test_attribute(&attrs));

        let attrs: Vec<syn::Attribute> = vec![syn::parse_quote! { #[derive(Debug)] }];
        assert!(!has_test_attribute(&attrs));
    }

    #[test]
    fn test_instrumentation_config_default() {
        let config = InstrumentationConfig::default();
        assert!(config.track_smart_pointers);
        assert!(config.track_async);
        assert!(!config.track_unsafe);
        assert!(!config.ignore_patterns.is_empty());
    }

    #[test]
    fn test_instrumentation_result_is_success() {
        let mut result = InstrumentationResult::default();
        assert!(!result.is_success());

        result.success_count = 1;
        assert!(result.is_success());

        result.error_count = 1;
        assert!(!result.is_success());
    }

    #[test]
    fn test_instrument_with_references() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("src");
        let output_dir = temp_dir.path().join("output");
        fs::create_dir_all(&source_dir).unwrap();

        let source_file = source_dir.join("main.rs");
        fs::write(
            &source_file,
            r#"
fn main() {
    let x = 42;
    let y = &x;
    let z = &mut x;
}
"#,
        )
        .unwrap();

        let instrumenter =
            Instrumenter::new(source_dir, output_dir, InstrumentationConfig::default());
        let result = instrumenter.instrument_file(&source_file).unwrap();

        let content = fs::read_to_string(&result).unwrap();
        assert!(content.contains("track_borrow"));
        assert!(content.contains("track_borrow_mut"));
    }

    #[test]
    fn test_instrument_preserves_structure() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("src");
        let subdir = source_dir.join("subdir");
        let output_dir = temp_dir.path().join("output");
        fs::create_dir_all(&subdir).unwrap();

        let source_file = subdir.join("module.rs");
        fs::write(&source_file, "pub fn foo() { let x = 1; }").unwrap();

        let instrumenter = Instrumenter::new(
            source_dir,
            output_dir.clone(),
            InstrumentationConfig::default(),
        );
        let result = instrumenter.instrument_file(&source_file).unwrap();

        assert!(result.starts_with(&output_dir));
        assert!(result.to_string_lossy().contains("subdir"));
    }

    #[test]
    fn test_invalid_rust_file() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("src");
        let output_dir = temp_dir.path().join("output");
        fs::create_dir_all(&source_dir).unwrap();

        let source_file = source_dir.join("invalid.rs");
        fs::write(&source_file, "this is not valid rust code {{{").unwrap();

        let instrumenter =
            Instrumenter::new(source_dir, output_dir, InstrumentationConfig::default());
        let result = instrumenter.instrument_file(&source_file);

        assert!(result.is_err());
    }

    #[test]
    fn test_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("src");
        let output_dir = temp_dir.path().join("output");
        fs::create_dir_all(&source_dir).unwrap();

        let source_file = source_dir.join("empty.rs");
        fs::write(&source_file, "").unwrap();

        let instrumenter =
            Instrumenter::new(source_dir, output_dir, InstrumentationConfig::default());
        let result = instrumenter.instrument_file(&source_file);

        assert!(result.is_ok());
    }

    #[test]
    fn test_custom_ignore_patterns() {
        let mut config = InstrumentationConfig::default();
        config.ignore_patterns.push("**/custom_*.rs".into());

        let instrumenter = Instrumenter::new(PathBuf::from("/src"), PathBuf::from("/out"), config);

        assert!(instrumenter.should_ignore(Path::new("/src/custom_file.rs")));
        assert!(!instrumenter.should_ignore(Path::new("/src/normal.rs")));
    }

    #[test]
    fn test_total_processed() {
        let mut result = InstrumentationResult::default();
        result.success_count = 5;
        result.error_count = 2;
        result.skipped_count = 3;

        assert_eq!(result.total_processed(), 10);
    }
}
