//! Discover and run `[test]` functions (Phase 3b).

mod query;
mod runner;

pub use query::{test_file, TestFileResult, TEST_FILE};
pub use runner::{discover_tests, run_tests, TestCaseResult, TestFn, TestReport};

pub const STAGE: &str = "test-runner";
