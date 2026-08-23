#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO Subsystem: origin-kernel
// Authoritative execution kernel and transactional state transition engine.

pub mod budget;
pub mod txn;

pub use budget::{BudgetError, ResourceBudget, StepCost};
pub use origin_core::{Claim, Evidence, Obligation, Operator, State, StateTxn};
pub use txn::{AtomicTxnEngine, TxnEngineError};

pub fn crate_name() -> &'static str {
    "origin-kernel"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_boundary() {
        assert_eq!(crate_name(), "origin-kernel");
        let state = State::new();
        let txn = StateTxn::new(state);
        assert!(txn.commit().is_ok());
    }
}
