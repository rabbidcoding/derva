// INVARIANT: State S = (G, C, E, U, O, B, Z) is the single authoritative ground truth.
// KPI: 0 authoritative state fields duplicated in numerical or runtime components.

use crate::object::{Claim, Evidence, Obligation, Operator};
use crate::orid::ORID;
use std::collections::HashMap;

pub const CURRENT_SCHEMA_VERSION: u32 = 0;

pub type GraphRoot = HashMap<ORID, Claim>;
pub type ConstraintRoot = Vec<u8>;
pub type EvidenceRoot = HashMap<ORID, Evidence>;
pub type OperatorRoot = HashMap<ORID, Operator>;
pub type ObligationRoot = HashMap<ORID, Obligation>;
pub type ArtifactRoot = HashMap<ORID, Vec<u8>>;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Budget {
    pub cpu_steps_remaining: u64,
    pub wall_time_ms_limit: u64,
    pub max_allocations: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub schema_version: u32,
    pub graph: GraphRoot,
    pub constraints: ConstraintRoot,
    pub evidence: EvidenceRoot,
    pub operators: OperatorRoot,
    pub obligations: ObligationRoot,
    pub budget: Budget,
    pub artifacts: ArtifactRoot,
}

impl Default for State {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            graph: HashMap::new(),
            constraints: Vec::new(),
            evidence: HashMap::new(),
            operators: HashMap::new(),
            obligations: HashMap::new(),
            budget: Budget::default(),
            artifacts: HashMap::new(),
        }
    }
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_schema_valid(&self) -> bool {
        self.schema_version == CURRENT_SCHEMA_VERSION
    }
}

#[derive(Debug, Clone)]
pub struct StateTxn {
    pub base_state: State,
    pub pending_claims: Vec<Claim>,
    pub pending_evidence: Vec<Evidence>,
    pub pending_operators: Vec<Operator>,
    pub pending_obligations: Vec<Obligation>,
    pub pending_artifacts: Vec<(ORID, Vec<u8>)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxnError {
    SchemaVersionMismatch { expected: u32, found: u32 },
    BudgetExhausted,
}

impl StateTxn {
    pub fn new(base: State) -> Self {
        Self {
            base_state: base,
            pending_claims: Vec::new(),
            pending_evidence: Vec::new(),
            pending_operators: Vec::new(),
            pending_obligations: Vec::new(),
            pending_artifacts: Vec::new(),
        }
    }

    pub fn add_claim(&mut self, claim: Claim) {
        self.pending_claims.push(claim);
    }

    pub fn add_evidence(&mut self, evidence: Evidence) {
        self.pending_evidence.push(evidence);
    }

    pub fn add_operator(&mut self, operator: Operator) {
        self.pending_operators.push(operator);
    }

    pub fn add_obligation(&mut self, obligation: Obligation) {
        self.pending_obligations.push(obligation);
    }

    pub fn add_artifact(&mut self, id: ORID, data: Vec<u8>) {
        self.pending_artifacts.push((id, data));
    }

    pub fn commit(mut self) -> Result<State, TxnError> {
        if !self.base_state.is_schema_valid() {
            return Err(TxnError::SchemaVersionMismatch {
                expected: CURRENT_SCHEMA_VERSION,
                found: self.base_state.schema_version,
            });
        }

        for claim in self.pending_claims {
            self.base_state.graph.insert(claim.id, claim);
        }
        for ev in self.pending_evidence {
            self.base_state.evidence.insert(ev.id, ev);
        }
        for op in self.pending_operators {
            self.base_state.operators.insert(op.id, op);
        }
        for ob in self.pending_obligations {
            self.base_state.obligations.insert(ob.id, ob);
        }
        for (id, data) in self.pending_artifacts {
            self.base_state.artifacts.insert(id, data);
        }

        Ok(self.base_state)
    }

    pub fn rollback(self) -> State {
        self.base_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_default_schema_version() {
        let state = State::new();
        assert_eq!(state.schema_version, 0);
        assert!(state.is_schema_valid());
    }

    #[test]
    fn test_state_txn_commit_and_rollback() {
        let state = State::new();
        let mut txn = StateTxn::new(state.clone());

        let claim_orid = ORID::compute(crate::orid::ObjectKind::Claim, b"test_claim");
        let claim = Claim {
            id: claim_orid,
            statement: "Ground truth invariant".to_string(),
            status: crate::status::Status::Supported,
            provenance_roots: Vec::new(),
        };

        txn.add_claim(claim.clone());

        // Test rollback preserves base state unchanged
        let rollback_state = txn.clone().rollback();
        assert!(rollback_state.graph.is_empty());

        // Test commit applies pending mutations
        let committed_state = txn.commit().unwrap();
        assert_eq!(committed_state.graph.len(), 1);
        assert_eq!(committed_state.graph.get(&claim_orid), Some(&claim));
    }
}
