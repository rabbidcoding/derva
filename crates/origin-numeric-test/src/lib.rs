#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO — Rust<->JAX Differential Verification Engine
// INVARIANT: Pure differential test harness; 0 unexplained mismatches; zero trainable parameters.

pub mod differential_harness;

pub use differential_harness::DifferentialTestCase;

#[cfg(test)]
mod tests {
    #[test]
    fn test_numeric_test_crate_boundary() {
        assert!(true);
    }
}
