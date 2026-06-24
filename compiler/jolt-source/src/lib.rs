//! Source locations and in-memory source maps.

use std::collections::HashMap;

/// Opaque identifier for a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(pub u32);

/// A byte range in a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub file: FileId,
    pub start: u32,
    pub end: u32,
}

/// In-memory map from file ids to source text.
#[derive(Debug, Default)]
pub struct SourceMap {
    files: HashMap<FileId, String>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, file: FileId, text: impl Into<String>) {
        self.files.insert(file, text.into());
    }

    pub fn get(&self, file: FileId) -> Option<&str> {
        self.files.get(&file).map(String::as_str)
    }

    pub fn span_text<'a>(&'a self, span: &Span) -> Option<&'a str> {
        let text = self.files.get(&span.file)?;
        let start = span.start as usize;
        let end = span.end as usize;
        if end < start || end > text.len() {
            return None;
        }
        Some(&text[start..end])
    }
}

pub const STAGE: &str = "source";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_text_resolves_slice() {
        let mut map = SourceMap::new();
        let fid = FileId(1);
        map.insert(fid, "hello world");
        let span = Span {
            file: fid,
            start: 6,
            end: 11,
        };
        assert_eq!(map.span_text(&span), Some("world"));
    }
}
