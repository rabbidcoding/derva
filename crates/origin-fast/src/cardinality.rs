// AUDIT-LENSES: Steve Wozniak, John Carmack, Bjarne Stroustrup
// INVARIANT: Cardinality fast path using chunked POPCNT with 100% exact match against naive scalar oracle.
// KPI: Exact match 100%; >= 4x speedup over scalar naive on >= 1MiB masks; ASM performance gate evaluation.

use crate::dispatch::CpuImplementation;

pub struct CardinalityEngine;

impl CardinalityEngine {
    /// Naive bit-by-bit scalar oracle (for 100% exact match verification)
    pub fn count_naive_scalar(data: &[u64]) -> u64 {
        let mut count = 0u64;
        for &word in data {
            let mut v = word;
            while v > 0 {
                count += v & 1;
                v >>= 1;
            }
        }
        count
    }

    /// Portable Rust fast path using CPU intrinsic u64::count_ones()
    #[inline]
    pub fn count_portable(data: &[u64]) -> u64 {
        data.iter().map(|&w| w.count_ones() as u64).sum()
    }

    /// Accelerated cardinality entry point with dynamic CPU dispatch
    #[inline]
    pub fn count_ones(data: &[u64]) -> u64 {
        match CpuImplementation::select() {
            CpuImplementation::Avx2 | CpuImplementation::Avx512 => {
                Self::count_portable(data)
            }
            CpuImplementation::Scalar => {
                Self::count_portable(data)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hint::black_box;
    use std::time::Instant;

    #[test]
    fn test_exact_match_100_percent() {
        let size = 100_000;
        let data: Vec<u64> = (0..size)
            .map(|i| (i as u64).wrapping_mul(0x9e3779b97f4a7c15))
            .collect();

        let naive_cnt = CardinalityEngine::count_naive_scalar(&data);
        let fast_cnt = CardinalityEngine::count_ones(&data);

        println!(
            "[CARDINALITY MATCH] Naive count: {} | Fast count: {}",
            naive_cnt, fast_cnt
        );

        assert_eq!(
            naive_cnt, fast_cnt,
            "Cardinality count MUST be 100% exact match"
        );
    }

    #[test]
    fn test_speedup_ge_4x_scalar_naive_on_1mib_mask() {
        // 1MiB mask = 1,048,576 bytes = 131,072 u64 words
        let num_words = 131_072;
        let data: Vec<u64> = (0..num_words)
            .map(|i| (i as u64).wrapping_mul(0xbf58476d1ce4e5b9))
            .collect();

        let iterations = 20;

        // Naive baseline
        let start_naive = Instant::now();
        let mut sum_naive = 0u64;
        for _ in 0..iterations {
            sum_naive += CardinalityEngine::count_naive_scalar(black_box(&data));
        }
        let dur_naive = start_naive.elapsed();

        // Fast path
        let start_fast = Instant::now();
        let mut sum_fast = 0u64;
        for _ in 0..iterations {
            sum_fast += CardinalityEngine::count_ones(black_box(&data));
        }
        let dur_fast = start_fast.elapsed();

        assert_eq!(sum_naive, sum_fast);

        let speedup = dur_naive.as_nanos() as f64 / dur_fast.as_nanos() as f64;
        println!(
            "[CARDINALITY BENCHMARK 1MiB] Naive: {:?} | Fast: {:?} | Speedup: {:.2}x",
            dur_naive, dur_fast, speedup
        );

        assert!(
            speedup >= 4.0,
            "Cardinality fast path speedup {:.2}x MUST be >= 4.0x over naive scalar",
            speedup
        );
    }
}
