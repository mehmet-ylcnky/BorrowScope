# Chapter 9 Quick Reference

## Section Overview

| # | Title | Lines | Focus |
|---|-------|-------|-------|
| 106 | Plugin System Architecture | 500+ | Design, lifecycle, registry |
| 107 | Custom Analysis Plugins | 500+ | Analyzers, patterns, findings |
| 108 | Visualization Plugins | 500+ | Renderers, 3D, heatmaps |
| 109 | Export Format Plugins | 500+ | GraphML, CSV, SQLite |
| 110 | Plugin Discovery/Loading | 500+ | Dynamic loading, dependencies |
| 111 | Plugin API Design | 500+ | Stable APIs, versioning, SDK |
| 112 | Plugin Sandboxing | 500+ | Isolation, limits, permissions |
| 113 | Plugin Configuration | 500+ | Schema, validation, UI |
| 114 | Plugin Testing | 500+ | Test harness, mocks, benchmarks |
| 115 | Macro-Based Analysis | 500+ | Expansion tracking, hygiene |
| 116 | Proc Macro Integration | 500+ | Transformations, debugging |
| 117 | Custom Derive Macros | 500+ | Code generation, generics |
| 118 | Attribute Macros | 500+ | Function instrumentation, async |
| 119 | Compiler Integration | 500+ | rustc hooks, phases |
| 120 | MIR Analysis | 500+ | Mid-level IR, flow analysis |
| 121 | HIR Analysis | 500+ | High-level IR, patterns |
| 122 | Type System Integration | 500+ | Type queries, generics |
| 123 | Trait Resolution | 500+ | Trait impls, bounds |
| 124 | Lifetime Visualization | 500+ | Diagrams, relationships |
| 125 | Borrow Checker Integration | 500+ | Conflicts, suggestions |

## Key Traits

```rust
// Core plugin trait
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn initialize(&mut self, context: &PluginContext) -> Result<()>;
    fn shutdown(&mut self) -> Result<()>;
}

// Analysis plugin
pub trait AnalysisPlugin: Plugin {
    fn analyze(&self, graph: &OwnershipGraph) -> Result<AnalysisResult>;
}

// Visualization plugin
pub trait VisualizationPlugin: Plugin {
    fn render(&self, graph: &OwnershipGraph, options: &RenderOptions) -> Result<RenderOutput>;
}

// Export plugin
pub trait ExportPlugin: Plugin {
    fn export(&self, graph: &OwnershipGraph, writer: &mut dyn Write) -> Result<()>;
}
```

## Key Structures

```rust
// Plugin registry
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn Plugin>>,
    hooks: HashMap<HookPoint, Vec<String>>,
}

// Plugin context
pub struct PluginContext {
    graph_api: Arc<GraphApi>,
    config: PluginConfig,
    logger: Logger,
}

// Analysis result
pub struct AnalysisResult {
    findings: Vec<Finding>,
}

// Finding
pub struct Finding {
    severity: Severity,
    message: String,
    location: SourceLocation,
}
```

## Common Patterns

### Plugin Registration
```rust
let mut registry = PluginRegistry::new();
registry.register(Box::new(MyPlugin::new()))?;
```

### Plugin Execution
```rust
let plugin = registry.get("my-plugin")?;
let result = plugin.analyze(&graph)?;
```

### Hook Registration
```rust
registry.register_hook(HookPoint::AfterAnalysis, "my-plugin")?;
```

## Compiler Integration

### MIR Access
```rust
let body = tcx.optimized_mir(def_id);
for (bb, data) in body.basic_blocks().iter_enumerated() {
    // Analyze basic block
}
```

### HIR Visitor
```rust
impl<'tcx> Visitor<'tcx> for MyVisitor {
    fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
        // Process expression
    }
}
```

### Type Queries
```rust
let ty = tcx.type_of(def_id);
let implements_copy = ty.is_copy_modulo_regions(tcx, param_env);
```

## Testing Patterns

### Plugin Test
```rust
#[test]
fn test_plugin_lifecycle() {
    let mut plugin = MyPlugin::new();
    let context = MockContext::new();
    
    plugin.initialize(&context).unwrap();
    assert!(plugin.is_initialized());
    
    plugin.shutdown().unwrap();
}
```

### Analysis Test
```rust
#[test]
fn test_analysis() {
    let graph = create_test_graph();
    let analyzer = MyAnalyzer::new();
    
    let result = analyzer.analyze(&graph).unwrap();
    assert_eq!(result.findings.len(), 2);
}
```

## Best Practices

1. **Plugin Design**
   - Keep plugins focused and single-purpose
   - Use stable APIs
   - Version plugins properly
   - Document thoroughly

2. **Performance**
   - Cache expensive computations
   - Use incremental analysis when possible
   - Implement resource limits
   - Profile regularly

3. **Security**
   - Sandbox untrusted plugins
   - Validate all inputs
   - Limit resource usage
   - Use permission system

4. **Testing**
   - Test plugin lifecycle
   - Mock external dependencies
   - Test error paths
   - Benchmark performance

## Common Issues

### Plugin Loading Fails
- Check plugin compatibility
- Verify dependencies
- Check file permissions
- Review error logs

### Analysis Slow
- Enable incremental mode
- Implement caching
- Optimize algorithms
- Profile bottlenecks

### Compiler Integration Issues
- Use correct rustc version
- Check feature flags
- Verify API usage
- Review compiler docs

## Resources

- Rust Compiler Internals: https://rustc-dev-guide.rust-lang.org/
- Plugin Architecture Patterns: https://www.patterns.dev/
- MIR Documentation: https://rustc-dev-guide.rust-lang.org/mir/
- HIR Documentation: https://rustc-dev-guide.rust-lang.org/hir.html
