#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO — Runtime Kernel Orchestrator, Two-Speed Scheduler & Deliberative Subsystem
// INVARIANT: Pure-Rust safe execution with mandatory fallback to slow path; deterministic event logging; 0 ML scheduler.

pub mod fast;
pub mod scheduler;
pub mod slow;

pub use fast::{ExecutionPath, ExecutionResult, FastArtifactExecutor};
pub use scheduler::{SchedulerStats, TwoSpeedScheduler};
pub use slow::{DeliberationError, EventRecord, Proposal, SlowDeliberativeRuntime, StepKind, VerifiedAction};

pub fn crate_name() -> &'static str {
    "origin-runtime"
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_runtime_crate_boundary() {
        assert_eq!(super::crate_name(), "origin-runtime");
    }
}
