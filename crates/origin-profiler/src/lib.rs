#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO — Profiler Subsystem
// INVARIANT: Zero-overhead profiling, RAII spans, assembly prerequisite validation.

pub mod profiler;

pub use profiler::{
    assert_assembly_task_has_profile_artifact, clear_spans, get_collected_spans,
    is_profiler_enabled, set_profiler_enabled, span, ProfileSpan, SpanMetrics,
};

pub fn crate_name() -> &'static str {
    "origin-profiler"
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_profiler_crate_boundary() {
        assert_eq!(super::crate_name(), "origin-profiler");
    }
}
