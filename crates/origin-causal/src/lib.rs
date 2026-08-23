#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO Subsystem: origin-causal
// Causal Operators, Counterfactuals & Structural Causal Models.

pub mod operator;

pub use operator::{CausalOperator, Cost, EffectId, OperatorError, PredicateId, Risk, SchemaId};

pub fn crate_name() -> &'static str {
    "origin-causal"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_causal_boundary() {
        assert_eq!(crate_name(), "origin-causal");
    }
}
