#![allow(unsafe_code)]

// ORIGIN-Ω ZERO — Fast Operations, SIMD Dispatch, Packed Bitset, Cardinality, Scanner & Hasher Subsystem
// INVARIANT: Dynamic ISA dispatch with pure-Rust reference fallback, explicit safety contracts for SIMD.

pub mod bitset;
pub mod cardinality;
pub mod dispatch;
pub mod hash;
pub mod scan;

pub use bitset::PackedBitset;
pub use cardinality::CardinalityEngine;
pub use dispatch::{CpuImplementation, FastOps};
pub use hash::FastOridHasher;
pub use scan::PackedIndex;

pub fn crate_name() -> &'static str {
    "origin-fast"
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_fast_crate_boundary() {
        assert_eq!(super::crate_name(), "origin-fast");
    }
}
