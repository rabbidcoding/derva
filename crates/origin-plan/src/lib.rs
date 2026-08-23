#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO Subsystem: origin-plan
// Deterministic Causal Planning & State Search Engine.

pub mod ao;
pub mod astar;
pub mod ida;
pub mod query;
pub mod select;

pub use ao::{AndOrBranch, AndOrPlanDomain, AoStarPlanner, HyperNodeKind};
pub use astar::{AStarPlanner, AdmissibleHeuristic, PlanDomain, PlanError, PlanNode, PlanResult};
pub use ida::IdaStarPlanner;
pub use query::{
    EpistemicQuery, EpistemicQueryPlanner, QueryBudget, QueryExecutionRecord, QueryOracle,
    QueryPlanError, WorldStateHypothesis,
};
pub use select::{MemoryPressure, PlannerKind, PlannerSelector, ProblemSignature};

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
