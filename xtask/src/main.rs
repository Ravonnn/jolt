mod learn;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};
use std::thread;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "xtask")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Smoke-run ui/run/custody sample corpora
    TestCorpora,
    /// End-to-end `jolt-cli` pipeline smoke (parse → custody → run/test)
    PipelineSmoke,
    /// Interactive learn platform
    Learn {
        #[command(subcommand)]
        command: LearnCommands,
    },
    /// Build or serve the mdBook documentation in `docs/`
    Docs {
        #[command(subcommand)]
        command: DocsCommands,
    },
}

#[derive(Subcommand)]
enum LearnCommands {
    /// Sync learn/ → docs/learn/, verify manifest, build jolt-cli + runner
    Verify,
    /// Start mdBook + jolt-learn-runner (local code execution)
    Serve {
        /// Open browser after starting
        #[arg(long)]
        open: bool,
    },
}

#[derive(Subcommand)]
enum DocsCommands {
    /// Run `mdbook build` (output: `docs/book/`)
    Build,
    /// Run `mdbook serve` for local preview
    Serve {
        /// Open the browser after starting the server
        #[arg(long)]
        open: bool,
    },
}

fn docs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn jolt_bin() -> PathBuf {
    repo_root().join("target/debug/jolt")
}

fn run_mdbook(subcommand: &str, extra: &[&str]) -> ExitCode {
    let repo = repo_root();
    if let Err(e) = learn::sync_learn_to_docs(&repo) {
        eprintln!("learn sync failed: {e}");
        return ExitCode::FAILURE;
    }

    let docs = docs_dir();
    if !docs.join("book.toml").is_file() {
        eprintln!("docs/book.toml not found at {}", docs.display());
        return ExitCode::FAILURE;
    }

    let status = Command::new("mdbook")
        .arg(subcommand)
        .arg(&docs)
        .args(extra)
        .status();

    match status {
        Ok(s) if s.success() => ExitCode::SUCCESS,
        Ok(s) => ExitCode::from(s.code().unwrap_or(1) as u8),
        Err(e) => {
            eprintln!(
                "failed to run mdbook: {e}\n\
                 Install with: cargo install mdbook --locked"
            );
            ExitCode::FAILURE
        }
    }
}

fn run_command(label: &str, mut cmd: Command) -> Result<(), String> {
    let status = cmd
        .status()
        .map_err(|e| format!("{label}: failed to spawn: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{label}: exited with {status}"))
    }
}

fn pipeline_smoke() -> ExitCode {
    let root = repo_root();
    let jolt = jolt_bin();

    let steps: [(&str, Command); 5] = [
        ("cargo build -p jolt-cli", {
            let mut cmd = Command::new("cargo");
            cmd.args(["build", "-p", "jolt-cli"]).current_dir(&root);
            cmd
        }),
        ("jolt run --interpret tests/run/hello.jolt", {
            let mut cmd = Command::new(&jolt);
            cmd.args(["run", "--interpret", "tests/run/hello.jolt"])
                .current_dir(&root);
            cmd
        }),
        ("jolt run --interpret tests/tutorial/hello.jolt", {
            let mut cmd = Command::new(&jolt);
            cmd.args(["run", "--interpret", "tests/tutorial/hello.jolt"])
                .current_dir(&root);
            cmd
        }),
        ("jolt test tests/test/", {
            let mut cmd = Command::new(&jolt);
            cmd.args(["test", "tests/test/"]).current_dir(&root);
            cmd
        }),
        ("jolt check tests/run/", {
            let mut cmd = Command::new(&jolt);
            cmd.args(["check", "tests/run/"]).current_dir(&root);
            cmd
        }),
    ];

    for (label, cmd) in steps {
        if let Err(e) = run_command(label, cmd) {
            eprintln!("pipeline-smoke failed: {e}");
            return ExitCode::FAILURE;
        }
    }

    println!("pipeline-smoke: ok");
    ExitCode::SUCCESS
}

fn learn_verify() -> ExitCode {
    let repo = repo_root();
    if let Err(e) = learn::verify_curriculum(&repo) {
        eprintln!("learn verify failed: {e}");
        return ExitCode::FAILURE;
    }
    if let Err(e) = learn::sync_learn_to_docs(&repo) {
        eprintln!("learn sync failed: {e}");
        return ExitCode::FAILURE;
    }
    println!("learn verify: ok");
    ExitCode::SUCCESS
}

fn learn_serve(open: bool) -> ExitCode {
    let repo = repo_root();
    if let Err(e) = learn::sync_learn_to_docs(&repo) {
        eprintln!("learn sync failed: {e}");
        return ExitCode::FAILURE;
    }

    let build = Command::new("cargo")
        .args(["build", "-p", "jolt-cli", "-p", "jolt-learn-runner"])
        .current_dir(&repo)
        .status();
    if build.as_ref().is_err() || !build.unwrap().success() {
        eprintln!("failed to build jolt-cli and jolt-learn-runner");
        return ExitCode::FAILURE;
    }

    let runner_bin = repo.join("target/debug/jolt-learn-runner");
    let mut runner = Command::new(&runner_bin);
    runner
        .current_dir(&repo)
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    let runner_child = runner.spawn();
    match runner_child {
        Ok(mut child) => {
            thread::sleep(Duration::from_millis(300));
            let code = run_mdbook(
                "serve",
                if open {
                    &["--port", "3000", "--open"]
                } else {
                    &["--port", "3000"]
                },
            );
            let _ = child.kill();
            let _ = child.wait();
            code
        }
        Err(e) => {
            eprintln!("failed to start jolt-learn-runner: {e}");
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Commands::TestCorpora => match jolt_harness::run_all_smoke() {
            Ok(()) => {
                println!("corpus smoke: ok");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("corpus smoke failed: {e}");
                ExitCode::FAILURE
            }
        },
        Commands::PipelineSmoke => pipeline_smoke(),
        Commands::Learn { command } => match command {
            LearnCommands::Verify => learn_verify(),
            LearnCommands::Serve { open } => learn_serve(open),
        },
        Commands::Docs { command } => match command {
            DocsCommands::Build => {
                let code = run_mdbook("build", &[]);
                if code == ExitCode::SUCCESS {
                    println!("docs built: {}", docs_dir().join("book").display());
                }
                code
            }
            DocsCommands::Serve { open } => {
                let mut extra = vec!["--port", "3000"];
                if open {
                    extra.push("--open");
                }
                run_mdbook("serve", &extra)
            }
        },
    }
}
