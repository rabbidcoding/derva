// INVARIANT: Verification obligations are explicit typed contracts; 0 self-satisfaction by protected claim.
// KPI: 100% of critical promotions resolve explicit obligation sets.

use crate::orid::ORID;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObligationKind {
    SourceRequired,
    IndependentSource,
    Execution,
    Proof,
    Intervention,
    Freshness,
    HumanApproval,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObligationError {
    SelfSatisfactionProhibited,
    Expired,
    UnresolvedDependencies,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedObligation {
    pub id: ORID,
    pub target_claim: ORID,
    pub kind: ObligationKind,
    pub expires_at: u64,
    pub is_resolved: bool,
    pub discharge_witness: Option<ORID>,
}

impl TypedObligation {
    pub fn new(id: ORID, target_claim: ORID, kind: ObligationKind, expires_at: u64) -> Self {
        Self {
            id,
            target_claim,
            kind,
            expires_at,
            is_resolved: false,
            discharge_witness: None,
        }
    }

    /// Attempts to discharge the obligation with an explicit witness ORID.
    /// Rejects self-satisfaction when witness_orid == target_claim.
    pub fn discharge(
        &mut self,
        witness_orid: ORID,
        current_time: u64,
    ) -> Result<(), ObligationError> {
        if witness_orid == self.target_claim {
            return Err(ObligationError::SelfSatisfactionProhibited);
        }

        if self.expires_at > 0 && current_time > self.expires_at {
            return Err(ObligationError::Expired);
        }

        self.is_resolved = true;
        self.discharge_witness = Some(witness_orid);
        Ok(())
    }

    /// Evaluates if the obligation is resolved and fresh (unexpired).
    pub fn is_valid_and_fresh(&self, current_time: u64) -> bool {
        if !self.is_resolved {
            return false;
        }
        if self.expires_at > 0 && current_time > self.expires_at {
            return false;
        }
        true
    }
}

/// Detects cycles in obligation dependency graphs.
pub fn detect_obligation_cycles(dependencies: &HashMap<ORID, Vec<ORID>>) -> bool {
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();

    for &node in dependencies.keys() {
        if dfs_cycle_check(node, dependencies, &mut visited, &mut rec_stack) {
            return true;
        }
    }
    false
}

fn dfs_cycle_check(
    node: ORID,
    graph: &HashMap<ORID, Vec<ORID>>,
    visited: &mut HashSet<ORID>,
    rec_stack: &mut HashSet<ORID>,
) -> bool {
    if rec_stack.contains(&node) {
        return true;
    }
    if visited.contains(&node) {
        return false;
    }

    visited.insert(node);
    rec_stack.insert(node);

    if let Some(neighbors) = graph.get(&node) {
        for &neighbor in neighbors {
            if dfs_cycle_check(neighbor, graph, visited, rec_stack) {
                return true;
            }
        }
    }

    rec_stack.remove(&node);
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orid::ObjectKind;

    #[test]
    fn test_self_satisfaction_prohibited() {
        let claim_id = ORID::compute(ObjectKind::Claim, b"claim_under_test");
        let ob_id = ORID::compute(ObjectKind::Obligation, b"ob_1");

        let mut ob = TypedObligation::new(ob_id, claim_id, ObligationKind::Proof, 0);

        // Attempting self-satisfaction (witness == target_claim) MUST fail
        let res = ob.discharge(claim_id, 100);
        assert_eq!(res, Err(ObligationError::SelfSatisfactionProhibited));
        assert!(!ob.is_valid_and_fresh(100));

        // Discharging with external witness MUST succeed
        let witness_id = ORID::compute(ObjectKind::Evidence, b"external_proof");
        let res_ok = ob.discharge(witness_id, 100);
        assert!(res_ok.is_ok());
        assert!(ob.is_valid_and_fresh(100));
    }

    #[test]
    fn test_expiration_invalidates_freshness() {
        let claim_id = ORID::compute(ObjectKind::Claim, b"claim_exp");
        let ob_id = ORID::compute(ObjectKind::Obligation, b"ob_exp");
        let witness_id = ORID::compute(ObjectKind::Evidence, b"witness_exp");

        let mut ob = TypedObligation::new(ob_id, claim_id, ObligationKind::Freshness, 1000);
        assert!(ob.discharge(witness_id, 500).is_ok());
        assert!(ob.is_valid_and_fresh(999));

        // Expired obligation MUST evaluate as invalid
        assert!(!ob.is_valid_and_fresh(1001));
    }

    #[test]
    fn test_obligation_cycle_detection() {
        let a = ORID::compute(ObjectKind::Obligation, b"A");
        let b = ORID::compute(ObjectKind::Obligation, b"B");

        let mut graph = HashMap::new();
        graph.insert(a, vec![b]);
        graph.insert(b, vec![a]); // Cycle A -> B -> A

        assert!(detect_obligation_cycles(&graph));

        graph.insert(b, vec![]); // Break cycle
        assert!(!detect_obligation_cycles(&graph));
    }
}
