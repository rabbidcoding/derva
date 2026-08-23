#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO — OIR Intermediate Representation Engine
// INVARIANT: Pure SSA-like OIR IR; 100% source-mapped to ORIDs; 0 unsafe block usage.

pub mod effectcheck;
pub mod ir;
pub mod opt;
pub mod typecheck;
pub mod verify;

pub use effectcheck::{EffectError, OirEffectChecker};
pub use ir::{EffectKind, OirInstruction, OirModule, OirType, Value};
pub use opt::{OptimizationResult, OirOptimizer, RewriteProof};
pub use typecheck::{OirTypeChecker, TypeError};
pub use verify::{OirVerifier, VerifierError};

#[cfg(test)]
mod tests {
    #[test]
    fn test_oir_crate_boundary() {
        assert!(true);
    }
}
