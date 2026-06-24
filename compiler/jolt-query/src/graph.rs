use std::collections::{HashMap, HashSet, VecDeque};

use crate::key::{InputHash, QueryKey};

/// Reverse edges: dependent query → queries/inputs it depends on.
#[derive(Debug, Default)]
pub struct DependencyGraph {
    /// Query → queries it read during last successful computation.
    query_deps: HashMap<QueryKey, HashSet<QueryKey>>,
    /// Query → external inputs it read.
    input_deps: HashMap<QueryKey, HashSet<InputHash>>,
    /// Input → queries that directly read it (for invalidation).
    input_readers: HashMap<InputHash, HashSet<QueryKey>>,
    /// Parent query → child queries recorded during nested `query()` calls.
    query_dependents: HashMap<QueryKey, HashSet<QueryKey>>,
}

impl DependencyGraph {
    pub fn record_query_dep(&mut self, reader: QueryKey, dependency: QueryKey) {
        self.query_deps
            .entry(reader)
            .or_default()
            .insert(dependency);
        self.query_dependents
            .entry(dependency)
            .or_default()
            .insert(reader);
    }

    pub fn record_input_dep(&mut self, reader: QueryKey, input: InputHash) {
        self.input_deps.entry(reader).or_default().insert(input);
        self.input_readers.entry(input).or_default().insert(reader);
    }

    pub fn clear_query_deps(&mut self, key: QueryKey) {
        if let Some(deps) = self.query_deps.remove(&key) {
            for dep in deps {
                if let Some(dependents) = self.query_dependents.get_mut(&dep) {
                    dependents.remove(&key);
                }
            }
        }
        if let Some(inputs) = self.input_deps.remove(&key) {
            for input in inputs {
                if let Some(readers) = self.input_readers.get_mut(&input) {
                    readers.remove(&key);
                }
            }
        }
    }

    /// All queries transitively affected by an input change.
    pub fn queries_invalidated_by_input(&self, input: InputHash) -> HashSet<QueryKey> {
        let mut affected = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(readers) = self.input_readers.get(&input) {
            for &key in readers {
                if affected.insert(key) {
                    queue.push_back(key);
                }
            }
        }

        while let Some(key) = queue.pop_front() {
            if let Some(dependents) = self.query_dependents.get(&key) {
                for &dep in dependents {
                    if affected.insert(dep) {
                        queue.push_back(dep);
                    }
                }
            }
        }

        affected
    }

    /// Direct child queries registered for a parent (used to propagate dirtiness).
    pub fn direct_dependents(&self, key: QueryKey) -> HashSet<QueryKey> {
        self.query_dependents.get(&key).cloned().unwrap_or_default()
    }
}
