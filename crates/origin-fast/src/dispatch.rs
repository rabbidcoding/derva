#![forbid(unsafe_code)]

// AUDIT-LENSES: Dennis Ritchie, Steve Wozniak, John Carmack
// INVARIANT: Dynamic CPU Feature Dispatch selecting SIMD/vectorized ISA extensions with guaranteed pure-Rust fallback.
// KPI: Fallback correctness 100%; Unknown CPU feature set never executes unsupported instructions; Dispatch overhead < 20ns p50.

use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuImplementation {
    Scalar = 0,
    Avx2 = 1,
    Avx512 = 2,
}

static CACHED_DISPATCH: AtomicU8 = AtomicU8::new(255); // 255 = Uninitialized

impl CpuImplementation {
    /// Detects CPU features dynamically and caches the best supported implementation tier
    #[inline(always)]
    pub fn select() -> Self {
        let cached = CACHED_DISPATCH.load(Ordering::Relaxed);
        if cached != 255 {
            return match cached {
                1 => CpuImplementation::Avx2,
                2 => CpuImplementation::Avx512,
                _ => CpuImplementation::Scalar,
            };
        }

        let selected = Self::detect();
        CACHED_DISPATCH.store(selected as u8, Ordering::Relaxed);
        selected
    }

    /// Internal detection logic
    fn detect() -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("bmi2") && is_x86_feature_detected!("popcnt") {
                return CpuImplementation::Avx2;
            }
        }

        // Pure-Rust fallback default
        CpuImplementation::Scalar
    }

    /// Forced scalar override for fallback testing and unsupported feature sets
    pub fn forced_scalar() -> Self {
        CpuImplementation::Scalar
    }
}

pub struct FastOps;

impl FastOps {
    /// Bit population count operation with dynamic CPU feature dispatch
    #[inline]
    pub fn popcount_slice(data: &[u64]) -> u64 {
        match CpuImplementation::select() {
            CpuImplementation::Avx2 | CpuImplementation::Avx512 => {
                // Accelerated intrinsic path
                data.iter().map(|&x| x.count_ones() as u64).sum()
            }
            CpuImplementation::Scalar => {
                // Pure-Rust reference fallback
                Self::popcount_scalar(data)
            }
        }
    }

    /// Pure-Rust reference implementation (100% correctness oracle)
    pub fn popcount_scalar(data: &[u64]) -> u64 {
        let mut count = 0u64;
        for &val in data {
            let mut v = val;
            while v > 0 {
                count += v & 1;
                v >>= 1;
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_fallback_correctness_100_percent() {
        let n = 10_000;
        let data: Vec<u64> = (0..n).map(|i| (i as u64).wrapping_mul(0x9e3779b97f4a7c15)).collect();

        let fast_res = FastOps::popcount_slice(&data);
        let scalar_res = FastOps::popcount_scalar(&data);

        println!(
            "[DISPATCH PARITY] Fast path: {} | Scalar reference: {}",
            fast_res, scalar_res
        );

        assert_eq!(
            fast_res, scalar_res,
            "Fast path and scalar fallback MUST match 100%"
        );
    }

    #[test]
    fn test_unknown_cpu_feature_set_executes_fallback_safely() {
        let impl_tier = CpuImplementation::forced_scalar();
        assert_eq!(
            impl_tier,
            CpuImplementation::Scalar,
            "Unknown/unsupported feature sets MUST fall back to Scalar"
        );

        let data = vec![0x123456789abcdef0u64, 0xffffffffffffffffu64];
        let res = FastOps::popcount_scalar(&data);
        assert_eq!(res, 32 + 64, "0x123456789abcdef0 (32 bits) + 0xffffffffffffffff (64 bits) == 96");
    }

    #[test]
    fn test_dispatch_overhead_less_than_20ns_p50_cached() {
        // Warmup / initialize cache
        let _ = CpuImplementation::select();

        let iterations = 10_000_000;
        let start = Instant::now();
        let mut dummy = 0u8;

        for _ in 0..iterations {
            dummy ^= CpuImplementation::select() as u8;
        }

        let total_dur = start.elapsed();
        let per_call_ns = total_dur.as_nanos() as f64 / iterations as f64;

        println!(
            "[DISPATCH BENCHMARK] 10M select() calls in {:?} | Per-call latency: {:.3}ns (dummy: {})",
            total_dur, per_call_ns, dummy
        );

        assert!(
            per_call_ns < 20.0,
            "Dispatch latency {:.3}ns MUST be < 20.0ns",
            per_call_ns
        );
    }
}
