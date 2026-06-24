//! Query-based incremental compilation engine (skeleton).
//!
//! Implements the red/green invalidation model from `jolt-caching-system.md` §2:
//! memoized queries, dependency edges, invalidation on input change, and early cutoff
//! when a recomputed result hash is unchanged.

mod engine;
mod graph;
mod key;

pub use engine::QueryEngine;
pub use key::{hash_bytes, hash_value, InputHash, QueryKey, QueryName, ResultHash};

pub const STAGE: &str = "query";

/// Demo query names for tests and integration smoke tests.
pub mod demo {
    use super::*;

    pub const INPUT_BYTES: QueryName = QueryName("input_bytes");
    pub const PARSE_OF: QueryName = QueryName("parse_of");
    pub const TYPE_OF: QueryName = QueryName("type_of");
    pub const UNRELATED: QueryName = QueryName("unrelated");
    pub const LOWER_OF: QueryName = QueryName("lower_of");
}

#[cfg(test)]
mod tests {
    use super::demo::*;
    use super::*;

    fn file_input(content: &str) -> InputHash {
        hash_bytes(content.as_bytes())
    }

    fn run_pipeline(engine: &mut QueryEngine, content: &str) -> (u64, u64, u64) {
        let file = file_input(content);
        let bytes = engine.query(INPUT_BYTES, file, &[file], |_| content.len() as u64);
        let ast = engine.query(PARSE_OF, file, &[file], |_| bytes.wrapping_mul(31));
        let ty = engine.query(TYPE_OF, file, &[file], |eng| {
            let ast = eng.query(PARSE_OF, file, &[file], |_| bytes.wrapping_mul(31));
            ast.wrapping_add(1)
        });
        (bytes, ast, ty)
    }

    #[test]
    fn cache_hit_on_unchanged_input() {
        let mut engine = QueryEngine::new();
        run_pipeline(&mut engine, "hello");
        let parse_count = engine.compute_count(PARSE_OF);
        let type_count = engine.compute_count(TYPE_OF);

        run_pipeline(&mut engine, "hello");

        assert_eq!(engine.compute_count(PARSE_OF), parse_count);
        assert_eq!(engine.compute_count(TYPE_OF), type_count);
    }

    #[test]
    fn invalidates_only_dependents() {
        let mut engine = QueryEngine::new();
        run_pipeline(&mut engine, "hello");

        engine.query(UNRELATED, InputHash(99), &[], |_| 1u64);

        let unrelated_before = engine.compute_count(UNRELATED);
        let parse_before = engine.compute_count(PARSE_OF);

        engine.invalidate_input(file_input("hello"));
        run_pipeline(&mut engine, "hello");

        assert!(engine.compute_count(PARSE_OF) > parse_before);
        assert!(engine.compute_count(TYPE_OF) > 0);
        assert_eq!(engine.compute_count(UNRELATED), unrelated_before);
    }

    #[test]
    fn early_cutoff_stops_propagation() {
        let mut engine = QueryEngine::new();
        let file = file_input("content");

        engine.query(PARSE_OF, file, &[file], |_| 100u64);
        engine.query(TYPE_OF, file, &[file], |_| 42u64);
        engine.query(LOWER_OF, file, &[], |eng| {
            eng.query(TYPE_OF, file, &[file], |_| 42u64);
            84u64
        });

        let lower_before = engine.compute_count(LOWER_OF);
        let type_before = engine.compute_count(TYPE_OF);

        engine.invalidate_input(file);
        engine.query(PARSE_OF, file, &[file], |_| 999u64);
        engine.query(TYPE_OF, file, &[file], |_| 42u64);

        assert!(engine.compute_count(TYPE_OF) > type_before);
        engine.query(LOWER_OF, file, &[], |eng| {
            eng.query(TYPE_OF, file, &[file], |_| 42u64);
            84u64
        });

        assert_eq!(
            engine.compute_count(LOWER_OF),
            lower_before,
            "early cutoff should avoid recomputing lower_of when type_of hash is stable"
        );
    }
}
