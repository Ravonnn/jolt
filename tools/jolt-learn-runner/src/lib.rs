//! Local HTTP sidecar that spawns the `jolt` CLI for interactive tutorials.

use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

pub const DEFAULT_PORT: u16 = 3847;
pub const MAX_OUTPUT_BYTES: usize = 256 * 1024;
pub const RUN_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JoltCommand {
    Run,
    Check,
    Test,
    Fmt,
}

impl JoltCommand {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "run" => Some(Self::Run),
            "check" => Some(Self::Check),
            "test" => Some(Self::Test),
            "fmt" => Some(Self::Fmt),
            _ => None,
        }
    }

    fn jolt_args(self) -> &'static [&'static str] {
        match self {
            Self::Run => &["run", "--interpret"],
            Self::Check => &["check"],
            Self::Test => &["test"],
            Self::Fmt => &["fmt"],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunRequest {
    pub source: String,
    #[serde(default = "default_run_command")]
    pub command: String,
}

fn default_run_command() -> String {
    "run".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunFileRequest {
    pub path: String,
    #[serde(default = "default_run_command")]
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunResponse {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub jolt_bin: String,
    pub jolt_available: bool,
    pub capabilities: Vec<String>,
}

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

pub fn resolve_jolt_bin() -> PathBuf {
    if let Ok(path) = std::env::var("JOLT_BIN") {
        return PathBuf::from(path);
    }
    repo_root().join("target/debug/jolt")
}

pub fn validate_repo_path(repo: &Path, user_path: &str) -> Result<PathBuf, String> {
    let path = Path::new(user_path);
    if path.is_absolute() {
        return Err("absolute paths are not allowed".to_string());
    }
    for component in path.components() {
        if matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        ) {
            return Err("path must stay inside the repository".to_string());
        }
    }
    let full = repo.join(path);
    let canonical = full
        .canonicalize()
        .map_err(|e| format!("invalid path: {e}"))?;
    let repo_canonical = repo.canonicalize().map_err(|e| format!("repo root: {e}"))?;
    if !canonical.starts_with(&repo_canonical) {
        return Err("path escapes repository root".to_string());
    }
    if canonical.extension().is_some_and(|e| e == "jolt") {
        Ok(canonical)
    } else {
        Err("path must be a .jolt file".to_string())
    }
}

fn truncate_output(bytes: Vec<u8>) -> String {
    let s = String::from_utf8_lossy(&bytes);
    if s.len() > MAX_OUTPUT_BYTES {
        format!("{}… [truncated]", &s[..MAX_OUTPUT_BYTES])
    } else {
        s.into_owned()
    }
}

fn spawn_jolt(
    jolt_bin: &Path,
    repo: &Path,
    command: JoltCommand,
    target: PathBuf,
) -> Result<RunResponse, String> {
    if !jolt_bin.is_file() {
        return Err(format!(
            "jolt binary not found at {}; run `cargo build -p jolt-cli`",
            jolt_bin.display()
        ));
    }

    let mut cmd = Command::new(jolt_bin);
    cmd.current_dir(repo);
    cmd.args(command.jolt_args());
    cmd.arg(&target);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let _ = tx.send(cmd.output());
    });

    let output = match rx.recv_timeout(RUN_TIMEOUT) {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => return Err(format!("failed to spawn jolt: {e}")),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            return Err("execution timed out after 5 seconds".to_string());
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            return Err("jolt process channel disconnected".to_string());
        }
    };

    Ok(RunResponse {
        success: output.status.success(),
        stdout: truncate_output(output.stdout),
        stderr: truncate_output(output.stderr),
        exit_code: output.status.code().unwrap_or(-1),
    })
}

pub fn run_source(
    jolt_bin: &Path,
    repo: &Path,
    source: &str,
    command: JoltCommand,
) -> Result<RunResponse, String> {
    let temp = tempfile_path(repo);
    std::fs::write(&temp, source).map_err(|e| format!("write temp file: {e}"))?;
    let result = spawn_jolt(jolt_bin, repo, command, temp.clone());
    let _ = std::fs::remove_file(&temp);
    result
}

pub fn run_file(
    jolt_bin: &Path,
    repo: &Path,
    path: &str,
    command: JoltCommand,
) -> Result<RunResponse, String> {
    let file = validate_repo_path(repo, path)?;
    spawn_jolt(jolt_bin, repo, command, file)
}

fn tempfile_path(repo: &Path) -> PathBuf {
    let mut path = repo.join("target");
    let _ = std::fs::create_dir_all(&path);
    path.push(format!("learn-runner-{}.jolt", std::process::id()));
    path
}

pub fn health(jolt_bin: &Path) -> HealthResponse {
    HealthResponse {
        status: "ok".to_string(),
        jolt_bin: jolt_bin.display().to_string(),
        jolt_available: jolt_bin.is_file(),
        capabilities: vec![
            "run".to_string(),
            "check".to_string(),
            "test".to_string(),
            "fmt".to_string(),
        ],
    }
}

pub fn handle_request(
    method: &str,
    url_path: &str,
    body: &str,
    jolt_bin: &Path,
    repo: &Path,
) -> (u16, String) {
    match (method, url_path) {
        ("GET", "/api/v1/health") => json_response(200, &health(jolt_bin)),
        ("POST", "/api/v1/run") => match serde_json::from_str::<RunRequest>(body) {
            Ok(req) => {
                let cmd = JoltCommand::parse(&req.command).unwrap_or(JoltCommand::Run);
                match run_source(jolt_bin, repo, &req.source, cmd) {
                    Ok(resp) => json_response(200, &resp),
                    Err(e) => error_response(500, &e),
                }
            }
            Err(e) => error_response(400, &format!("invalid JSON: {e}")),
        },
        ("POST", "/api/v1/run-file") => match serde_json::from_str::<RunFileRequest>(body) {
            Ok(req) => {
                let cmd = JoltCommand::parse(&req.command).unwrap_or(JoltCommand::Run);
                match run_file(jolt_bin, repo, &req.path, cmd) {
                    Ok(resp) => json_response(200, &resp),
                    Err(e) => error_response(400, &e),
                }
            }
            Err(e) => error_response(400, &format!("invalid JSON: {e}")),
        },
        ("OPTIONS", _) => (204, String::new()),
        _ => error_response(404, "not found"),
    }
}

fn json_response<T: Serialize>(status: u16, value: &T) -> (u16, String) {
    (
        status,
        serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string()),
    )
}

fn error_response(status: u16, message: &str) -> (u16, String) {
    (status, serde_json::json!({ "error": message }).to_string())
}

pub fn cors_headers() -> Vec<tiny_http::Header> {
    vec![
        tiny_http::Header::from_bytes(b"Access-Control-Allow-Origin", b"*").unwrap(),
        tiny_http::Header::from_bytes(b"Access-Control-Allow-Methods", b"GET, POST, OPTIONS")
            .unwrap(),
        tiny_http::Header::from_bytes(b"Access-Control-Allow-Headers", b"Content-Type").unwrap(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_path_traversal() {
        let repo = repo_root();
        assert!(validate_repo_path(&repo, "../Cargo.toml").is_err());
        assert!(validate_repo_path(&repo, "tests/tutorial/hello.jolt").is_ok());
    }

    #[test]
    fn health_reports_capabilities() {
        let h = health(&resolve_jolt_bin());
        assert_eq!(h.status, "ok");
        assert_eq!(h.capabilities.len(), 4);
    }

    #[test]
    fn parse_commands() {
        assert_eq!(JoltCommand::parse("run"), Some(JoltCommand::Run));
        assert_eq!(JoltCommand::parse("check"), Some(JoltCommand::Check));
        assert_eq!(JoltCommand::parse("nope"), None);
    }
}
