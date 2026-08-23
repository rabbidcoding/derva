#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO — Subsystem: origin-chaos
// INVARIANT: Crash & Power-Loss Fault Injection Framework; 100% recovery to last committed root; 0 invalid published roots.

pub mod campaign;
pub mod injector;
pub mod matrix;

pub use campaign::{ChaosCampaign, ChaosReport};
pub use injector::{ChaosStore, CrashInjectionPoint, FaultInjector, FaultType};
pub use matrix::CrashMatrixConfig;

pub fn crate_name() -> &'static str {
    "origin-chaos"
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_chaos_crate_boundary() {
        assert_eq!(super::crate_name(), "origin-chaos");
    }
}
