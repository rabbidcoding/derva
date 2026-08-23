#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO Subsystem: origin-logic
// Horn-rule IR and logic evaluation engine.

pub mod horn;

pub use horn::{HornError, HornRule, Literal, Term};

pub fn crate_name() -> &'static str {
    "origin-logic"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logic_boundary() {
        assert_eq!(crate_name(), "origin-logic");
    }
}
