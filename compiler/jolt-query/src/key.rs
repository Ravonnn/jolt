use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Identifier for a query definition (e.g. `parse_of`, `type_of`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QueryName(pub &'static str);

/// Hash of a query's inputs (content-addressed, not timestamps).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct InputHash(pub u64);

/// Hash of a query's output; used for early cutoff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResultHash(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QueryKey {
    pub name: QueryName,
    pub input: InputHash,
}

pub fn hash_value<T: Hash>(value: &T) -> ResultHash {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    ResultHash(hasher.finish())
}

pub fn hash_bytes(bytes: &[u8]) -> InputHash {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    InputHash(hasher.finish())
}
