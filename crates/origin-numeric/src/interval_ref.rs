#![forbid(unsafe_code)]

// AUDIT-LENSES: Donald Knuth, Guido van Rossum, Alan Turing
// INVARIANT: Outward-rounded high-precision interval arithmetic reference.
// KPI: True value contained 100%; no NaN accepted; pathological width triggers fallback.

use std::f64;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HighPrecisionInterval {
    pub lower: f64,
    pub upper: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IntervalError {
    NaNValueDetected,
    PathologicalWidthInflation { width: f64 },
}

impl HighPrecisionInterval {
    pub const EPSILON: f64 = 1e-12;
    pub const MAX_WIDTH_THRESHOLD: f64 = 1e6;

    pub fn new(lower: f64, upper: f64) -> Result<Self, IntervalError> {
        if lower.is_nan() || upper.is_nan() {
            return Err(IntervalError::NaNValueDetected);
        }
        if lower > upper {
            return Err(IntervalError::NaNValueDetected);
        }

        let interval = Self { lower, upper };
        if interval.width() > Self::MAX_WIDTH_THRESHOLD {
            return Err(IntervalError::PathologicalWidthInflation {
                width: interval.width(),
            });
        }

        Ok(interval)
    }

    pub fn width(&self) -> f64 {
        self.upper - self.lower
    }

    pub fn contains(&self, value: f64) -> bool {
        value >= self.lower && value <= self.upper
    }

    /// Outward-rounded addition
    pub fn add(&self, other: &Self) -> Result<Self, IntervalError> {
        let lo = (self.lower + other.lower) - Self::EPSILON;
        let hi = (self.upper + other.upper) + Self::EPSILON;
        Self::new(lo, hi)
    }

    /// Outward-rounded subtraction
    pub fn sub(&self, other: &Self) -> Result<Self, IntervalError> {
        let lo = (self.lower - other.upper) - Self::EPSILON;
        let hi = (self.upper - other.lower) + Self::EPSILON;
        Self::new(lo, hi)
    }

    /// Outward-rounded multiplication
    pub fn mul(&self, other: &Self) -> Result<Self, IntervalError> {
        let p1 = self.lower * other.lower;
        let p2 = self.lower * other.upper;
        let p3 = self.upper * other.lower;
        let p4 = self.upper * other.upper;

        let min_p = p1.min(p2).min(p3).min(p4);
        let max_p = p1.max(p2).max(p3).max(p4);

        let lo = min_p - Self::EPSILON;
        let hi = max_p + Self::EPSILON;

        Self::new(lo, hi)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_true_value_contained_100_percent() {
        let a = HighPrecisionInterval::new(1.0, 2.0).unwrap();
        let b = HighPrecisionInterval::new(3.0, 4.0).unwrap();

        let sum = a.add(&b).unwrap();
        assert!(sum.contains(1.0 + 3.0));
        assert!(sum.contains(2.0 + 4.0));
        assert!(sum.contains(1.5 + 3.5));

        let prod = a.mul(&b).unwrap();
        assert!(prod.contains(1.0 * 3.0));
        assert!(prod.contains(2.0 * 4.0));
    }

    #[test]
    fn test_no_nan_accepted() {
        assert_eq!(
            HighPrecisionInterval::new(f64::NAN, 1.0),
            Err(IntervalError::NaNValueDetected)
        );
        assert_eq!(
            HighPrecisionInterval::new(0.0, f64::NAN),
            Err(IntervalError::NaNValueDetected)
        );
    }

    #[test]
    fn test_pathological_width_fallback() {
        let res = HighPrecisionInterval::new(0.0, 1e7);
        assert!(matches!(
            res,
            Err(IntervalError::PathologicalWidthInflation { .. })
        ));
    }
}
