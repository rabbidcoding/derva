#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO — Runtime Kernel Orchestrator & Fast Artifact Execution Engine
// INVARIANT: Pure-Rust safe execution with mandatory fallback to slow path on guard rejection or stale dependencies.

pub mod fast;

pub use fast::{ExecutionPath, ExecutionResult, FastArtifactExecutor};

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
