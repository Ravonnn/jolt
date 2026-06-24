//! Validate learn/curriculum.yaml and sync learn content into docs/.

use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct SnippetEntry {
    path: String,
    #[serde(default)]
    expected_stdout: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    harness: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LessonEntry {
    id: String,
    file: String,
}

#[derive(Debug, Deserialize)]
struct ExampleEntry {
    id: String,
    file: String,
    #[serde(default)]
    snippet: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GuideEntry {
    id: String,
    file: String,
}

#[derive(Debug, Deserialize)]
struct MigrationEntry {
    id: String,
    file: String,
}

#[derive(Debug, Deserialize)]
struct QuizEntry {
    id: String,
    file: String,
}

#[derive(Debug, Deserialize)]
struct Curriculum {
    #[serde(default)]
    lessons: Vec<LessonEntry>,
    #[serde(default)]
    snippets: std::collections::HashMap<String, SnippetEntry>,
    #[serde(default)]
    examples: Vec<ExampleEntry>,
    #[serde(default)]
    guides: Vec<GuideEntry>,
    #[serde(default)]
    migrations: Vec<MigrationEntry>,
    #[serde(default)]
    quizzes: Vec<QuizEntry>,
}

pub fn sync_learn_to_docs(repo: &Path) -> Result<(), String> {
    let manifest = repo.join("learn/curriculum.yaml");
    let text =
        fs::read_to_string(&manifest).map_err(|e| format!("read {}: {e}", manifest.display()))?;
    let curriculum: Curriculum =
        serde_yaml::from_str(&text).map_err(|e| format!("parse curriculum.yaml: {e}"))?;

    let learn = repo.join("learn");
    let docs_learn = repo.join("docs/learn");
    copy_dir_recursive(&learn, &docs_learn, &curriculum, repo, |path| {
        path.extension()
            .is_some_and(|e| e == "md" || e == "js" || e == "css" || e == "toml" || e == "yaml")
            || path.file_name().is_some_and(|n| n == "curriculum.yaml")
    })?;
    Ok(())
}

fn expand_snippets(content: &str, curriculum: &Curriculum, repo: &Path) -> String {
    let mut result = content.to_string();
    for (id, entry) in &curriculum.snippets {
        let needle = format!("{{{{#snippet {id}}}}}");
        if result.contains(&needle) {
            let code = fs::read_to_string(repo.join(&entry.path)).unwrap_or_default();
            let fence = format!("```jolt runnable\n{code}```");
            result = result.replace(&needle, &fence);
        }
    }
    result
}

fn copy_dir_recursive<F>(
    src: &Path,
    dst: &Path,
    curriculum: &Curriculum,
    repo: &Path,
    filter: F,
) -> Result<(), String>
where
    F: Fn(&Path) -> bool + Copy,
{
    if !src.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(dst).map_err(|e| format!("mkdir {}: {e}", dst.display()))?;
    for entry in fs::read_dir(src).map_err(|e| format!("read_dir {}: {e}", src.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let rel = path.strip_prefix(src).map_err(|e| e.to_string())?;
        let target = dst.join(rel);
        if path.is_dir() {
            copy_dir_recursive(&path, &target, curriculum, repo, filter)?;
        } else if filter(&path) {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            if path.extension().is_some_and(|e| e == "md") {
                let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
                let expanded = expand_snippets(&raw, curriculum, repo);
                fs::write(&target, expanded).map_err(|e| e.to_string())?;
            } else {
                fs::copy(&path, &target).map_err(|e| format!("copy {}: {e}", path.display()))?;
            }
        }
    }
    Ok(())
}

pub fn verify_curriculum(repo: &Path) -> Result<(), String> {
    let manifest = repo.join("learn/curriculum.yaml");
    let text =
        fs::read_to_string(&manifest).map_err(|e| format!("read {}: {e}", manifest.display()))?;
    let curriculum: Curriculum =
        serde_yaml::from_str(&text).map_err(|e| format!("parse curriculum.yaml: {e}"))?;

    let mut snippet_ids = HashSet::new();
    for (id, entry) in &curriculum.snippets {
        if !snippet_ids.insert(id.clone()) {
            return Err(format!("duplicate snippet id: {id}"));
        }
        let path = repo.join(&entry.path);
        if !path.is_file() {
            return Err(format!("snippet {id}: missing file {}", entry.path));
        }
        if let Some(stdout) = &entry.expected_stdout {
            let stdout_path = repo.join(stdout);
            if !stdout_path.is_file() {
                return Err(format!("snippet {id}: missing stdout {stdout}"));
            }
        }
    }

    for lesson in &curriculum.lessons {
        let path = repo.join(&lesson.file);
        if !path.is_file() {
            return Err(format!(
                "lesson {}: missing file {}",
                lesson.id, lesson.file
            ));
        }
    }

    for ex in &curriculum.examples {
        let path = repo.join(&ex.file);
        if !path.is_file() {
            return Err(format!("example {}: missing file {}", ex.id, ex.file));
        }
        if let Some(snippet) = &ex.snippet {
            if !curriculum.snippets.contains_key(snippet) {
                return Err(format!("example {}: unknown snippet {snippet}", ex.id));
            }
        }
    }

    for guide in &curriculum.guides {
        let path = repo.join(&guide.file);
        if !path.is_file() {
            return Err(format!("guide {}: missing file {}", guide.id, guide.file));
        }
    }

    for mig in &curriculum.migrations {
        let path = repo.join(&mig.file);
        if !path.is_file() {
            return Err(format!("migration {}: missing file {}", mig.id, mig.file));
        }
    }

    for quiz in &curriculum.quizzes {
        let path = repo.join(&quiz.file);
        if !path.is_file() {
            return Err(format!("quiz {}: missing file {}", quiz.id, quiz.file));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_curriculum_passes() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        verify_curriculum(&repo.canonicalize().unwrap()).expect("curriculum should verify");
    }
}
