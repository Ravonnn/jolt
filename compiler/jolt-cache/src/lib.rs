//! Stage-0 placeholder for `jolt-cache`.

pub const STAGE: &str = "cache";

#[cfg(test)]
mod tests {
    #[test]
    fn compiles() {
        assert!(!super::STAGE.is_empty());
    }
}
