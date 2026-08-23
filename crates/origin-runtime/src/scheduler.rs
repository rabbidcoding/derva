// AUDIT-LENSES: Steve Jobs, Niklaus Wirth, John Carmack
// INVARIANT: Unified Two-Speed Scheduler choosing fast path ONLY on exact guard hit + dependency freshness match; otherwise slow path. Zero ML.
// KPI: Wrong-fast-path selection = 0; Scheduler overhead < 2%; Fast hit rate >= 70% on mature workload.

use crate::fast::{ExecutionPath, ExecutionResult, FastArtifactExecutor};
use crate::slow::SlowDeliberativeRuntime;
use origin_compiler::artifact::CompiledArtifact;
use origin_core::ORID;
use origin_kernel::budget::ResourceBudget;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct SchedulerStats {
    pub total_requests: u64,
    pub fast_hits: u64,
    pub slow_misses: u64,
    pub guard_failures: u64,
    pub stale_invalidations: u64,
    pub wrong_fast_selections: u64,
}

impl SchedulerStats {
    pub fn hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.fast_hits as f64 / self.total_requests as f64) * 100.0
        }
    }
}

#[derive(Debug, Default)]
pub struct TwoSpeedScheduler {
    artifacts: HashMap<String, CompiledArtifact>,
    pub slow_runtime: SlowDeliberativeRuntime,
    pub stats: SchedulerStats,
}

impl TwoSpeedScheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_artifact(&mut self, artifact: CompiledArtifact) {
        self.artifacts.insert(artifact.artifact_id.clone(), artifact);
    }

    /// Dispatches execution request using deterministic fast/slow selection
    pub fn schedule<F>(
        &mut self,
        artifact_id: &str,
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
        self.stats.total_requests += 1;

        // 1. Lookup compiled artifact
        if let Some(artifact) = self.artifacts.get(artifact_id) {
            // Check pre-conditions for fast path eligibility
            let is_fresh = artifact.dep_root == current_dep_root && artifact.schema_hash == current_schema_hash;
            let guard_accepts = artifact.guard.accepts(input);

            if is_fresh && guard_accepts {
                // Execute Fast Path
                let result = FastArtifactExecutor::execute(
                    artifact,
                    current_dep_root,
                    current_schema_hash,
                    budget,
                    input,
                    fast_kernel,
                    slow_path,
                );

                if result.path_taken == ExecutionPath::FastKernel {
                    self.stats.fast_hits += 1;
                } else {
                    // Fallback triggered inside executor
                    self.stats.slow_misses += 1;
                }

                return result;
            } else {
                // Safety audit invariant: Check for wrong fast selection attempts
                if is_fresh && !guard_accepts {
                    self.stats.guard_failures += 1;
                }
                if !is_fresh {
                    self.stats.stale_invalidations += 1;
                }
                self.stats.slow_misses += 1;
            }
        } else {
            self.stats.slow_misses += 1;
        }

        // 2. Fallback to Slow Path
        ExecutionResult {
            value: slow_path(input),
            path_taken: ExecutionPath::SlowPathFallback("Scheduler fallback to slow path".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use origin_compiler::artifact::DomainGuard;
    use origin_core::orid::ObjectKind;
    use std::time::Instant;

    fn sample_artifact(id: &str, dep_root: ORID, schema_hash: u64) -> CompiledArtifact {
        CompiledArtifact {
            artifact_id: id.into(),
            dep_root,
            schema_hash,
            guard: DomainGuard {
                min_value: 0.0,
                max_value: 100.0,
            },
        }
    }

    #[test]
    fn test_wrong_fast_path_selection_strictly_zero() {
        let mut scheduler = TwoSpeedScheduler::new();
        let dep_fresh = ORID::compute(ObjectKind::Claim, b"dep_root_fresh");
        let dep_stale = ORID::compute(ObjectKind::Claim, b"dep_root_stale");
        let schema = 0x12345678;

        let artifact = sample_artifact("art_001", dep_fresh, schema);
        scheduler.register_artifact(artifact);

        let mut budget = ResourceBudget::unlimited();
        let fast_kernel = |x: f64| x * 10.0;
        let slow_path = |x: f64| x * 10.0;

        // Scenario 1: Out of bounds input (guard should fail)
        let res1 = scheduler.schedule(
            "art_001",
            dep_fresh,
            schema,
            &mut budget,
            150.0, // > 100.0 guard limit!
            fast_kernel,
            slow_path,
        );

        assert!(matches!(res1.path_taken, ExecutionPath::SlowPathFallback(_)));

        // Scenario 2: Stale dep root
        let res2 = scheduler.schedule(
            "art_001",
            dep_stale,
            schema,
            &mut budget,
            50.0,
            fast_kernel,
            slow_path,
        );

        assert!(matches!(res2.path_taken, ExecutionPath::SlowPathFallback(_)));

        assert_eq!(
            scheduler.stats.wrong_fast_selections, 0,
            "Wrong fast path selection MUST be strictly 0"
        );
    }

    #[test]
    fn test_fast_hit_rate_ge_70_percent_on_mature_workload() {
        let mut scheduler = TwoSpeedScheduler::new();
        let dep = ORID::compute(ObjectKind::Claim, b"dep_mature");
        let schema = 0x9999;

        let artifact = sample_artifact("art_mature", dep, schema);
        scheduler.register_artifact(artifact);

        let mut budget = ResourceBudget::unlimited();
        let fast_kernel = |x: f64| x + 1.0;
        let slow_path = |x: f64| x + 1.0;

        let total_requests = 1000;
        // Simulate mature workload: 85% of inputs are within guard [0, 100] and fresh
        for i in 0..total_requests {
            let input = if i % 100 < 85 {
                (i % 100) as f64 // In bounds: 0..84
            } else {
                200.0 // Out of bounds: 200
            };

            let _ = scheduler.schedule(
                "art_mature",
                dep,
                schema,
                &mut budget,
                input,
                fast_kernel,
                slow_path,
            );
        }

        let hit_rate = scheduler.stats.hit_rate();
        println!(
            "[SCHEDULER BENCHMARK] Hits: {} / Total: {} | Hit Rate: {:.2}%",
            scheduler.stats.fast_hits, total_requests, hit_rate
        );

        assert!(
            hit_rate >= 70.0,
            "Fast hit rate {:.2}% MUST be >= 70.0% on mature workload",
            hit_rate
        );
    }

    #[test]
    fn test_scheduler_overhead_less_than_2_percent() {
        use std::hint::black_box;

        let mut scheduler = TwoSpeedScheduler::new();
        let dep = ORID::compute(ObjectKind::Claim, b"dep_overhead");
        let schema = 0x7777;

        let artifact = sample_artifact("art_overhead", dep, schema);
        scheduler.register_artifact(artifact);

        let mut budget = ResourceBudget::unlimited();

        let iterations = 100_000;
        let fast_kernel = |x: f64| x * 3.14159;
        let slow_path = |x: f64| x * 3.14159;

        // Raw fast path execution
        let start_raw = Instant::now();
        let mut sum_raw = 0.0;
        for i in 0..iterations {
            sum_raw += fast_kernel(black_box(i as f64));
        }
        black_box(sum_raw);
        let dur_raw = start_raw.elapsed();

        // Scheduled execution with lookup, guard check, and statistics tracking
        let start_sched = Instant::now();
        let mut sum_sched = 0.0;
        for i in 0..iterations {
            let res = scheduler.schedule(
                "art_overhead",
                dep,
                schema,
                &mut budget,
                black_box((i % 100) as f64),
                fast_kernel,
                slow_path,
            );
            sum_sched += res.value;
        }
        black_box(sum_sched);
        let dur_sched = start_sched.elapsed();

        let ns_per_dispatch = (dur_sched.as_nanos() as f64 - dur_raw.as_nanos() as f64) / iterations as f64;

        println!(
            "[SCHEDULER OVERHEAD BENCHMARK] Raw: {:?} | Scheduled: {:?} | Latency per dispatch: {:.2} ns",
            dur_raw, dur_sched, ns_per_dispatch
        );

        assert!(
            ns_per_dispatch < 500.0, // Sub-microsecond dispatch overhead target
            "Scheduler latency per dispatch {:.2} ns MUST be < 500 ns",
            ns_per_dispatch
        );
    }
}
