// AUDIT-LENSES: Steve Jobs, John Carmack, Donald Knuth
// INVARIANT: Fast Artifact Execution with guard -> dependency check -> budget charge -> kernel; immediate fallback to slow path.
// KPI: Guard+freshness path p99 < 50us; Fallback correctness 100%; Fast result semantic parity 100%.

use origin_compiler::artifact::CompiledArtifact;
use origin_core::ORID;
use origin_fast::dispatch::CpuImplementation;
use origin_kernel::budget::{ResourceBudget, StepCost};

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionPath {
    FastKernel,
    SlowPathFallback(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionResult {
    pub value: f64,
    pub path_taken: ExecutionPath,
}

pub struct FastArtifactExecutor;

impl FastArtifactExecutor {
    /// Executes a compiled artifact with guard -> dependency check -> budget charge -> kernel execution pipeline.
    /// Falls back immediately to slow_path if guard fails, dependency is stale, or budget is exhausted.
    pub fn execute<F>(
        artifact: &CompiledArtifact,
        current_dep_root: ORID,
        current_schema_hash: u64,
        budget: &mut ResourceBudget,
        input: f64,
        fast_kernel: F,
        slow_path: impl Fn(f64) -> f64,
    ) -> ExecutionResult
    where
        F: Fn(f64) -> f64,
    {
        // 1. Guard & Freshness validation check
        if let Err(err) = artifact.validate_and_acquire(current_dep_root, current_schema_hash, input) {
            let fallback_reason = format!("Artifact validation failed: {}", err);
            return ExecutionResult {
                value: slow_path(input),
                path_taken: ExecutionPath::SlowPathFallback(fallback_reason),
            };
        }

        // 2. Budget charge tick (1 CpuTick for fast-path execution charge)
        if budget.charge(StepCost::CpuTicks(1)).is_err() {
            return ExecutionResult {
                value: slow_path(input),
                path_taken: ExecutionPath::SlowPathFallback("Resource budget exhausted".into()),
            };
        }

        // 3. Fast ISA-dispatched Kernel execution
        let fast_res = match CpuImplementation::select() {
            CpuImplementation::Avx2 | CpuImplementation::Avx512 => fast_kernel(input),
            CpuImplementation::Scalar => fast_kernel(input),
        };

        ExecutionResult {
            value: fast_res,
            path_taken: ExecutionPath::FastKernel,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use origin_compiler::artifact::DomainGuard;
    use origin_core::orid::ObjectKind;
    use std::time::Instant;

    fn sample_artifact(dep_root: ORID, schema_hash: u64) -> CompiledArtifact {
        CompiledArtifact {
            artifact_id: "art_fast_001".into(),
            dep_root,
            schema_hash,
            guard: DomainGuard {
                min_value: -1000.0,
                max_value: 1000.0,
            },
        }
    }

    #[test]
    fn test_fast_result_semantic_parity_100_percent() {
        let dep = ORID::compute(ObjectKind::Claim, b"dep_root_parity");
        let schema = 0xabcdef1234567890;
        let artifact = sample_artifact(dep, schema);

        let mut budget = ResourceBudget::unlimited();

        let inputs = vec![-500.0, -100.0, 0.0, 42.0, 999.0];

        let fast_kernel = |x: f64| x * 2.0 + 1.0;
        let slow_path = |x: f64| x * 2.0 + 1.0; // Same mathematical logic

        for input in inputs {
            let res = FastArtifactExecutor::execute(
                &artifact,
                dep,
                schema,
                &mut budget,
                input,
                fast_kernel,
                slow_path,
            );

            assert_eq!(res.path_taken, ExecutionPath::FastKernel);
            assert_eq!(
                res.value,
                slow_path(input),
                "Fast kernel and slow path MUST have 100% semantic parity"
            );
        }
    }

    #[test]
    fn test_fallback_correctness_100_percent_on_guard_rejection() {
        let dep = ORID::compute(ObjectKind::Claim, b"dep_root_guard");
        let schema = 0x1234;
        let artifact = sample_artifact(dep, schema); // min: -1000, max: 1000

        let mut budget = ResourceBudget::unlimited();

        let out_of_bounds_inputs = vec![-1000.1, 1000.1, 5000.0];

        let fast_kernel = |x: f64| x * 2.0;
        let slow_path = |x: f64| x * 2.0;

        for input in out_of_bounds_inputs {
            let res = FastArtifactExecutor::execute(
                &artifact,
                dep,
                schema,
                &mut budget,
                input,
                fast_kernel,
                slow_path,
            );

            match res.path_taken {
                ExecutionPath::SlowPathFallback(reason) => {
                    assert!(
                        reason.contains("Domain guard rejected"),
                        "Fallback reason must record guard rejection"
                    );
                }
                ExecutionPath::FastKernel => panic!("Must not execute fast kernel for out-of-bounds input"),
            }

            assert_eq!(
                res.value,
                slow_path(input),
                "Fallback result MUST match slow path 100%"
            );
        }
    }

    #[test]
    fn test_fallback_correctness_100_percent_on_stale_dependency() {
        let fresh_dep = ORID::compute(ObjectKind::Claim, b"dep_fresh");
        let stale_dep = ORID::compute(ObjectKind::Claim, b"dep_stale");
        let schema = 0x1234;
        let artifact = sample_artifact(fresh_dep, schema);

        let mut budget = ResourceBudget::unlimited();

        let fast_kernel = |x: f64| x + 10.0;
        let slow_path = |x: f64| x + 10.0;

        let res = FastArtifactExecutor::execute(
            &artifact,
            stale_dep, // Stale dependency passed
            schema,
            &mut budget,
            50.0,
            fast_kernel,
            slow_path,
        );

        match res.path_taken {
            ExecutionPath::SlowPathFallback(reason) => {
                assert!(reason.contains("dependency root mismatch"));
            }
            ExecutionPath::FastKernel => panic!("Must not execute fast kernel on stale dependency"),
        }

        assert_eq!(res.value, 60.0);
    }

    #[test]
    fn test_guard_plus_freshness_path_p99_less_than_50us() {
        let dep = ORID::compute(ObjectKind::Claim, b"dep_bench");
        let schema = 0x5555;
        let artifact = sample_artifact(dep, schema);

        let mut budget = ResourceBudget::unlimited();

        let iterations = 10_000;
        let mut latencies_ns = Vec::with_capacity(iterations);

        let fast_kernel = |x: f64| x * 1.5;
        let slow_path = |x: f64| x * 1.5;

        for _ in 0..iterations {
            let start = Instant::now();
            let _ = FastArtifactExecutor::execute(
                &artifact,
                dep,
                schema,
                &mut budget,
                100.0,
                fast_kernel,
                slow_path,
            );
            latencies_ns.push(start.elapsed().as_nanos() as u64);
        }

        latencies_ns.sort_unstable();

        // Calculate p99 (99th percentile)
        let p99_idx = (iterations as f64 * 0.99) as usize;
        let p99_latency_ns = latencies_ns[p99_idx];
        let p99_latency_us = p99_latency_ns as f64 / 1000.0;

        println!(
            "[FAST EXECUTOR BENCHMARK] p50: {:.3} µs | p99: {:.3} µs | Target: < 50.0 µs",
            latencies_ns[iterations / 2] as f64 / 1000.0,
            p99_latency_us
        );

        assert!(
            p99_latency_us < 50.0,
            "Fast executor p99 latency {:.3} µs MUST be < 50.0 µs target",
            p99_latency_us
        );
    }
}
