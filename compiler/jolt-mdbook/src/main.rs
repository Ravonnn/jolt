//! mdBook preprocessor: expand `{{#snippet id}}` from learn/curriculum.yaml.

use serde::Deserialize;
use std::collections::HashMap;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct SnippetEntry {
    path: String,
}

#[derive(Debug, Deserialize)]
struct Curriculum {
    #[serde(default)]
    snippets: HashMap<String, SnippetEntry>,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn load_snippets() -> HashMap<String, String> {
    let path = repo_root().join("learn/curriculum.yaml");
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let curriculum: Curriculum = serde_yaml::from_str(&text).unwrap_or(Curriculum {
        snippets: HashMap::new(),
    });
    let mut out = HashMap::new();
    for (id, entry) in curriculum.snippets {
        if let Ok(content) = std::fs::read_to_string(repo_root().join(&entry.path)) {
            out.insert(id, content);
        }
    }
    out
}

fn expand_snippets(content: &str, snippets: &HashMap<String, String>) -> String {
    let mut result = content.to_string();
    for (id, code) in snippets {
        let needle = format!("{{{{#snippet {id}}}}}");
        if result.contains(&needle) {
            let fence = format!("```jolt runnable\n{code}```");
            result = result.replace(&needle, &fence);
        }
    }
    result
}

fn process_sections(sections: &mut [serde_json::Value], snippets: &HashMap<String, String>) {
    for section in sections.iter_mut() {
        if let Some(chapters) = section.get_mut("sub_items").and_then(|s| s.as_array_mut()) {
            process_sections(chapters, snippets);
        }
        if section.get("type").and_then(|t| t.as_str()) == Some("chapter") {
            if let Some(content) = section.get("content").and_then(|c| c.as_str()) {
                let expanded = expand_snippets(content, snippets);
                if let Some(obj) = section.as_object_mut() {
                    obj.insert("content".to_string(), serde_json::Value::String(expanded));
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 3 && args[1] == "supports" {
        if args[2] == "html" {
            print!("true");
        }
        return;
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read stdin");
    let snippets = load_snippets();
    let mut book: serde_json::Value =
        serde_json::from_str(&input).expect("parse mdbook preprocessor input");

    if let Some(chapters) = book.get_mut("sections").and_then(|s| s.as_array_mut()) {
        process_sections(chapters, &snippets);
    }

    print!("{}", serde_json::to_string(&book).expect("serialize book"));
}
