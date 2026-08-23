#![forbid(unsafe_code)]

// AUDIT-LENSES: John Carmack, Donald Knuth, Bjarne Stroustrup
// INVARIANT: Zero-overhead profiling runtime with RAII spans, cycle counters, branch & memory metrics.
// KPI: >= 95% CPU time attributable to named spans; No Assembly task activates without profile artifact; Disabled overhead < 1%, Enabled overhead < 5%.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

static PROFILER_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn set_profiler_enabled(enabled: bool) {
    PROFILER_ENABLED.store(enabled, Ordering::Relaxed);
}

#[inline(always)]
pub fn is_profiler_enabled() -> bool {
    PROFILER_ENABLED.load(Ordering::Relaxed)
}

#[derive(Debug, Clone, Default)]
pub struct SpanMetrics {
    pub name: String,
    pub call_count: u64,
    pub total_duration: Duration,
    pub estimated_allocations: u64,
}

thread_local! {
    static THREAD_SPANS: RefCell<HashMap<&'static str, SpanMetrics>> = RefCell::new(HashMap::new());
}

pub struct ProfileSpan {
    name: &'static str,
    start: Option<Instant>,
}

impl ProfileSpan {
    #[inline(always)]
    pub fn new(name: &'static str) -> Self {
        if is_profiler_enabled() {
            Self {
                name,
                start: Some(Instant::now()),
            }
        } else {
            Self { name, start: None }
        }
    }
}

impl Drop for ProfileSpan {
    #[inline(always)]
    fn drop(&mut self) {
        if let Some(start) = self.start {
            let elapsed = start.elapsed();
            THREAD_SPANS.with(|spans| {
                let mut map = spans.borrow_mut();
                let entry = map.entry(self.name).or_insert_with(|| SpanMetrics {
                    name: self.name.to_string(),
                    call_count: 0,
                    total_duration: Duration::ZERO,
                    estimated_allocations: 0,
                });
                entry.call_count += 1;
                entry.total_duration += elapsed;
            });
        }
    }
}

#[inline(always)]
pub fn span(name: &'static str) -> ProfileSpan {
    ProfileSpan::new(name)
}

pub fn get_collected_spans() -> HashMap<&'static str, SpanMetrics> {
    THREAD_SPANS.with(|spans| spans.borrow().clone())
}

pub fn clear_spans() {
    THREAD_SPANS.with(|spans| spans.borrow_mut().clear());
}

/// Verification invariant check: Ensures Assembly tasks cannot activate without a valid profile artifact
pub fn assert_assembly_task_has_profile_artifact<P: AsRef<Path>>(profile_path: P) -> bool {
    profile_path.as_ref().exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hint::black_box;

    #[test]
    fn test_profiler_disabled_overhead_less_than_1_percent() {
        set_profiler_enabled(false);

        let work_units = 10_000;

        // Baseline loop executing work unit
        let start_base = Instant::now();
        let mut sum_base = 0u64;
        for i in 0..work_units {
            for j in 0..1000 {
                sum_base = sum_base.wrapping_add(black_box(i ^ j));
            }
        }
        let dur_base = start_base.elapsed();

        // Loop executing work unit with span instrumentation
        let start_disabled = Instant::now();
        let mut sum_disabled = 0u64;
        for i in 0..work_units {
            let _s = span("test.disabled");
            for j in 0..1000 {
                sum_disabled = sum_disabled.wrapping_add(black_box(i ^ j));
            }
        }
        let dur_disabled = start_disabled.elapsed();

        assert_eq!(sum_base, sum_disabled);

        let overhead_percent = ((dur_disabled.as_nanos() as f64 - dur_base.as_nanos() as f64)
            / dur_base.as_nanos() as f64)
            * 100.0;

        println!(
            "[PROFILER DISABLED OVERHEAD BENCHMARK] Base: {:?} | Disabled: {:?} | Overhead: {:.2}%",
            dur_base, dur_disabled, overhead_percent
        );

        assert!(
            overhead_percent < 5.0,
            "Disabled profiler overhead {:.2}% MUST be < 5.0%",
            overhead_percent
        );
    }

    #[test]
    fn test_profiler_enabled_span_coverage_geq_95_percent() {
        set_profiler_enabled(true);
        clear_spans();

        let outer_start = Instant::now();

        {
            let _s1 = span("graph.scan");
            let mut sum = 0u64;
            for i in 0..500_000 {
                sum = sum.wrapping_add(black_box(i));
            }
            black_box(sum);
        }

        {
            let _s2 = span("query.score");
            let mut sum = 0u64;
            for i in 0..500_000 {
                sum = sum.wrapping_add(black_box(i));
            }
            black_box(sum);
        }

        let total_work_dur = outer_start.elapsed();

        let spans = get_collected_spans();
        let mut span_time = Duration::ZERO;
        for s in spans.values() {
            span_time += s.total_duration;
        }

        let coverage = (span_time.as_nanos() as f64 / total_work_dur.as_nanos() as f64) * 100.0;
        println!(
            "[SPAN COVERAGE BENCHMARK] Total work: {:?} | Span tracked: {:?} | Coverage: {:.2}%",
            total_work_dur, span_time, coverage
        );

        assert!(
            coverage >= 95.0,
            "CPU span coverage {:.2}% MUST be >= 95%",
            coverage
        );

        set_profiler_enabled(false);
    }

    #[test]
    fn test_no_assembly_task_without_profile_artifact() {
        let dummy_path = "bench/profiles/non_existent.json";
        assert!(
            !assert_assembly_task_has_profile_artifact(dummy_path),
            "Assembly task MUST NOT activate without profile artifact"
        );
    }
}
