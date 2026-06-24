use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::graph::DependencyGraph;
use crate::key::{hash_value, InputHash, QueryKey, QueryName, ResultHash};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CellState {
    Clean,
    Dirty,
}

struct CachedCell {
    state: CellState,
    result_hash: ResultHash,
    value: Box<dyn Any + Send>,
}

/// Memoized query engine with red/green invalidation and early cutoff.
///
/// See `jolt-caching-system.md` §2: queries are keyed by input hash, record dependencies,
/// and skip propagating invalidation when a recomputed result hash is unchanged.
#[derive(Default)]
pub struct QueryEngine {
    cells: HashMap<QueryKey, CachedCell>,
    graph: DependencyGraph,
    stack: Vec<QueryKey>,
    compute_counts: HashMap<QueryName, usize>,
    dirty: HashSet<QueryKey>,
}

impl QueryEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register that the current query reads an external input (e.g. file bytes).
    pub fn read_input(&mut self, input: InputHash) {
        if let Some(&reader) = self.stack.last() {
            self.graph.record_input_dep(reader, input);
        }
    }

    /// Execute or return a memoized query.
    ///
    /// `external_inputs` are content hashes (e.g. file bytes) that this query reads; used for
    /// invalidation when those inputs change.
    pub fn query<T>(
        &mut self,
        name: QueryName,
        input: InputHash,
        external_inputs: &[InputHash],
        f: impl FnOnce(&mut Self) -> T,
    ) -> T
    where
        T: Hash + Clone + Send + 'static,
    {
        let key = QueryKey { name, input };

        if let Some(parent) = self.stack.last().copied() {
            self.graph.record_query_dep(parent, key);
        }

        if let Some(cell) = self.cells.get(&key) {
            if cell.state == CellState::Clean && !self.dirty.contains(&key) {
                let value = cell
                    .value
                    .downcast_ref::<T>()
                    .expect("query result type mismatch")
                    .clone();
                return value;
            }
        }

        self.stack.push(key);
        for &ext in external_inputs {
            self.graph.record_input_dep(key, ext);
        }
        let result = f(self);
        self.stack.pop();

        let result_hash = hash_value(&result);
        self.bump_compute(name);

        let old_hash = self.cells.get(&key).map(|c| c.result_hash);
        let early_cutoff = old_hash == Some(result_hash);

        self.graph.clear_query_deps(key);
        for &ext in external_inputs {
            self.graph.record_input_dep(key, ext);
        }

        if let Some(parent) = self.stack.last().copied() {
            self.graph.record_query_dep(parent, key);
        }

        self.cells.insert(
            key,
            CachedCell {
                state: CellState::Clean,
                result_hash,
                value: Box::new(result.clone()),
            },
        );
        self.dirty.remove(&key);

        if early_cutoff {
            // Green: semantic output unchanged — do not dirty downstream queries.
        } else if old_hash.is_some() {
            self.mark_dependents_dirty(key);
        }

        result
    }

    /// Invalidate all queries that transitively depend on this input.
    pub fn invalidate_input(&mut self, input: InputHash) {
        let affected = self.graph.queries_invalidated_by_input(input);
        for key in affected {
            self.dirty.insert(key);
            if let Some(cell) = self.cells.get_mut(&key) {
                cell.state = CellState::Dirty;
            }
        }
    }

    /// Force recomputation of all dirty queries (eager validation pass).
    pub fn recompute_all_dirty(&mut self) {
        let dirty: Vec<QueryKey> = self.dirty.iter().copied().collect();
        for key in dirty {
            self.dirty.remove(&key);
            if let Some(cell) = self.cells.get_mut(&key) {
                cell.state = CellState::Dirty;
            }
        }
    }

    pub fn compute_count(&self, name: QueryName) -> usize {
        self.compute_counts.get(&name).copied().unwrap_or(0)
    }

    pub fn is_dirty(&self, key: QueryKey) -> bool {
        self.dirty.contains(&key)
    }

    fn bump_compute(&mut self, name: QueryName) {
        *self.compute_counts.entry(name).or_insert(0) += 1;
    }

    fn mark_dependents_dirty(&mut self, key: QueryKey) {
        let mut queue: Vec<QueryKey> = self.graph.direct_dependents(key).into_iter().collect();
        let mut seen = HashSet::new();

        while let Some(dep) = queue.pop() {
            if !seen.insert(dep) {
                continue;
            }
            self.dirty.insert(dep);
            if let Some(cell) = self.cells.get_mut(&dep) {
                cell.state = CellState::Dirty;
            }
            queue.extend(self.graph.direct_dependents(dep));
        }
    }
}
