#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO — OIR Intermediate Representation Engine
// INVARIANT: Pure SSA-like OIR IR; 100% source-mapped to ORIDs; 0 unsafe block usage.

pub mod effectcheck;
pub mod ir;
pub mod typecheck;

pub use effectcheck::{EffectError, OirEffectChecker};
pub use ir::{EffectKind, OirInstruction, OirModule, OirType, Value};
pub use typecheck::{OirTypeChecker, TypeError};

#[cfg(test)]
mod tests {
    #[test]
    fn test_oir_crate_boundary() {
        assert!(true);
    }
}
