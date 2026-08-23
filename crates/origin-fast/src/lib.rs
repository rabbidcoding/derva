#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO — Fast Operations & SIMD Dispatch Subsystem
// INVARIANT: Dynamic ISA dispatch with pure-Rust reference fallback, zero unsafe code.

pub mod dispatch;

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
