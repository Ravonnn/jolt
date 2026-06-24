//! Discovers and smoke-runs test corpora under `tests/`.

mod ui;

use std::path::{Path, PathBuf};

pub use ui::{
    check_custody_corpus, check_run_corpus, check_test_corpus, check_tutorial_corpus,
    check_ui_tiny_corpus, check_ui_type_reject_corpus, property_fmt_idempotent_tiny,
};

const UI_SAMPLE: &str = "tests/ui/sample";
const RUN_SAMPLE: &str = "tests/run/sample";
const RUN_CORPUS: &str = "tests/run";
const TEST_CORPUS: &str = "tests/test";
const TUTORIAL_CORPUS: &str = "tests/tutorial";
const CUSTODY_ACCEPT: &str = "tests/custody/should_accept";
const CUSTODY_REJECT: &str = "tests/custody/should_reject";
const CUSTODY_SAMPLE: &str = "tests/custody/sample/should_accept";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn count_jolt_files(dir: &Path) -> usize {
    if !dir.is_dir() {
        return 0;
    }
    walkdir::WalkDir::new(dir)
        .min_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "jolt"))
        .count()
}

pub fn ui_sample_count() -> usize {
    count_jolt_files(&repo_root().join(UI_SAMPLE))
}

pub fn run_corpus_count() -> usize {
    count_top_level_jolt_files(&repo_root().join(RUN_CORPUS))
}

pub fn test_corpus_count() -> usize {
    count_top_level_jolt_files(&repo_root().join(TEST_CORPUS))
}

pub fn tutorial_corpus_count() -> usize {
    count_top_level_jolt_files(&repo_root().join(TUTORIAL_CORPUS))
}

fn count_top_level_jolt_files(dir: &Path) -> usize {
    if !dir.is_dir() {
        return 0;
    }
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "jolt"))
        .count()
}

pub fn run_sample_count() -> usize {
    count_jolt_files(&repo_root().join(RUN_SAMPLE))
}

pub fn custody_sample_count() -> usize {
    count_jolt_files(&repo_root().join(CUSTODY_SAMPLE))
}

pub fn custody_accept_count() -> usize {
    count_jolt_files(&repo_root().join(CUSTODY_ACCEPT))
}

pub fn custody_reject_count() -> usize {
    count_jolt_files(&repo_root().join(CUSTODY_REJECT))
}

pub fn run_all_smoke() -> Result<(), String> {
    let ui = ui_sample_count();
    let run = run_sample_count();
    let custody = custody_sample_count();
    let custody_accept = custody_accept_count();
    if ui == 0 {
        return Err(format!("no .jolt files under {UI_SAMPLE}"));
    }
    if run == 0 {
        return Err(format!("no .jolt files under {RUN_SAMPLE}"));
    }
    if custody == 0 {
        return Err(format!("no .jolt files under {CUSTODY_SAMPLE}"));
    }
    if custody_accept < 10 {
        return Err(format!(
            "expected at least 10 .jolt files under {CUSTODY_ACCEPT}, found {custody_accept}"
        ));
    }
    let custody_reject = custody_reject_count();
    let run_corpus = run_corpus_count();
    if custody_reject < 8 {
        return Err(format!(
            "expected at least 8 .jolt files under {CUSTODY_REJECT}, found {custody_reject}"
        ));
    }
    if run_corpus < 5 {
        return Err(format!(
            "expected at least 5 .jolt files under {RUN_CORPUS}, found {run_corpus}"
        ));
    }
    let test_corpus = test_corpus_count();
    if test_corpus < 2 {
        return Err(format!(
            "expected at least 2 .jolt files under {TEST_CORPUS}, found {test_corpus}"
        ));
    }
    let tutorial_corpus = tutorial_corpus_count();
    if tutorial_corpus < 8 {
        return Err(format!(
            "expected at least 8 .jolt files under {TUTORIAL_CORPUS}, found {tutorial_corpus}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_sample_exists() {
        assert!(ui_sample_count() >= 1);
    }

    #[test]
    fn run_sample_exists() {
        assert!(run_sample_count() >= 1);
    }

    #[test]
    fn custody_sample_exists() {
        assert!(custody_sample_count() >= 1);
    }

    #[test]
    fn custody_accept_corpus_size() {
        assert!(custody_accept_count() >= 10);
    }

    #[test]
    fn custody_reject_corpus_size() {
        assert!(custody_reject_count() >= 8);
    }

    #[test]
    fn run_corpus_size() {
        assert!(run_corpus_count() >= 5);
    }

    #[test]
    fn test_corpus_size() {
        assert!(test_corpus_count() >= 2);
    }

    #[test]
    fn tutorial_corpus_size() {
        assert!(tutorial_corpus_count() >= 8);
    }

    #[test]
    fn all_corpora_smoke() {
        run_all_smoke().expect("corpus smoke");
    }
}
