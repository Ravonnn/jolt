//! Additional UI corpus runners and property tests.

use std::path::{Path, PathBuf};

use libjolt::jolt_diagnostics::render_diagnostic;
use libjolt::jolt_source::FileId;
use libjolt::Session;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn collect_jolt_files(dir: &Path) -> Vec<PathBuf> {
    if !dir.is_dir() {
        return Vec::new();
    }
    walkdir::WalkDir::new(dir)
        .min_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "jolt"))
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// Check every `.jolt` under `tests/ui/tiny/`.
pub fn check_ui_tiny_corpus() -> Result<(), String> {
    let root = repo_root().join("tests/ui/tiny");
    let mut session = Session::new();
    for path in collect_jolt_files(&root) {
        let src = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let name = path.file_name().unwrap().to_string_lossy();
        let report = session.check_source(FileId(1), &src);
        if !report.is_ok() {
            return Err(format!(
                "{name}: {}",
                report.render_diagnostics(&src).join("\n")
            ));
        }
    }
    Ok(())
}

/// Assert `fmt(fmt(x)) == fmt(x)` for every file under `tests/ui/tiny/`.
pub fn property_fmt_idempotent_tiny() -> Result<(), String> {
    let root = repo_root().join("tests/ui/tiny");
    let mut session = Session::new();
    for path in collect_jolt_files(&root) {
        let src = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let name = path.file_name().unwrap().to_string_lossy();
        let once = session.format_file(FileId(1), &src);
        if !once.is_ok() {
            return Err(format!("{name}: fmt parse errors {:?}", once.errors));
        }
        let twice = session.format_file(FileId(1), &once.source);
        if once.source != twice.source {
            return Err(format!("{name}: fmt not idempotent"));
        }
    }
    Ok(())
}

/// Compare type-reject `.stderr` snapshots under `tests/ui/type-reject/`.
pub fn check_ui_type_reject_corpus() -> Result<(), String> {
    let root = repo_root().join("tests/ui/type-reject");
    let mut session = Session::new();
    for path in collect_jolt_files(&root) {
        let src = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let stem = path.file_stem().unwrap().to_string_lossy();
        let stderr_path = path.with_extension("stderr");
        let report = session.check_source(FileId(1), &src);
        if report.parse.is_ok() && report.resolve.is_ok() && report.check.is_ok() {
            return Err(format!("{stem}: expected type errors"));
        }
        if !report.parse.is_ok() || !report.resolve.is_ok() {
            continue;
        }
        let actual: Vec<String> = report
            .check
            .diagnostics
            .items
            .iter()
            .map(|d| render_diagnostic(d, &src))
            .map(|l| l.trim_end().to_string())
            .collect();
        let expected = std::fs::read_to_string(&stderr_path).map_err(|e| e.to_string())?;
        let expected: Vec<String> = expected
            .lines()
            .map(str::trim_end)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect();
        if actual != expected {
            return Err(format!(
                "{stem}: stderr mismatch\nexpected: {expected:?}\nactual: {actual:?}"
            ));
        }
    }
    Ok(())
}

/// Compare custody-reject `.stderr` snapshots under `tests/custody/should_reject/`.
pub fn check_custody_corpus() -> Result<(), String> {
    let accept = repo_root().join("tests/custody/should_accept");
    let reject = repo_root().join("tests/custody/should_reject");
    let mut session = Session::new();
    for path in collect_jolt_files(&accept) {
        let src = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let name = path.file_name().unwrap().to_string_lossy();
        let report = session.check_source(FileId(1), &src);
        if !report.is_ok() {
            return Err(format!(
                "{name}: {}",
                report.render_diagnostics(&src).join("\n")
            ));
        }
    }
    for path in collect_jolt_files(&reject) {
        let src = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let stem = path.file_stem().unwrap().to_string_lossy();
        let stderr_path = path.with_extension("stderr");
        let report = session.check_source(FileId(1), &src);
        if report.is_ok() {
            return Err(format!("{stem}: expected custody errors"));
        }
        if !report.parse.is_ok() || !report.resolve.is_ok() || !report.check.is_ok() {
            return Err(format!("{stem}: expected custody-only failure"));
        }
        let actual: Vec<String> = report
            .custody
            .diagnostics
            .items
            .iter()
            .map(|d| render_diagnostic(d, &src))
            .map(|l| l.trim_end().to_string())
            .collect();
        if std::env::var("UPDATE_UI").is_ok() {
            let body = actual.join("\n");
            let body = if body.is_empty() {
                body
            } else {
                format!("{body}\n")
            };
            std::fs::write(&stderr_path, body).map_err(|e| e.to_string())?;
            continue;
        }
        let expected = std::fs::read_to_string(&stderr_path).map_err(|e| e.to_string())?;
        let expected: Vec<String> = expected
            .lines()
            .map(str::trim_end)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect();
        if actual != expected {
            return Err(format!(
                "{stem}: stderr mismatch\nexpected: {expected:?}\nactual: {actual:?}"
            ));
        }
    }
    Ok(())
}

/// Run each top-level `tests/run/*.jolt` and compare stdout to `.stdout` snapshots.
pub fn check_run_corpus() -> Result<(), String> {
    check_stdout_corpus(&repo_root().join("tests/run"))
}

/// Run each top-level `tests/tutorial/*.jolt` and compare stdout to `.stdout` snapshots.
pub fn check_tutorial_corpus() -> Result<(), String> {
    check_stdout_corpus(&repo_root().join("tests/tutorial"))
}

fn check_stdout_corpus(root: &Path) -> Result<(), String> {
    let mut session = Session::new();
    let mut ran = 0usize;
    for entry in std::fs::read_dir(root).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().is_some_and(|e| e == "jolt") {
            ran += 1;
            let src = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let stem = path.file_stem().unwrap().to_string_lossy();
            let stdout_path = path.with_extension("stdout");
            let report = session.run_source(FileId(1), &src);
            if !report.is_ok() {
                return Err(format!(
                    "{stem}: {}",
                    report.render_all_errors(&src).join("\n")
                ));
            }
            let actual = report.stdout().unwrap_or("").to_string();
            if std::env::var("UPDATE_UI").is_ok() {
                std::fs::write(&stdout_path, &actual).map_err(|e| e.to_string())?;
                continue;
            }
            let expected = std::fs::read_to_string(&stdout_path).map_err(|e| e.to_string())?;
            if actual != expected {
                return Err(format!(
                    "{stem}: stdout mismatch\nexpected: {expected:?}\nactual: {actual:?}"
                ));
            }
        }
    }
    if ran == 0 {
        return Err(format!("no .jolt files in {}", root.display()));
    }
    Ok(())
}

/// Run each top-level `tests/test/*.jolt` and expect all `[test]` fns to pass.
pub fn check_test_corpus() -> Result<(), String> {
    let root = repo_root().join("tests/test");
    let mut session = Session::new();
    let mut ran = 0usize;
    for entry in std::fs::read_dir(&root).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().is_some_and(|e| e == "jolt") {
            ran += 1;
            let src = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let stem = path.file_stem().unwrap().to_string_lossy();
            let report = session.test_source(FileId(1), &src);
            if !report.is_ok() {
                let mut msg = report.render_all_errors(&src).join("\n");
                for case in &report.report().cases {
                    if !case.passed {
                        msg.push_str(&format!(
                            "\n{}: {}",
                            case.name,
                            case.message.as_deref().unwrap_or("failed")
                        ));
                    }
                }
                return Err(format!("{stem}: {msg}"));
            }
        }
    }
    if ran == 0 {
        return Err("no .jolt files in tests/test/".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod property_tests {
    use super::*;

    #[test]
    fn ui_tiny_corpus_check_clean() {
        check_ui_tiny_corpus().expect("tiny accept corpus");
    }

    #[test]
    fn fmt_idempotent_on_tiny_corpus() {
        property_fmt_idempotent_tiny().expect("fmt idempotent");
    }

    #[test]
    fn ui_type_reject_corpus_snapshots() {
        check_ui_type_reject_corpus().expect("type reject corpus");
    }

    #[test]
    fn custody_corpus_snapshots() {
        check_custody_corpus().expect("custody corpus");
    }

    #[test]
    fn run_corpus_snapshots() {
        check_run_corpus().expect("run corpus");
    }

    #[test]
    fn test_corpus_snapshots() {
        check_test_corpus().expect("test corpus");
    }

    #[test]
    fn tutorial_corpus_snapshots() {
        check_tutorial_corpus().expect("tutorial corpus");
    }
}
