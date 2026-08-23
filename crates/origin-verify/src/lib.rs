#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO Subsystem: origin-verify
// Proof and obligation verification engine.

pub use origin_core::{Status, TypedObligation};

pub fn crate_name() -> &'static str {
    "origin-verify"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_boundary() {
        assert_eq!(crate_name(), "origin-verify");
    }
}
