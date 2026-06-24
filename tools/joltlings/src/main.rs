//! Rustlings-style exercise runner for Jolt Tiny.

use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "joltlings", about = "Jolt hands-on exercises")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Copy exercises into ./joltlings/
    Init,
    /// List exercises
    List,
    /// Run one exercise by name or number
    Run { name: Option<String> },
    /// Re-run on file changes (poll)
    Watch { name: Option<String> },
    /// Run all exercises
    Verify,
    /// Reset ./joltlings/ from embedded templates
    Reset,
}

#[derive(Debug, Deserialize)]
struct Exercise {
    name: String,
    path: String,
    #[serde(default)]
    hint: String,
    #[serde(default = "default_mode")]
    mode: String,
}

fn default_mode() -> String {
    "run".to_string()
}

#[derive(Debug, Deserialize)]
struct Info {
    exercise: Vec<Exercise>,
}

fn embedded_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn workspace_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("joltlings")
}

fn jolt_bin() -> PathBuf {
    if let Ok(p) = std::env::var("JOLT_BIN") {
        return PathBuf::from(p);
    }
    embedded_root()
        .join("../../target/debug/jolt")
        .canonicalize()
        .unwrap_or_else(|_| embedded_root().join("../../target/debug/jolt"))
}

fn repo_root() -> PathBuf {
    embedded_root()
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn load_info(root: &Path) -> Result<Info, String> {
    let path = root.join("info.toml");
    let text = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    toml::from_str(&text).map_err(|e| format!("parse info.toml: {e}"))
}

fn resolve_exercise_path(root: &Path, ex: &Exercise) -> PathBuf {
    root.join(&ex.path)
}

fn run_jolt(mode: &str, file: &Path) -> Result<bool, String> {
    let jolt = jolt_bin();
    if !jolt.is_file() {
        return Err("jolt not found — run `cargo build -p jolt-cli`".to_string());
    }
    let repo = repo_root();
    let rel = file
        .strip_prefix(&repo)
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|_| file.to_path_buf());

    let mut cmd = Command::new(&jolt);
    cmd.current_dir(&repo);
    match mode {
        "check" => {
            cmd.args(["check", rel.to_str().unwrap_or("")]);
        }
        "test" => {
            cmd.args(["test", rel.to_str().unwrap_or("")]);
        }
        _ => {
            cmd.args(["run", "--interpret", rel.to_str().unwrap_or("")]);
        }
    }
    let output = cmd.output().map_err(|e| format!("spawn jolt: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stdout.is_empty() {
        print!("{stdout}");
    }
    if !stderr.is_empty() {
        eprint!("{stderr}");
    }
    Ok(output.status.success())
}

fn copy_dir(src: &Path, dst: &Path) -> Result<(), String> {
    if !src.is_dir() {
        return Err(format!("missing {}", src.display()));
    }
    fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = entry.file_name();
        let target = dst.join(name);
        if path.is_dir() {
            copy_dir(&path, &target)?;
        } else {
            fs::copy(&path, &target).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn cmd_init() -> Result<(), String> {
    let dst = workspace_dir();
    let src = embedded_root();
    if dst.exists() {
        return Err(format!(
            "{} already exists — use `joltlings reset`",
            dst.display()
        ));
    }
    copy_dir(&src.join("exercises"), &dst.join("exercises"))?;
    fs::copy(src.join("info.toml"), dst.join("info.toml")).map_err(|e| e.to_string())?;
    println!("Initialized {}", dst.display());
    Ok(())
}

fn cmd_reset() -> Result<(), String> {
    let dst = workspace_dir();
    if dst.exists() {
        fs::remove_dir_all(&dst).map_err(|e| e.to_string())?;
    }
    cmd_init()
}

fn find_exercise<'a>(info: &'a Info, name: &str) -> Option<&'a Exercise> {
    if let Ok(n) = name.parse::<usize>() {
        return info.exercise.get(n.saturating_sub(1));
    }
    info.exercise
        .iter()
        .find(|e| e.name == name || e.path.contains(name))
}

fn cmd_run(root: &Path, name: Option<&str>, watch: bool) -> Result<(), String> {
    let info = load_info(root)?;
    let ex = match name {
        Some(n) => find_exercise(&info, n).ok_or_else(|| format!("unknown exercise: {n}"))?,
        None => info.exercise.first().ok_or("no exercises")?,
    };
    let path = resolve_exercise_path(root, ex);
    println!("{} ({})", ex.name, ex.mode);
    if !ex.hint.is_empty() {
        println!("hint: {}", ex.hint);
    }

    loop {
        print!("checking… ");
        let ok = run_jolt(&ex.mode, &path)?;
        if ok {
            println!("✓ ok");
            if !watch {
                return Ok(());
            }
        } else {
            println!("✗ failed");
            if !watch {
                return Err("exercise failed".to_string());
            }
        }
        if !watch {
            break;
        }
        std::thread::sleep(Duration::from_millis(800));
    }
    Ok(())
}

fn cmd_verify(root: &Path) -> Result<(), String> {
    let info = load_info(root)?;
    let mut failed = 0usize;
    for ex in &info.exercise {
        let path = resolve_exercise_path(root, ex);
        print!("{} … ", ex.name);
        match run_jolt(&ex.mode, &path) {
            Ok(true) => println!("ok"),
            Ok(false) => {
                println!("FAILED");
                failed += 1;
            }
            Err(e) => {
                println!("ERROR: {e}");
                failed += 1;
            }
        }
    }
    if failed > 0 {
        Err(format!("{failed} exercise(s) failed"))
    } else {
        println!("all {} exercises passed", info.exercise.len());
        Ok(())
    }
}

fn cmd_list(root: &Path) -> Result<(), String> {
    let info = load_info(root)?;
    for (i, ex) in info.exercise.iter().enumerate() {
        println!("{:2}. {} [{}] {}", i + 1, ex.name, ex.mode, ex.path);
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    let root = if workspace_dir().join("info.toml").is_file() {
        workspace_dir()
    } else {
        embedded_root()
    };

    let result = match cli.command {
        Commands::Init => cmd_init(),
        Commands::Reset => cmd_reset(),
        Commands::List => cmd_list(&root),
        Commands::Run { name } => cmd_run(&root, name.as_deref(), false),
        Commands::Watch { name } => cmd_run(&root, name.as_deref(), true),
        Commands::Verify => cmd_verify(&root),
    };

    if let Err(e) = result {
        eprintln!("joltlings: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_info_loads() {
        let info = load_info(&embedded_root()).expect("info.toml");
        assert!(info.exercise.len() >= 10);
    }

    #[test]
    fn exercise_paths_exist() {
        let root = embedded_root();
        let info = load_info(&root).unwrap();
        for ex in &info.exercise {
            let p = resolve_exercise_path(&root, ex);
            assert!(p.is_file(), "missing {}", p.display());
        }
    }
}
