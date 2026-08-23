#![forbid(unsafe_code)]

// INVARIANT: 0 diagnostics promoted automatically to causal status; exact agreement in finite worlds; budget-bounded for large domains.
// KPI: 0 automatic causal promotions; 100% exact commutativity delta in finite worlds; strict budget compliance.

use crate::counterfactual::CounterfactualFork;
use origin_core::{Budget, ObjectKind, ORID};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateDelta {
    pub graph_diff_count: usize,
    pub evidence_diff_count: usize,
    pub operator_diff_count: usize,
    pub total_differences: usize,
}

impl StateDelta {
    pub fn is_zero(&self) -> bool {
        self.total_differences == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticTag {
    OrderInteraction,
}

// AUDIT-LENSES: Knuth, Guido, Turing
#[derive(Debug, Clone, PartialEq)]
pub struct OrderInteractionDiagnostic {
    pub diagnostic_id: ORID,
    pub tag: DiagnosticTag,
    pub operator_a_id: ORID,
    pub operator_b_id: ORID,
    pub is_commutative: bool,
    pub delta: StateDelta,
    pub evaluated_steps: usize,
    pub budget_exhausted: bool,
}

#[derive(Debug, Clone, Default)]
pub struct InteractionDiagnosticChecker;

impl InteractionDiagnosticChecker {
    pub fn new() -> Self {
        Self
    }

    /// Evaluates state differences between U_a o U_b and U_b o U_a on counterfactual forks.
    /// Returns a purely diagnostic result. MUST NEVER modify or emit CausalStatus.
    pub fn evaluate_order_interaction(
        &self,
        op_a_id: ORID,
        op_b_id: ORID,
        fork_a_then_b: &CounterfactualFork,
        fork_b_then_a: &CounterfactualFork,
        budget: &Budget,
    ) -> OrderInteractionDiagnostic {
        let state_ab = &fork_a_then_b.state;
        let state_ba = &fork_b_then_a.state;

        let mut graph_diff = 0;
        let mut steps_used = 0usize;
        let max_steps = (budget.cpu_steps_remaining.max(1)) as usize;
        let mut budget_exhausted = false;

        // Compare graph claims between (A o B) and (B o A)
        for (id, claim_ab) in &state_ab.graph {
            if steps_used >= max_steps {
                budget_exhausted = true;
                break;
            }
            steps_used += 1;

            if let Some(claim_ba) = state_ba.graph.get(id) {
                if claim_ab != claim_ba {
                    graph_diff += 1;
                }
            } else {
                graph_diff += 1;
            }
        }

        // Account for keys present in B o A but missing in A o B
        for id in state_ba.graph.keys() {
            if !state_ab.graph.contains_key(id) {
                graph_diff += 1;
            }
        }

        let evidence_diff = if state_ab.evidence == state_ba.evidence {
            0
        } else {
            1
        };
        let operator_diff = if state_ab.operators == state_ba.operators {
            0
        } else {
            1
        };

        let total_diffs = graph_diff + evidence_diff + operator_diff;
        let delta = StateDelta {
            graph_diff_count: graph_diff,
            evidence_diff_count: evidence_diff,
            operator_diff_count: operator_diff,
            total_differences: total_diffs,
        };

        let is_commutative = delta.is_zero();

        let seed = format!("{}:{}:{}", op_a_id, op_b_id, total_diffs);
        let diagnostic_id = ORID::compute(ObjectKind::Artifact, seed.as_bytes());

        OrderInteractionDiagnostic {
            diagnostic_id,
            tag: DiagnosticTag::OrderInteraction,
            operator_a_id: op_a_id,
            operator_b_id: op_b_id,
            is_commutative,
            delta,
            evaluated_steps: steps_used,
            budget_exhausted,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use origin_core::object::Claim;
    use origin_core::state::StateTxn;
    use origin_core::status::Status;
    use origin_core::State;

    fn create_test_forks_commutative() -> (CounterfactualFork, CounterfactualFork) {
        let state = State::new();
        let parent_id = ORID::compute(ObjectKind::Commit, b"root");

        let fork_ab = CounterfactualFork::fork(&state, parent_id);
        let fork_ba = CounterfactualFork::fork(&state, parent_id);

        (fork_ab, fork_ba)
    }

    fn create_test_forks_non_commutative() -> (CounterfactualFork, CounterfactualFork) {
        let state = State::new();
        let parent_id = ORID::compute(ObjectKind::Commit, b"root");

        let mut fork_ab = CounterfactualFork::fork(&state, parent_id);
        let fork_ba = CounterfactualFork::fork(&state, parent_id);

        // Mutate fork_ab only
        let mut txn = StateTxn::new(fork_ab.state.clone());
        let claim_id = ORID::compute(ObjectKind::Claim, b"order_dependent_claim");
        txn.add_claim(Claim {
            id: claim_id,
            statement: "State changed by A then B".to_string(),
            status: Status::Supported,
            provenance_roots: vec![],
        });
        fork_ab.apply_txn(txn).unwrap();

        (fork_ab, fork_ba)
    }

    #[test]
    fn test_zero_diagnostics_promoted_automatically_to_causal_status() {
        let checker = InteractionDiagnosticChecker::new();
        let (fork_ab, fork_ba) = create_test_forks_commutative();
        let op_a = ORID::compute(ObjectKind::Operator, b"op_a");
        let op_b = ORID::compute(ObjectKind::Operator, b"op_b");
        let budget = Budget {
            cpu_steps_remaining: 1000,
            wall_time_ms_limit: 100,
            max_allocations: 100,
        };

        let diag = checker.evaluate_order_interaction(op_a, op_b, &fork_ab, &fork_ba, &budget);

        // Assert tag is strictly DiagnosticTag::OrderInteraction
        assert_eq!(diag.tag, DiagnosticTag::OrderInteraction);

        // Inspect file content static safety check: interaction.rs does NOT contain CausalPromotionValidator or causal_promote in non-test code
        let code = include_str!("interaction.rs");
        let non_test_code = code.split("#[cfg(test)]").next().unwrap();
        let forbidden_fn = format!("{}_{}", "causal", "promote");

        assert!(
            !non_test_code.contains(&forbidden_fn),
            "interaction.rs non-test code MUST NOT invoke promotion function"
        );
        assert!(
            !non_test_code.contains("CausalPromotionValidator"),
            "interaction.rs non-test code MUST NOT instantiate CausalPromotionValidator"
        );
    }

    #[test]
    fn test_exact_agreement_in_finite_worlds() {
        let checker = InteractionDiagnosticChecker::new();
        let op_a = ORID::compute(ObjectKind::Operator, b"op_a");
        let op_b = ORID::compute(ObjectKind::Operator, b"op_b");
        let budget = Budget {
            cpu_steps_remaining: 1000,
            wall_time_ms_limit: 100,
            max_allocations: 100,
        };

        // 1. Commutative evaluation
        let (fork_ab_c, fork_ba_c) = create_test_forks_commutative();
        let diag_c =
            checker.evaluate_order_interaction(op_a, op_b, &fork_ab_c, &fork_ba_c, &budget);
        assert!(diag_c.is_commutative);
        assert!(diag_c.delta.is_zero());

        // 2. Non-commutative evaluation
        let (fork_ab_nc, fork_ba_nc) = create_test_forks_non_commutative();
        let diag_nc =
            checker.evaluate_order_interaction(op_a, op_b, &fork_ab_nc, &fork_ba_nc, &budget);
        assert!(!diag_nc.is_commutative);
        assert_eq!(diag_nc.delta.graph_diff_count, 1);
        assert_eq!(diag_nc.delta.total_differences, 1);
    }

    #[test]
    fn test_budget_bounded_for_large_domains() {
        let checker = InteractionDiagnosticChecker::new();
        let mut state = State::new();

        // Populate 100 claims
        for i in 0..100 {
            let id = ORID::compute(ObjectKind::Claim, format!("c_{}", i).as_bytes());
            state.graph.insert(
                id,
                Claim {
                    id,
                    statement: format!("Claim {}", i),
                    status: Status::Supported,
                    provenance_roots: vec![],
                },
            );
        }

        let parent_id = ORID::compute(ObjectKind::Commit, b"root");
        let fork_ab = CounterfactualFork::fork(&state, parent_id);
        let fork_ba = CounterfactualFork::fork(&state, parent_id);

        let tight_budget = Budget {
            cpu_steps_remaining: 10, // Only 10 steps allowed out of 100
            wall_time_ms_limit: 10,
            max_allocations: 10,
        };

        let op_a = ORID::compute(ObjectKind::Operator, b"op_a");
        let op_b = ORID::compute(ObjectKind::Operator, b"op_b");

        let diag =
            checker.evaluate_order_interaction(op_a, op_b, &fork_ab, &fork_ba, &tight_budget);

        assert!(diag.budget_exhausted, "Budget MUST be marked as exhausted");
        assert_eq!(
            diag.evaluated_steps, 10,
            "Evaluated steps MUST be capped at budget ceiling"
        );
    }
}
