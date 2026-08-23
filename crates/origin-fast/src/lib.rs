#![allow(unsafe_code)]

// ORIGIN-Ω ZERO — Fast Operations, SIMD Dispatch & Packed Bitset Subsystem
// INVARIANT: Dynamic ISA dispatch with pure-Rust reference fallback, explicit safety contracts for SIMD.

pub mod bitset;
pub mod dispatch;

pub use bitset::PackedBitset;
pub use dispatch::{CpuImplementation, FastOps};

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
