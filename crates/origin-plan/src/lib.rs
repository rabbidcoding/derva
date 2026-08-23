#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO Subsystem: origin-plan
// Deterministic Causal Planning & State Search Engine.

pub mod astar;

pub use astar::{AStarPlanner, AdmissibleHeuristic, PlanDomain, PlanError, PlanNode, PlanResult};

pub fn crate_name() -> &'static str {
    "origin-plan"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_boundary() {
        assert_eq!(crate_name(), "origin-plan");
    }
}
