#![forbid(unsafe_code)]

// INVARIANT: 100% completeness on Horn subset; replayable proof trace per derived claim; incremental update <= 20% facts re-evaluated.
// KPI: 100% completeness; 100% replayable proof trace; incremental update <= 20% facts re-evaluated.

pub mod forward;

pub use forward::{DerivedConsequence, ForwardReasoner, ProofStep, ProofTrace};

pub fn crate_name() -> &'static str {
    "origin-reason"
}
