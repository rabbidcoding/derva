#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO — Runtime Kernel Orchestrator, Fast Execution & Slow Deliberative Subsystem
// INVARIANT: Pure-Rust safe execution with mandatory fallback to slow path; deterministic event logging for deliberation.

pub mod fast;
pub mod slow;

pub use fast::{ExecutionPath, ExecutionResult, FastArtifactExecutor};
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
