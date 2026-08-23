#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO — OIR Compiler Subsystem
// INVARIANT: Safe region stability detection, zero unsafe code, strict obligation checking.

pub mod stability;

pub use stability::{EligibilityResult, RegionMetrics, StabilityConfig, StableRegionDetector};

pub fn crate_name() -> &'static str {
    "origin-compiler"
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_compiler_crate_boundary() {
        assert_eq!(super::crate_name(), "origin-compiler");
    }
}
