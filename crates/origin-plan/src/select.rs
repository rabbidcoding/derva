#![forbid(unsafe_code)]

// INVARIANT: Planner selection 100% deterministic based on ProblemSignature; 0 ML weights.
// KPI: 100% deterministic planner routing.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryPressure {
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProblemSignature {
    pub state_space_estimate: u64,
    pub memory_pressure: MemoryPressure,
    pub has_and_or_subgoals: bool,
    pub max_depth_estimate: usize,
}

impl Default for ProblemSignature {
    fn default() -> Self {
        Self {
            state_space_estimate: 1000,
            memory_pressure: MemoryPressure::Normal,
            has_and_or_subgoals: false,
            max_depth_estimate: 50,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlannerKind {
    AStar,
    IdaStar,
    AoStar,
}

// AUDIT-LENSES: Wozniak, Stroustrup, Knuth
#[derive(Debug, Clone, Default)]
pub struct PlannerSelector;

impl PlannerSelector {
    pub fn new() -> Self {
        Self
    }

    /// Pure deterministic planner selection based on problem signature.
    pub fn select_planner(&self, sig: &ProblemSignature) -> PlannerKind {
        if sig.has_and_or_subgoals {
            PlannerKind::AoStar
        } else if matches!(
            sig.memory_pressure,
            MemoryPressure::High | MemoryPressure::Critical
        ) || sig.state_space_estimate > 100_000
        {
            PlannerKind::IdaStar
        } else {
            PlannerKind::AStar
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planner_selector_is_100_percent_deterministic() {
        let selector = PlannerSelector::new();

        let sig_high_mem = ProblemSignature {
            memory_pressure: MemoryPressure::High,
            ..Default::default()
        };
        assert_eq!(selector.select_planner(&sig_high_mem), PlannerKind::IdaStar);

        let sig_and_or = ProblemSignature {
            has_and_or_subgoals: true,
            ..Default::default()
        };
        assert_eq!(selector.select_planner(&sig_and_or), PlannerKind::AoStar);

        let sig_normal = ProblemSignature::default();
        assert_eq!(selector.select_planner(&sig_normal), PlannerKind::AStar);
    }
}
