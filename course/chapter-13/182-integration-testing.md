# Section 182: Integration Testing

## Learning Objectives

By the end of this section, you will:
- Master integration testing techniques for BorrowScope
- Implement comprehensive test suites
- Ensure code quality and reliability
- Automate testing workflows
- Maintain high test coverage

## Prerequisites

- Completed Chapters 1-12
- Understanding of testing principles
- Familiarity with Rust testing frameworks
- Knowledge of CI/CD pipelines
- Experience with quality assurance

## Introduction

Integration Testing is essential for maintaining code quality and reliability in BorrowScope. This section covers comprehensive testing strategies, automation, and best practices for ensuring production-ready software.

Quality assurance through systematic testing prevents bugs, ensures correctness, and provides confidence in code changes. This section provides practical implementations and strategies for building robust test suites.

## Core Concepts

### Testing Fundamentals

Key testing principles:

1. **Correctness**: Code behaves as expected
2. **Coverage**: All code paths tested
3. **Maintainability**: Tests are easy to update
4. **Speed**: Fast feedback loops
5. **Reliability**: Tests are deterministic

### Testing Architecture

```rust
/// Test framework
pub struct TestFramework {
    suites: Vec<TestSuite>,
    config: TestConfig,
    reporter: TestReporter,
}

impl TestFramework {
    pub fn new(config: TestConfig) -> Self {
        Self {
            suites: Vec::new(),
            config,
            reporter: TestReporter::new(),
        }
    }
    
    pub fn add_suite(&mut self, suite: TestSuite) {
        self.suites.push(suite);
    }
    
    pub fn run_all(&mut self) -> TestResults {
        let mut results = TestResults::new();
        
        for suite in &self.suites {
            let suite_results = self.run_suite(suite);
            results.add_suite_results(suite.name.clone(), suite_results);
        }
        
        results
    }
    
    fn run_suite(&self, suite: &TestSuite) -> SuiteResults {
        let mut results = SuiteResults::new();
        
        for test in &suite.tests {
            let result = self.run_test(test);
            results.add_test_result(test.name.clone(), result);
        }
        
        results
    }
    
    fn run_test(&self, test: &Test) -> TestResult {
        let start = std::time::Instant::now();
        
        let outcome = std::panic::catch_unwind(|| {
            (test.function)();
        });
        
        let duration = start.elapsed();
        
        match outcome {
            Ok(_) => TestResult::Passed { duration },
            Err(e) => TestResult::Failed {
                duration,
                error: format!("{:?}", e),
            },
        }
    }
}

/// Test suite
pub struct TestSuite {
    pub name: String,
    pub tests: Vec<Test>,
}

impl TestSuite {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tests: Vec::new(),
        }
    }
    
    pub fn add_test(&mut self, name: impl Into<String>, function: TestFn) {
        self.tests.push(Test {
            name: name.into(),
            function,
        });
    }
}

/// Individual test
pub struct Test {
    pub name: String,
    pub function: TestFn,
}

type TestFn = fn();

/// Test result
#[derive(Debug)]
pub enum TestResult {
    Passed { duration: std::time::Duration },
    Failed { duration: std::time::Duration, error: String },
    Skipped,
}

/// Suite results
pub struct SuiteResults {
    test_results: HashMap<String, TestResult>,
}

impl SuiteResults {
    pub fn new() -> Self {
        Self {
            test_results: HashMap::new(),
        }
    }
    
    pub fn add_test_result(&mut self, name: String, result: TestResult) {
        self.test_results.insert(name, result);
    }
    
    pub fn passed_count(&self) -> usize {
        self.test_results.values()
            .filter(|r| matches!(r, TestResult::Passed { .. }))
            .count()
    }
    
    pub fn failed_count(&self) -> usize {
        self.test_results.values()
            .filter(|r| matches!(r, TestResult::Failed { .. }))
            .count()
    }
}

/// Overall test results
pub struct TestResults {
    suite_results: HashMap<String, SuiteResults>,
}

impl TestResults {
    pub fn new() -> Self {
        Self {
            suite_results: HashMap::new(),
        }
    }
    
    pub fn add_suite_results(&mut self, name: String, results: SuiteResults) {
        self.suite_results.insert(name, results);
    }
    
    pub fn total_passed(&self) -> usize {
        self.suite_results.values()
            .map(|s| s.passed_count())
            .sum()
    }
    
    pub fn total_failed(&self) -> usize {
        self.suite_results.values()
            .map(|s| s.failed_count())
            .sum()
    }
    
    pub fn success_rate(&self) -> f64 {
        let total = self.total_passed() + self.total_failed();
        if total == 0 {
            return 0.0;
        }
        self.total_passed() as f64 / total as f64
    }
}

/// Test configuration
pub struct TestConfig {
    pub parallel: bool,
    pub timeout: std::time::Duration,
    pub verbose: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            parallel: true,
            timeout: std::time::Duration::from_secs(60),
            verbose: false,
        }
    }
}

/// Test reporter
pub struct TestReporter {
    format: ReportFormat,
}

impl TestReporter {
    pub fn new() -> Self {
        Self {
            format: ReportFormat::Text,
        }
    }
    
    pub fn report(&self, results: &TestResults) -> String {
        match self.format {
            ReportFormat::Text => self.text_report(results),
            ReportFormat::Json => self.json_report(results),
            ReportFormat::Xml => self.xml_report(results),
        }
    }
    
    fn text_report(&self, results: &TestResults) -> String {
        format!(
            "Test Results:\n  Passed: {}\n  Failed: {}\n  Success Rate: {:.1}%\n",
            results.total_passed(),
            results.total_failed(),
            results.success_rate() * 100.0
        )
    }
    
    fn json_report(&self, results: &TestResults) -> String {
        format!(
            "{\"passed\": {}, \"failed\": {}, \"success_rate\": {}}",
            results.total_passed(),
            results.total_failed(),
            results.success_rate()
        )
    }
    
    fn xml_report(&self, results: &TestResults) -> String {
        format!(
            "<results><passed>{}</passed><failed>{}</failed></results>",
            results.total_passed(),
            results.total_failed()
        )
    }
}

pub enum ReportFormat {
    Text,
    Json,
    Xml,
}
```

## Implementation

### Test Utilities

```rust
/// Test utilities
pub mod test_utils {
    use super::*;
    
    /// Create test graph
    pub fn create_test_graph() -> OwnershipGraph {
        let mut graph = OwnershipGraph::new();
        
        graph.add_variable(Variable {
            id: "v1".to_string(),
            name: "x".to_string(),
            type_name: "i32".to_string(),
            created_at: 0,
            dropped_at: Some(10),
            scope_depth: 0,
        });
        
        graph.add_variable(Variable {
            id: "v2".to_string(),
            name: "y".to_string(),
            type_name: "i32".to_string(),
            created_at: 5,
            dropped_at: Some(15),
            scope_depth: 0,
        });
        
        graph.add_relationship(Relationship {
            source: "v1".to_string(),
            target: "v2".to_string(),
            relationship_type: RelationshipType::BorrowsImmut,
            timestamp: 7,
        });
        
        graph
    }
    
    /// Assert graph properties
    pub fn assert_graph_valid(graph: &OwnershipGraph) {
        assert!(graph.variable_count() > 0, "Graph should have variables");
        
        for rel in graph.relationships() {
            assert!(
                graph.has_variable(&rel.source),
                "Source variable should exist"
            );
            assert!(
                graph.has_variable(&rel.target),
                "Target variable should exist"
            );
        }
    }
    
    /// Create mock context
    pub fn create_mock_context() -> Context {
        Context {
            config: Config::default(),
            logger: Logger::new(),
        }
    }
}
```

### Assertion Helpers

```rust
/// Custom assertions
pub mod assertions {
    /// Assert approximately equal for floats
    pub fn assert_approx_eq(a: f64, b: f64, epsilon: f64) {
        assert!(
            (a - b).abs() < epsilon,
            "Values not approximately equal: {} vs {}",
            a, b
        );
    }
    
    /// Assert collection contains
    pub fn assert_contains<T: PartialEq>(collection: &[T], item: &T) {
        assert!(
            collection.contains(item),
            "Collection does not contain item"
        );
    }
    
    /// Assert eventually true (for async tests)
    pub async fn assert_eventually<F>(mut condition: F, timeout: std::time::Duration)
    where
        F: FnMut() -> bool,
    {
        let start = std::time::Instant::now();
        
        while !condition() {
            if start.elapsed() > timeout {
                panic!("Condition not met within timeout");
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }
}
```

### Test Fixtures

```rust
/// Test fixtures
pub struct TestFixture {
    temp_dir: tempfile::TempDir,
    test_files: Vec<PathBuf>,
}

impl TestFixture {
    pub fn new() -> Result<Self, std::io::Error> {
        let temp_dir = tempfile::tempdir()?;
        
        Ok(Self {
            temp_dir,
            test_files: Vec::new(),
        })
    }
    
    pub fn create_test_file(&mut self, name: &str, content: &str) -> Result<PathBuf, std::io::Error> {
        let path = self.temp_dir.path().join(name);
        std::fs::write(&path, content)?;
        self.test_files.push(path.clone());
        Ok(path)
    }
    
    pub fn temp_dir(&self) -> &Path {
        self.temp_dir.path()
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        // Cleanup happens automatically with TempDir
    }
}
```

## Example Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_functionality() {
        let graph = test_utils::create_test_graph();
        assert_eq!(graph.variable_count(), 2);
    }
    
    #[test]
    fn test_graph_validation() {
        let graph = test_utils::create_test_graph();
        test_utils::assert_graph_valid(&graph);
    }
    
    #[test]
    fn test_relationship_creation() {
        let mut graph = OwnershipGraph::new();
        
        graph.add_variable(Variable {
            id: "v1".to_string(),
            name: "x".to_string(),
            type_name: "i32".to_string(),
            created_at: 0,
            dropped_at: None,
            scope_depth: 0,
        });
        
        assert_eq!(graph.variable_count(), 1);
    }
    
    #[test]
    #[should_panic(expected = "Invalid variable")]
    fn test_error_handling() {
        let graph = OwnershipGraph::new();
        graph.get_variable("nonexistent").expect("Invalid variable");
    }
    
    #[tokio::test]
    async fn test_async_operation() {
        let result = async_operation().await;
        assert!(result.is_ok());
    }
}
```

## Integration Tests

```rust
// tests/integration_test.rs
use borrowscope::*;

#[test]
fn test_full_workflow() {
    // Setup
    let mut analyzer = Analyzer::new();
    
    // Load test data
    let graph = load_test_graph();
    
    // Analyze
    let results = analyzer.analyze(&graph).unwrap();
    
    // Verify
    assert!(results.conflicts.is_empty());
    assert!(results.statistics.total_variables > 0);
}

#[test]
fn test_cli_integration() {
    // Test CLI commands
    let output = std::process::Command::new("borrowscope")
        .arg("analyze")
        .arg("test_file.rs")
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
}
```

## Best Practices

1. **Test Pyramid**: More unit tests, fewer integration tests
2. **Fast Tests**: Keep tests fast for quick feedback
3. **Isolated Tests**: Tests should not depend on each other
4. **Clear Names**: Test names should describe what they test
5. **Arrange-Act-Assert**: Structure tests clearly

## Common Pitfalls

1. **Flaky Tests**: Avoid non-deterministic behavior
2. **Slow Tests**: Optimize test execution time
3. **Brittle Tests**: Don't test implementation details
4. **Poor Coverage**: Ensure critical paths are tested
5. **No Cleanup**: Always clean up test resources

## Key Takeaways

- Integration Testing ensures code quality and reliability
- Comprehensive test suites catch bugs early
- Automated testing provides confidence in changes
- Fast feedback loops improve productivity
- Continuous testing maintains quality

## Further Reading

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Test-Driven Development](https://martinfowler.com/bliki/TestDrivenDevelopment.html)
- [Testing Best Practices](https://github.com/rust-lang/rfcs/blob/master/text/1105-api-evolution.md)
- [Continuous Integration](https://martinfowler.com/articles/continuousIntegration.html)

## Summary

This section covered integration testing with comprehensive examples, best practices, and strategies for building robust test suites that ensure BorrowScope's quality and reliability.
