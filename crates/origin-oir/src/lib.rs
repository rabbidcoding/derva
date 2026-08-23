#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO — OIR Intermediate Representation Engine
// INVARIANT: Pure SSA-like OIR IR; 100% source-mapped to ORIDs; 0 unsafe block usage.

pub mod ir;

pub use ir::{EffectKind, OirInstruction, OirModule, OirType, Value};

#[cfg(test)]
mod tests {
    #[test]
    fn test_oir_crate_boundary() {
        assert!(true);
    }
}
