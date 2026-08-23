#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO Subsystem: origin-search
// Typed hypothesis grammar and proof search intelligence.

pub mod grammar;

pub use grammar::{ASTExpr, GrammarError, GrammarRule, Operator, Production, Type, TypedGrammar};

pub fn crate_name() -> &'static str {
    "origin-search"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_boundary() {
        assert_eq!(crate_name(), "origin-search");
    }
}
