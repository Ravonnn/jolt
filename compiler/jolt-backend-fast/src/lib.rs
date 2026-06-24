//! Stage-0 placeholder for `jolt-backend-fast`.

pub const STAGE: &str = "backend-fast";

#[cfg(test)]
mod tests {
    #[test]
    fn compiles() {
        assert!(!super::STAGE.is_empty());
    }
}
