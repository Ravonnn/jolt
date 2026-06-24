//! Stage-0 placeholder for `jolt-mono`.

pub const STAGE: &str = "mono";

#[cfg(test)]
mod tests {
    #[test]
    fn compiles() {
        assert!(!super::STAGE.is_empty());
    }
}
