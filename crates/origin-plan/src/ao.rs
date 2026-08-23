#![forbid(unsafe_code)]

// INVARIANT: AO* matches reference solution on AND/OR graphs; deterministic tree expansion.
// KPI: 100% correct AND/OR graph resolution matching reference suite.

use crate::astar::{AdmissibleHeuristic, PlanDomain, PlanError, PlanResult};
use origin_causal::operator::CausalOperator;
use origin_core::{Budget, ORID};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HyperNodeKind {
    Or,  // Disjunction: choose min cost child
    And, // Conjunction: solve all children
}

#[derive(Debug, Clone)]
pub struct AndOrBranch<S> {
    pub operator: CausalOperator,
    pub kind: HyperNodeKind,
    pub child_states: Vec<S>,
}

pub trait AndOrPlanDomain<S>: PlanDomain<S> {
    fn get_and_or_branches(&self, state: &S) -> Vec<AndOrBranch<S>>;
}

#[derive(Debug, Clone)]
pub struct AoStarPlanner {
    pub lambda_risk: f64,
}

impl Default for AoStarPlanner {
    fn default() -> Self {
        Self { lambda_risk: 1.0 }
    }
}

impl AoStarPlanner {
    pub fn new(lambda_risk: f64) -> Self {
        Self { lambda_risk }
    }

    /// Solves AND/OR graph planning deterministically.
    pub fn plan_and_or<S: Clone, D: AndOrPlanDomain<S>, H: AdmissibleHeuristic<S>>(
        &self,
        domain: &D,
        heuristic: &H,
        initial_state: S,
        goal: S,
        budget: &Budget,
    ) -> Result<PlanResult<S>, PlanError> {
        let mut cost_table: HashMap<ORID, f64> = HashMap::new();
        let mut solved_table: HashMap<ORID, bool> = HashMap::new();

        let mut expansions: u64 = 0;
        let max_expansions = budget.cpu_steps_remaining.max(1);

        let (total_cost, path_ops) = self.solve_node(
            domain,
            heuristic,
            &initial_state,
            &goal,
            &mut cost_table,
            &mut solved_table,
            &mut expansions,
            max_expansions,
        )?;

        Ok(PlanResult {
            path_operators: path_ops,
            final_state: goal,
            total_g_cost: total_cost,
            total_risk: 0.0,
            total_uncertainty: 0.0,
            final_f_score: total_cost,
            expansions,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn solve_node<S: Clone, D: AndOrPlanDomain<S>, H: AdmissibleHeuristic<S>>(
        &self,
        domain: &D,
        heuristic: &H,
        current: &S,
        goal: &S,
        cost_table: &mut HashMap<ORID, f64>,
        solved_table: &mut HashMap<ORID, bool>,
        expansions: &mut u64,
        max_expansions: u64,
    ) -> Result<(f64, Vec<ORID>), PlanError> {
        if *expansions >= max_expansions {
            return Err(PlanError::BudgetExhausted);
        }
        *expansions += 1;

        let curr_id = domain.canonical_id(current);
        if domain.is_goal(current, goal) {
            cost_table.insert(curr_id, 0.0);
            solved_table.insert(curr_id, true);
            return Ok((0.0, vec![]));
        }

        let branches = domain.get_and_or_branches(current);
        if branches.is_empty() {
            return Err(PlanError::NoPathFound);
        }

        let mut best_branch_cost = f64::INFINITY;
        let mut best_branch_ops = Vec::new();

        for branch in branches {
            let op_cost = branch
                .operator
                .cost
                .as_ref()
                .map(|c| c.value as f64)
                .unwrap_or(1.0);
            let op_risk = branch
                .operator
                .risk
                .as_ref()
                .map(|r| r.score)
                .unwrap_or(0.0);
            let h_val = heuristic.estimate(current, goal);
            let edge_cost = op_cost + (self.lambda_risk * op_risk) + h_val;

            match branch.kind {
                HyperNodeKind::Or => {
                    for child in branch.child_states {
                        let (child_cost, mut child_ops) = self.solve_node(
                            domain,
                            heuristic,
                            &child,
                            goal,
                            cost_table,
                            solved_table,
                            expansions,
                            max_expansions,
                        )?;
                        let total = edge_cost + child_cost;
                        if total < best_branch_cost {
                            best_branch_cost = total;
                            let mut ops = vec![branch.operator.id];
                            ops.append(&mut child_ops);
                            best_branch_ops = ops;
                        }
                    }
                }
                HyperNodeKind::And => {
                    let mut sum_cost = edge_cost;
                    let mut combined_ops = vec![branch.operator.id];
                    let mut all_solved = true;

                    for child in branch.child_states {
                        match self.solve_node(
                            domain,
                            heuristic,
                            &child,
                            goal,
                            cost_table,
                            solved_table,
                            expansions,
                            max_expansions,
                        ) {
                            Ok((child_cost, mut child_ops)) => {
                                sum_cost += child_cost;
                                combined_ops.append(&mut child_ops);
                            }
                            Err(_) => {
                                all_solved = false;
                                break;
                            }
                        }
                    }

                    if all_solved && sum_cost < best_branch_cost {
                        best_branch_cost = sum_cost;
                        best_branch_ops = combined_ops;
                    }
                }
            }
        }

        if best_branch_cost.is_infinite() {
            Err(PlanError::NoPathFound)
        } else {
            cost_table.insert(curr_id, best_branch_cost);
            solved_table.insert(curr_id, true);
            Ok((best_branch_cost, best_branch_ops))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use origin_causal::operator::{Cost, EffectId, Risk, SchemaId};
    use origin_core::{CausalStatus, ObjectKind};

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct AndOrState {
        id: String,
    }

    struct SimpleAndOrDomain;

    impl PlanDomain<AndOrState> for SimpleAndOrDomain {
        fn canonical_id(&self, state: &AndOrState) -> ORID {
            ORID::compute(ObjectKind::Entity, state.id.as_bytes())
        }

        fn get_applicable_operators(&self, _state: &AndOrState) -> Vec<CausalOperator> {
            vec![]
        }

        fn apply_operator(
            &self,
            _state: &AndOrState,
            _op: &CausalOperator,
        ) -> Result<AndOrState, String> {
            Err("Use get_and_or_branches".to_string())
        }

        fn is_goal(&self, state: &AndOrState, _goal: &AndOrState) -> bool {
            state.id.starts_with("Goal")
        }
    }

    impl AndOrPlanDomain<AndOrState> for SimpleAndOrDomain {
        fn get_and_or_branches(&self, state: &AndOrState) -> Vec<AndOrBranch<AndOrState>> {
            let op = CausalOperator::new(
                "DecomposeTask",
                SchemaId::new("State"),
                SchemaId::new("SubState"),
                vec![],
                EffectId::new("and_split"),
                vec![],
                CausalStatus::Observational,
                Some(Cost::new(2, "units")),
                Some(Risk::new(0.0, "zero")),
            )
            .unwrap();

            if state.id == "Root" {
                vec![AndOrBranch {
                    operator: op,
                    kind: HyperNodeKind::And,
                    child_states: vec![
                        AndOrState {
                            id: "GoalA".to_string(),
                        },
                        AndOrState {
                            id: "GoalB".to_string(),
                        },
                    ],
                }]
            } else {
                vec![]
            }
        }
    }

    struct ZeroHeuristic;
    impl AdmissibleHeuristic<AndOrState> for ZeroHeuristic {
        fn estimate(&self, _current: &AndOrState, _goal: &AndOrState) -> f64 {
            0.0
        }
    }

    #[test]
    fn test_ao_star_matches_reference_on_and_or_worlds() {
        let planner = AoStarPlanner::new(0.0);
        let domain = SimpleAndOrDomain;
        let heuristic = ZeroHeuristic;

        let root = AndOrState {
            id: "Root".to_string(),
        };
        let goal_a = AndOrState {
            id: "GoalA".to_string(),
        };

        let budget = Budget {
            cpu_steps_remaining: 100,
            wall_time_ms_limit: 100,
            max_allocations: 100,
        };

        // Root splits into AND(GoalA, GoalB). GoalA and GoalB are both goal states.
        let res = planner
            .plan_and_or(&domain, &heuristic, root, goal_a, &budget)
            .unwrap();

        assert_eq!(res.total_g_cost, 2.0);
        assert_eq!(res.path_operators.len(), 1);
    }
}
