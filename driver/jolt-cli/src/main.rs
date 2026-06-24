use clap::{Parser, Subcommand};
use libjolt::jolt_diagnostics::{render_at, Severity};
use libjolt::jolt_source::FileId;
use libjolt::{CheckReport, RunReport, Session, TestSourceReport};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "jolt", version = libjolt::VERSION, about = "The Jolt compiler driver")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Type-check `.jolt` source file(s) or a directory tree
    Check {
        /// File or directory to check
        path: PathBuf,
    },
    /// Run a program (interpreted when `--interpret` is set)
    Run {
        /// Path to the source file
        path: PathBuf,
        /// Execute via the MIR interpreter
        #[arg(long)]
        interpret: bool,
    },
    /// Format a `.jolt` source file
    Fmt {
        /// Path to the source file
        path: PathBuf,
        /// Write formatted output back to the file
        #[arg(long)]
        write: bool,
    },
    /// Discover and run `[test]` functions in `.jolt` file(s)
    Test {
        /// File or directory to test
        path: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Check { path }) => check_path(&path),
        Some(Commands::Run { path, interpret }) => run_file(&path, interpret),
        Some(Commands::Fmt { path, write }) => fmt_file(&path, write),
        Some(Commands::Test { path }) => test_path(&path),
        None => ExitCode::SUCCESS,
    }
}

fn stub(name: &str) -> ExitCode {
    eprintln!("not implemented yet: {name}");
    ExitCode::from(2)
}

fn check_path(path: &Path) -> ExitCode {
    if path.is_dir() {
        check_directory(path)
    } else {
        check_single_file(path)
    }
}

fn check_single_file(path: &Path) -> ExitCode {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}: {e}", path.display());
            return ExitCode::FAILURE;
        }
    };
    let mut session = Session::new();
    let report = session.check_source(FileId(1), &source);
    emit_report(path, &source, &report)
}

fn check_directory(dir: &Path) -> ExitCode {
    let mut files: Vec<PathBuf> = Vec::new();
    collect_jolt_files(dir, &mut files);
    if files.is_empty() {
        eprintln!("{}: no .jolt files found", dir.display());
        return ExitCode::FAILURE;
    }
    files.sort();
    let mut session = Session::new();
    let mut failed = false;
    for path in files {
        let source = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}: {e}", path.display());
                failed = true;
                continue;
            }
        };
        let report = session.check_source(FileId(1), &source);
        if emit_report(&path, &source, &report) != ExitCode::SUCCESS {
            failed = true;
        }
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn collect_jolt_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_jolt_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "jolt") {
            out.push(path);
        }
    }
}

fn emit_report(path: &Path, source: &str, report: &CheckReport) -> ExitCode {
    if report.is_ok() {
        return ExitCode::SUCCESS;
    }
    for line in report.render_diagnostics(source) {
        eprintln!("{}: {line}", path.display());
    }
    ExitCode::FAILURE
}

fn run_file(path: &Path, interpret: bool) -> ExitCode {
    if !interpret {
        return stub("run (pass --interpret to execute)");
    }
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}: {e}", path.display());
            return ExitCode::FAILURE;
        }
    };
    let mut session = Session::new();
    let report = session.run_source(FileId(1), &source);
    emit_run_report(path, &source, &report)
}

fn emit_run_report(path: &Path, source: &str, report: &RunReport) -> ExitCode {
    if !report.is_ok() {
        for line in report.render_all_errors(source) {
            eprintln!("{}: {line}", path.display());
        }
        return ExitCode::FAILURE;
    }
    if let Some(out) = report.stdout() {
        print!("{out}");
    }
    ExitCode::SUCCESS
}

fn test_path(path: &Path) -> ExitCode {
    if path.is_dir() {
        test_directory(path)
    } else {
        test_single_file(path)
    }
}

fn test_single_file(path: &Path) -> ExitCode {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}: {e}", path.display());
            return ExitCode::FAILURE;
        }
    };
    let mut session = Session::new();
    let report = session.test_source(FileId(1), &source);
    emit_test_report(path, &source, &report)
}

fn test_directory(dir: &Path) -> ExitCode {
    let mut files: Vec<PathBuf> = Vec::new();
    collect_jolt_files(dir, &mut files);
    if files.is_empty() {
        eprintln!("{}: no .jolt files found", dir.display());
        return ExitCode::FAILURE;
    }
    files.sort();
    let mut session = Session::new();
    let mut failed = false;
    for path in files {
        let source = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}: {e}", path.display());
                failed = true;
                continue;
            }
        };
        let report = session.test_source(FileId(1), &source);
        if emit_test_report(&path, &source, &report) != ExitCode::SUCCESS {
            failed = true;
        }
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn emit_test_report(path: &Path, source: &str, report: &TestSourceReport) -> ExitCode {
    if !report.parse.is_ok()
        || !report.resolve.is_ok()
        || !report.check.diagnostics.is_empty()
        || !report.custody.is_ok()
        || !report.mir.is_ok()
    {
        for line in report.render_all_errors(source) {
            eprintln!("{}: {line}", path.display());
        }
        return ExitCode::FAILURE;
    }
    if report.report().cases.is_empty() {
        eprintln!("{}: no [test] functions found", path.display());
        return ExitCode::FAILURE;
    }
    let mut failed = false;
    for case in &report.report().cases {
        if case.passed {
            if case.should_fail {
                println!("ok (expected fail) {}::{}", path.display(), case.name);
            } else {
                println!("ok {}::{}", path.display(), case.name);
            }
        } else {
            failed = true;
            let msg = case.message.as_deref().unwrap_or("failed");
            if case.should_fail {
                eprintln!(
                    "FAILED (should have failed) {}::{} — {msg}",
                    path.display(),
                    case.name
                );
            } else {
                eprintln!("FAILED {}::{} — {msg}", path.display(), case.name);
            }
        }
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn fmt_file(path: &PathBuf, write: bool) -> ExitCode {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}: {e}", path.display());
            return ExitCode::FAILURE;
        }
    };
    let file = FileId(1);
    let mut session = Session::new();
    let result = session.format_file(file, &source);
    if !result.is_ok() {
        for err in &result.errors {
            eprintln!(
                "{}",
                render_at(Severity::Error, err.span.start, &source, &err.message, None)
            );
        }
        return ExitCode::FAILURE;
    }
    if write {
        if let Err(e) = std::fs::write(path, &result.source) {
            eprintln!("{}: {e}", path.display());
            return ExitCode::FAILURE;
        }
    } else {
        print!("{}", result.source);
    }
    ExitCode::SUCCESS
}
