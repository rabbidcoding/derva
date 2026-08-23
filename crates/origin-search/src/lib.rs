#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO Subsystem: origin-search
// Typed hypothesis grammar, cost-ordered enumeration, constraint-guided pruning, and search intelligence.

pub mod enumerate;
pub mod grammar;
pub mod prune;

pub use enumerate::{CostEnumerator, FrontierState, SearchNode};
pub use grammar::{ASTExpr, GrammarError, GrammarRule, Operator, Production, Type, TypedGrammar};
pub use prune::ConstraintPruner;

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
