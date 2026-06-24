use clap::Parser;
use libjolt::jolt_diagnostics::{render_at, Severity};
use libjolt::jolt_source::FileId;
use libjolt::Session;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "jolt-fmt", about = "Format Jolt source files")]
struct Cli {
    /// Path to the source file
    path: PathBuf,
    /// Write formatted output back to the file
    #[arg(long)]
    write: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let source = match std::fs::read_to_string(&cli.path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}: {e}", cli.path.display());
            return ExitCode::FAILURE;
        }
    };
    let mut session = Session::new();
    let result = session.format_file(FileId(1), &source);
    if !result.is_ok() {
        for err in &result.errors {
            eprintln!(
                "{}",
                render_at(Severity::Error, err.span.start, &source, &err.message, None)
            );
        }
        return ExitCode::FAILURE;
    }
    if cli.write {
        if let Err(e) = std::fs::write(&cli.path, &result.source) {
            eprintln!("{}: {e}", cli.path.display());
            return ExitCode::FAILURE;
        }
    } else {
        print!("{}", result.source);
    }
    ExitCode::SUCCESS
}
