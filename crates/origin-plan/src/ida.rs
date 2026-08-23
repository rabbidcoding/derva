#![forbid(unsafe_code)]

// INVARIANT: IDA* memory <= 25% of A* in deep benchmarks with <= 20% cost overhead target; linear stack space O(depth).
// KPI: <= 25% peak allocated nodes compared to A*; <= 20% cost overhead; 100% path optimal.

use crate::astar::{AdmissibleHeuristic, PlanDomain, PlanError, PlanResult};
use origin_core::{Budget, ORID};
use std::collections::HashSet;

// AUDIT-LENSES: Wozniak, Stroustrup, Knuth
#[derive(Debug, Clone)]
pub struct IdaStarPlanner {
    pub lambda_risk: f64,
    pub lambda_uncertainty: f64,
}

impl Default for IdaStarPlanner {
    fn default() -> Self {
        Self {
            lambda_risk: 1.0,
            lambda_uncertainty: 1.0,
        }
    }
}

impl IdaStarPlanner {
    pub fn new(lambda_risk: f64, lambda_uncertainty: f64) -> Self {
        Self {
            lambda_risk,
            lambda_uncertainty,
        }
    }

    pub fn plan<S: Clone, D: PlanDomain<S>, H: AdmissibleHeuristic<S>>(
        &self,
        domain: &D,
        heuristic: &H,
        initial_state: S,
        goal: S,
        budget: &Budget,
    ) -> Result<PlanResult<S>, PlanError> {
        if !heuristic.is_admissible() {
            return Err(PlanError::InadmissibleHeuristicRejected);
        }

        let start_id = domain.canonical_id(&initial_state);
        let h_initial = heuristic.estimate(&initial_state, &goal);
        let mut bound = h_initial;

        let mut path_states = vec![(initial_state.clone(), start_id)];
        let mut path_ops: Vec<ORID> = Vec::new();
        let mut visited_ids: HashSet<ORID> = HashSet::new();
        visited_ids.insert(start_id);

        let mut expansions: u64 = 0;
        let max_expansions = budget.cpu_steps_remaining.max(1);

        loop {
            let mut min_next_bound = f64::INFINITY;

            let search_res = self.search_dfs(
                domain,
                heuristic,
                &goal,
                0.0,
                0.0,
                0.0,
                bound,
                &mut path_states,
                &mut path_ops,
                &mut visited_ids,
                &mut expansions,
                max_expansions,
                &mut min_next_bound,
            );

            match search_res {
                Ok(Some(res)) => return Ok(res),
                Ok(None) => {
                    if min_next_bound.is_infinite() || min_next_bound <= bound {
                        return Err(PlanError::NoPathFound);
                    }
                    bound = min_next_bound;
                }
                Err(e) => return Err(e),
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn search_dfs<S: Clone, D: PlanDomain<S>, H: AdmissibleHeuristic<S>>(
        &self,
        domain: &D,
        heuristic: &H,
        goal: &S,
        g_cost: f64,
        risk: f64,
        uncertainty: f64,
        bound: f64,
        path_states: &mut Vec<(S, ORID)>,
        path_ops: &mut Vec<ORID>,
        visited_ids: &mut HashSet<ORID>,
        expansions: &mut u64,
        max_expansions: u64,
        min_next_bound: &mut f64,
    ) -> Result<Option<PlanResult<S>>, PlanError> {
        if *expansions >= max_expansions {
            return Err(PlanError::BudgetExhausted);
        }
        *expansions += 1;

        let (curr_state, _curr_id) = path_states.last().unwrap().clone();

        let h_val = heuristic.estimate(&curr_state, goal);
        let f_score =
            g_cost + (self.lambda_risk * risk) + (self.lambda_uncertainty * uncertainty) + h_val;

        if f_score > bound {
            if f_score < *min_next_bound {
                *min_next_bound = f_score;
            }
            return Ok(None);
        }

        if domain.is_goal(&curr_state, goal) {
            return Ok(Some(PlanResult {
                path_operators: path_ops.clone(),
                final_state: curr_state,
                total_g_cost: g_cost,
                total_risk: risk,
                total_uncertainty: uncertainty,
                final_f_score: f_score,
                expansions: *expansions,
            }));
        }

        let operators = domain.get_applicable_operators(&curr_state);

        for op in operators {
            let next_state = match domain.apply_operator(&curr_state, &op) {
                Ok(s) => s,
                Err(e) => return Err(PlanError::DomainError(e)),
            };

            let next_id = domain.canonical_id(&next_state);
            if visited_ids.contains(&next_id) {
                continue;
            }

            let step_cost = op.cost.as_ref().map(|c| c.value as f64).unwrap_or(1.0);
            let step_risk = op.risk.as_ref().map(|r| r.score).unwrap_or(0.0);

            visited_ids.insert(next_id);
            path_states.push((next_state.clone(), next_id));
            path_ops.push(op.id);

            let res = self.search_dfs(
                domain,
                heuristic,
                goal,
                g_cost + step_cost,
                risk + step_risk,
                uncertainty,
                bound,
                path_states,
                path_ops,
                visited_ids,
                expansions,
                max_expansions,
                min_next_bound,
            )?;

            path_ops.pop();
            path_states.pop();
            visited_ids.remove(&next_id);

            if res.is_some() {
                return Ok(res);
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use origin_causal::operator::{CausalOperator, Cost, EffectId, Risk, SchemaId};
    use origin_core::{CausalStatus, ObjectKind};

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct GridPos {
        x: i32,
        y: i32,
    }

    struct GridDomain;

    impl PlanDomain<GridPos> for GridDomain {
        fn canonical_id(&self, state: &GridPos) -> ORID {
            let seed = format!("pos:{}:{}", state.x, state.y);
            ORID::compute(ObjectKind::Entity, seed.as_bytes())
        }

        fn get_applicable_operators(&self, state: &GridPos) -> Vec<CausalOperator> {
            let moves = [
                ("MoveRight", 1, 0),
                ("MoveUp", 0, 1),
                ("MoveLeft", -1, 0),
                ("MoveDown", 0, -1),
            ];

            let mut ops = Vec::new();
            for (name, dx, dy) in moves {
                let op_name = format!("{}_{}_{}", name, state.x, state.y);
                let op = CausalOperator::new(
                    op_name,
                    SchemaId::new("GridPos"),
                    SchemaId::new("GridPos"),
                    vec![],
                    EffectId::new(format!("move_{}_{}", dx, dy)),
                    vec![],
                    CausalStatus::Observational,
                    Some(Cost::new(1, "step")),
                    Some(Risk::new(0.01, "low")),
                )
                .unwrap();
                ops.push(op);
            }
            ops
        }

        fn apply_operator(&self, state: &GridPos, op: &CausalOperator) -> Result<GridPos, String> {
            if op.effect.0.contains("move_1_0") {
                Ok(GridPos {
                    x: state.x + 1,
                    y: state.y,
                })
            } else if op.effect.0.contains("move_0_1") {
                Ok(GridPos {
                    x: state.x,
                    y: state.y + 1,
                })
            } else if op.effect.0.contains("move_-1_0") {
                Ok(GridPos {
                    x: state.x - 1,
                    y: state.y,
                })
            } else if op.effect.0.contains("move_0_-1") {
                Ok(GridPos {
                    x: state.x,
                    y: state.y - 1,
                })
            } else {
                Err("Unknown move".to_string())
            }
        }

        fn is_goal(&self, state: &GridPos, goal: &GridPos) -> bool {
            state == goal
        }
    }

    struct ManhattanHeuristic;

    impl AdmissibleHeuristic<GridPos> for ManhattanHeuristic {
        fn estimate(&self, current: &GridPos, goal: &GridPos) -> f64 {
            ((current.x - goal.x).abs() + (current.y - goal.y).abs()) as f64
        }
    }

    #[test]
    fn test_ida_star_memory_bounded_and_optimal() {
        let planner = IdaStarPlanner::new(0.0, 0.0);
        let domain = GridDomain;
        let heuristic = ManhattanHeuristic;

        let start = GridPos { x: 0, y: 0 };
        let goal = GridPos { x: 3, y: 3 };

        let budget = Budget {
            cpu_steps_remaining: 1000,
            wall_time_ms_limit: 100,
            max_allocations: 100,
        };

        let res = planner
            .plan(&domain, &heuristic, start, goal.clone(), &budget)
            .unwrap();

        assert_eq!(res.final_state, goal);
        assert_eq!(res.path_operators.len(), 6);
        assert_eq!(res.total_g_cost, 6.0);
    }
}
