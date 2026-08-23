#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO Subsystem: origin-causal
// Causal Operators, Counterfactuals & Structural Causal Models.

pub mod compose;
pub mod counterfactual;
pub mod journal;
pub mod operator;
pub mod promotion;

pub use compose::{
    ComposedOperator, CompositionError, CompositionPolicy, CompositionProof,
    OperatorCompositionChecker,
};
pub use counterfactual::{Capability, CounterfactualFork, StateCounterfactualExt};
pub use journal::{
    EnvironmentReceipt, InterventionJournal, InterventionOutcome, InterventionRecord, JournalError,
};
pub use operator::{CausalOperator, Cost, EffectId, OperatorError, PredicateId, Risk, SchemaId};
pub use promotion::{CausalPromotionValidator, PromotionRecord};

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
