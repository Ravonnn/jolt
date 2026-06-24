//! Stage-0 placeholder for `jolt-backend-llvm`.

pub const STAGE: &str = "backend-llvm";

#[cfg(test)]
mod tests {
    #[test]
    fn compiles() {
        assert!(!super::STAGE.is_empty());
    }
}
