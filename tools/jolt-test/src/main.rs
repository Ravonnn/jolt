use clap::Parser;
use libjolt::jolt_source::FileId;
use libjolt::Session;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "jolt-test", version = libjolt::VERSION, about = "Run Jolt [test] functions")]
struct Cli {
    /// File or directory of `.jolt` sources
    path: PathBuf,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    test_path(&cli.path)
}

fn test_path(path: &Path) -> ExitCode {
    let mut files: Vec<PathBuf> = Vec::new();
    if path.is_dir() {
        collect_jolt_files(path, &mut files);
    } else {
        files.push(path.to_path_buf());
    }
    if files.is_empty() {
        eprintln!("{}: no .jolt files found", path.display());
        return ExitCode::FAILURE;
    }
    files.sort();
    let mut session = Session::new();
    let mut failed = false;
    for file in files {
        let source = match std::fs::read_to_string(&file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}: {e}", file.display());
                failed = true;
                continue;
            }
        };
        let report = session.test_source(FileId(1), &source);
        if !report.is_ok() {
            for line in report.render_all_errors(&source) {
                eprintln!("{}: {line}", file.display());
            }
            for case in &report.report().cases {
                if !case.passed {
                    let msg = case.message.as_deref().unwrap_or("failed");
                    eprintln!("FAILED {}::{} — {msg}", file.display(), case.name);
                }
            }
            failed = true;
            continue;
        }
        for case in &report.report().cases {
            if case.passed {
                if case.should_fail {
                    println!("ok (expected fail) {}::{}", file.display(), case.name);
                } else {
                    println!("ok {}::{}", file.display(), case.name);
                }
            } else {
                failed = true;
                let msg = case.message.as_deref().unwrap_or("failed");
                if case.should_fail {
                    eprintln!(
                        "FAILED (should have failed) {}::{} — {msg}",
                        file.display(),
                        case.name
                    );
                } else {
                    eprintln!("FAILED {}::{} — {msg}", file.display(), case.name);
                }
            }
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
