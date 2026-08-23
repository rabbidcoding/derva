// INVARIANT: Contradictions mark both claims as CONTESTED; 0 silent overwrites; minimal conflicting set query returns complete conflict pairs.
// KPI: 100% contradictory fixtures preserve both claims; 0 silent overwrites; Conflict query returns minimal conflicting set.

use origin_core::{Status, ORID};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContradictionPair {
    pub claim_a: ORID,
    pub claim_b: ORID,
    pub reason: String,
    pub detected_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictingSet {
    pub primary_claim: ORID,
    pub conflicting_claims: Vec<ORID>,
    pub minimal_conflict_subgraph: Vec<ContradictionPair>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContradictionError {
    SelfContradictionProhibited(ORID),
    AlreadyContested(ORID),
}

impl std::fmt::Display for ContradictionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContradictionError::SelfContradictionProhibited(orid) => {
                write!(
                    f,
                    "Self-contradiction prohibited: claim {} cannot contradict itself",
                    orid
                )
            }
            ContradictionError::AlreadyContested(orid) => {
                write!(f, "Claim {} is already marked CONTESTED", orid)
            }
        }
    }
}

impl std::error::Error for ContradictionError {}

#[derive(Debug, Default, Clone)]
pub struct ContradictionEngine {
    contradiction_map: HashMap<ORID, Vec<ContradictionPair>>,
    statuses: HashMap<ORID, Status>,
}

impl ContradictionEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an incompatibility between claim `a` and claim `b`.
    /// INVARIANT: Marks BOTH claims as Status::Contested; NEVER deletes or overwrites either branch.
    pub fn register_contradiction(
        &mut self,
        a: ORID,
        b: ORID,
        reason: impl Into<String>,
        timestamp: u64,
    ) -> Result<(), ContradictionError> {
        if a == b {
            return Err(ContradictionError::SelfContradictionProhibited(a));
        }

        let reason_str = reason.into();

        let pair_a = ContradictionPair {
            claim_a: a,
            claim_b: b,
            reason: reason_str.clone(),
            detected_at: timestamp,
        };

        let pair_b = ContradictionPair {
            claim_a: b,
            claim_b: a,
            reason: reason_str,
            detected_at: timestamp,
        };

        self.contradiction_map.entry(a).or_default().push(pair_a);
        self.contradiction_map.entry(b).or_default().push(pair_b);

        // Transition BOTH claims to Status::Contested in epistemic lattice
        self.statuses.insert(a, Status::Contested);
        self.statuses.insert(b, Status::Contested);

        Ok(())
    }

    pub fn get_status(&self, claim: &ORID) -> Status {
        self.statuses.get(claim).cloned().unwrap_or(Status::Unknown)
    }

    /// Returns the minimal conflicting set of ORIDs and contradiction relations for `claim`.
    pub fn conflict_query(&self, claim: &ORID) -> Option<ConflictingSet> {
        let pairs = self.contradiction_map.get(claim)?;
        if pairs.is_empty() {
            return None;
        }

        let mut conflict_set = HashSet::new();
        let mut subgraph = Vec::new();

        for pair in pairs {
            conflict_set.insert(pair.claim_b);
            subgraph.push(pair.clone());
        }

        let mut conflicting_claims: Vec<ORID> = conflict_set.into_iter().collect();
        conflicting_claims.sort_by_key(|orid| orid.hash);

        Some(ConflictingSet {
            primary_claim: *claim,
            conflicting_claims,
            minimal_conflict_subgraph: subgraph,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use origin_core::ObjectKind;

    #[test]
    fn test_contradiction_preserves_both_claims_and_marks_contested() {
        let mut engine = ContradictionEngine::new();

        let claim_a = ORID::compute(ObjectKind::Claim, b"temperature_is_20C");
        let claim_b = ORID::compute(ObjectKind::Claim, b"temperature_is_50C");

        engine
            .register_contradiction(claim_a, claim_b, "mutually_exclusive_value", 1000)
            .unwrap();

        // 0 silent overwrites: both claims exist in state
        assert_eq!(engine.get_status(&claim_a), Status::Contested);
        assert_eq!(engine.get_status(&claim_b), Status::Contested);
    }

    #[test]
    fn test_conflict_query_returns_minimal_conflicting_set() {
        let mut engine = ContradictionEngine::new();

        let primary = ORID::compute(ObjectKind::Claim, b"primary_hypothesis");
        let conflict_1 = ORID::compute(ObjectKind::Claim, b"opposing_claim_1");
        let conflict_2 = ORID::compute(ObjectKind::Claim, b"opposing_claim_2");

        engine
            .register_contradiction(primary, conflict_1, "incompatible_facts_1", 1000)
            .unwrap();
        engine
            .register_contradiction(primary, conflict_2, "incompatible_facts_2", 1005)
            .unwrap();

        let query_res = engine.conflict_query(&primary).unwrap();
        assert_eq!(query_res.primary_claim, primary);
        assert_eq!(query_res.conflicting_claims.len(), 2);
        assert!(query_res.conflicting_claims.contains(&conflict_1));
        assert!(query_res.conflicting_claims.contains(&conflict_2));
        assert_eq!(query_res.minimal_conflict_subgraph.len(), 2);
    }

    #[test]
    fn test_self_contradiction_prohibited() {
        let mut engine = ContradictionEngine::new();
        let claim_a = ORID::compute(ObjectKind::Claim, b"claim_x");

        let res = engine.register_contradiction(claim_a, claim_a, "impossible", 1000);
        assert!(res.is_err());
        assert_eq!(engine.get_status(&claim_a), Status::Unknown);
    }
}
