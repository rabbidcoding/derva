// INVARIANT: Reference constraint solver produces exact SAT/UNSAT/UNKNOWN results; Timeout NEVER converts to UNSAT.
// KPI: Reference solver exact on small exhaustive domains; 0 Timeout-to-UNSAT conversion.

use crate::{BudgetStop, Constraint, Proof, SolveResult, Witness};
use origin_core::{ObjectKind, ORID};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default, Clone)]
pub struct ReferenceSolver;

struct SearchContext<'a> {
    vars: &'a [String],
    constraints: &'a [Constraint],
    assignment: HashMap<String, i64>,
    steps: u64,
    budget_steps: u64,
    found_witness: Option<Witness>,
    budget_exceeded: bool,
}

impl ReferenceSolver {
    pub fn new() -> Self {
        Self
    }

    /// Solves a set of finite-domain linear/equality constraints under budget step constraints.
    /// INVARIANT: If budget is exhausted, returns SolveResult::Unknown(BudgetStop), NEVER SolveResult::Unsat.
    pub fn solve(&self, constraints: &[Constraint], budget_steps: u64) -> SolveResult<Witness> {
        let mut vars = HashSet::new();
        for c in constraints {
            c.collect_variables(&mut vars);
        }

        let mut var_list: Vec<String> = vars.into_iter().collect();
        var_list.sort();

        let mut ctx = SearchContext {
            vars: &var_list,
            constraints,
            assignment: HashMap::new(),
            steps: 0,
            budget_steps,
            found_witness: None,
            budget_exceeded: false,
        };

        search_backtrack(&mut ctx, 0);

        if let Some(w) = ctx.found_witness {
            SolveResult::Sat(w)
        } else if ctx.budget_exceeded {
            SolveResult::Unknown(BudgetStop {
                reason: format!("Budget step limit exceeded (steps >= {})", budget_steps),
                steps_taken: ctx.steps,
            })
        } else {
            // Unsatisfiable proof
            let proof_buf = format!("unsat_core_{:?}", constraints).into_bytes();
            let proof_id = ORID::compute(ObjectKind::Artifact, &proof_buf);
            SolveResult::Unsat(Proof {
                unsat_core: constraints.to_vec(),
                proof_id,
            })
        }
    }
}

fn search_backtrack(ctx: &mut SearchContext, idx: usize) {
    if idx == ctx.vars.len() {
        if check_all_constraints(ctx.constraints, &ctx.assignment) {
            let mut w_buf = Vec::new();
            let mut sorted_keys: Vec<&String> = ctx.assignment.keys().collect();
            sorted_keys.sort();
            for k in sorted_keys {
                w_buf.extend_from_slice(k.as_bytes());
                w_buf.extend_from_slice(&ctx.assignment[k].to_be_bytes());
            }
            let witness_id = ORID::compute(ObjectKind::Artifact, &w_buf);
            ctx.found_witness = Some(Witness {
                assignments: ctx.assignment.clone(),
                witness_id,
            });
        }
        return;
    }

    if ctx.found_witness.is_some() || ctx.budget_exceeded {
        return;
    }

    let var_name = ctx.vars[idx].clone();
    for val in -100..=100 {
        if ctx.steps >= ctx.budget_steps {
            ctx.budget_exceeded = true;
            return;
        }
        ctx.steps += 1;

        ctx.assignment.insert(var_name.clone(), val);
        if check_all_constraints(ctx.constraints, &ctx.assignment) {
            search_backtrack(ctx, idx + 1);
            if ctx.found_witness.is_some() || ctx.budget_exceeded {
                return;
            }
        }
        ctx.assignment.remove(&var_name);
    }
}

fn check_all_constraints(constraints: &[Constraint], env: &HashMap<String, i64>) -> bool {
    for c in constraints {
        if !c.evaluate(env) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Constraint;

    #[test]
    fn test_sat_linear_equality_constraint() {
        let solver = ReferenceSolver::new();
        // x == 42, y == 10
        let constraints = vec![
            Constraint::VarEquals("x".to_string(), 42),
            Constraint::VarEquals("y".to_string(), 10),
        ];

        let res = solver.solve(&constraints, 10000);
        match res {
            SolveResult::Sat(w) => {
                assert_eq!(w.assignments.get("x"), Some(&42));
                assert_eq!(w.assignments.get("y"), Some(&10));
            }
            _ => panic!("Expected Sat result"),
        }
    }

    #[test]
    fn test_unsat_contradictory_constraints() {
        let solver = ReferenceSolver::new();
        // x == 10 AND x == 20 (contradiction)
        let constraints = vec![
            Constraint::VarEquals("x".to_string(), 10),
            Constraint::VarEquals("x".to_string(), 20),
        ];

        let res = solver.solve(&constraints, 10000);
        match res {
            SolveResult::Unsat(proof) => {
                assert_eq!(proof.unsat_core.len(), 2);
            }
            _ => panic!("Expected Unsat result"),
        }
    }

    #[test]
    fn test_timeout_never_converts_to_unsat() {
        let solver = ReferenceSolver::new();
        // Constraint over large search domain with tiny budget (budget_steps = 5)
        let constraints = vec![
            Constraint::VarGreaterThan("x".to_string(), 50),
            Constraint::VarLessThan("x".to_string(), 90),
            Constraint::VarGreaterThan("y".to_string(), 50),
        ];

        let res = solver.solve(&constraints, 5);
        match res {
            SolveResult::Unknown(stop) => {
                assert!(stop.steps_taken >= 5);
            }
            SolveResult::Unsat(_) => panic!("Timeout must NEVER return Unsat!"),
            _ => panic!("Expected Unknown result due to budget stop"),
        }
    }
}
