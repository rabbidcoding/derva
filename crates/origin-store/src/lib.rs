#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO Subsystem: origin-store
// Content-addressed immutable persistence layer.

pub use origin_core::{Canonical, ORID};

pub fn crate_name() -> &'static str {
    "origin-store"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_boundary() {
        assert_eq!(crate_name(), "origin-store");
    }
}
