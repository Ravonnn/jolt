//! Stage-0 placeholder for `jolt-capability`.

pub const STAGE: &str = "capability";

#[cfg(test)]
mod tests {
    #[test]
    fn compiles() {
        assert!(!super::STAGE.is_empty());
    }
}
