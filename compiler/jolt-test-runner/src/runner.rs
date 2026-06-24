use jolt_ast::{Item, Program};
use jolt_backend_interp::interpret_named_fn;
use jolt_mir::MirModule;

/// A discovered `[test]` function.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TestFn {
    pub name: String,
    pub should_fail: bool,
}

/// Outcome of one test function.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TestCaseResult {
    pub name: String,
    pub passed: bool,
    pub should_fail: bool,
    pub message: Option<String>,
}

/// Aggregate test run report.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TestReport {
    pub cases: Vec<TestCaseResult>,
}

impl TestReport {
    pub fn passed_count(&self) -> usize {
        self.cases.iter().filter(|c| c.passed).count()
    }

    pub fn failed_count(&self) -> usize {
        self.cases.iter().filter(|c| !c.passed).count()
    }

    pub fn is_ok(&self) -> bool {
        self.failed_count() == 0 && !self.cases.is_empty()
    }
}

pub fn discover_tests(program: &Program) -> Vec<TestFn> {
    let mut tests = Vec::new();
    for item in &program.items {
        let Item::Fn(f) = item;
        if f.name.name == "main" {
            continue;
        }
        if f.attrs.iter().any(|a| a.name == "test") {
            let should_fail = f.attrs.iter().any(|a| a.name == "should_fail");
            tests.push(TestFn {
                name: f.name.name.clone(),
                should_fail,
            });
        }
    }
    tests
}

pub fn run_tests(module: &MirModule, tests: &[TestFn]) -> TestReport {
    let cases = tests
        .iter()
        .map(|t| {
            let result = interpret_named_fn(module, &t.name, &[]);
            let (passed, message) = if t.should_fail {
                match result {
                    Ok(()) => (false, Some("expected test to fail".to_string())),
                    Err(e) => (true, Some(e.to_string())),
                }
            } else {
                match result {
                    Ok(()) => (true, None),
                    Err(e) => (false, Some(e.to_string())),
                }
            };
            TestCaseResult {
                name: t.name.clone(),
                passed,
                should_fail: t.should_fail,
                message,
            }
        })
        .collect();
    TestReport { cases }
}
