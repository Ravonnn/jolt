//! Stage-0 placeholder for `jolt-comptime`.

pub const STAGE: &str = "comptime";

#[cfg(test)]
mod tests {
    #[test]
    fn compiles() {
        assert!(!super::STAGE.is_empty());
    }
}
