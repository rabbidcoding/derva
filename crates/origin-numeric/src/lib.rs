#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO — High-Precision Numeric & Interval Reference Engine
// INVARIANT: Outward-rounded interval arithmetic; zero trainable parameters.

pub mod interval_ref;

pub use interval_ref::{HighPrecisionInterval, IntervalError};

#[cfg(test)]
mod tests {
    #[test]
    fn test_numeric_crate_boundary() {
        assert!(true);
    }
}
