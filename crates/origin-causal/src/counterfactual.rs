#![forbid(unsafe_code)]

// INVARIANT: Counterfactual branch modifies real root 0%; external effect capabilities absent by construction; fork creation p99 < 1ms.
// KPI: 0 mutations to original state; 0 real intervention capabilities by construction; p99 fork creation < 1ms.

use origin_core::state::TxnError;
use origin_core::{ObjectKind, State, StateTxn, ORID};
use std::collections::HashSet;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability {
    ReadState,
    SimulateHypothesis,
    ExtractActiveSlice,
}

// AUDIT-LENSES: Torvalds, Thompson, Wozniak
#[derive(Debug, Clone)]
pub struct CounterfactualFork {
    pub parent_commit_id: ORID,
    pub fork_id: ORID,
    pub state: State,
    pub capabilities: HashSet<Capability>,
    pub creation_duration_us: u64,
}

impl CounterfactualFork {
    /// Creates an isolated copy-on-write counterfactual branch from base State.
    /// INVARIANT: External effect capabilities ABSENT by construction.
    pub fn fork(base_state: &State, parent_commit_id: ORID) -> Self {
        let start = Instant::now();

        let fork_seed = format!("{}:{}", parent_commit_id, base_state.schema_version);
        let fork_id = ORID::compute(ObjectKind::Commit, fork_seed.as_bytes());

        // CoW clone of authoritative State
        let fork_state = base_state.clone();

        // Capabilities initialized WITHOUT real intervention or external write abilities
        let mut capabilities = HashSet::new();
        capabilities.insert(Capability::ReadState);
        capabilities.insert(Capability::SimulateHypothesis);
        capabilities.insert(Capability::ExtractActiveSlice);

        let duration_us = start.elapsed().as_micros() as u64;

        Self {
            parent_commit_id,
            fork_id,
            state: fork_state,
            capabilities,
            creation_duration_us: duration_us,
        }
    }

    pub fn capabilities(&self) -> &HashSet<Capability> {
        &self.capabilities
    }

    pub fn allows_capability(&self, cap: &Capability) -> bool {
        self.capabilities.contains(cap)
    }

    /// Applies mutations isolated exclusively to this counterfactual fork.
    pub fn apply_txn(&mut self, txn: StateTxn) -> Result<(), TxnError> {
        self.state = txn.commit()?;
        Ok(())
    }
}

pub trait StateCounterfactualExt {
    fn fork_counterfactual(&self, parent_commit_id: ORID) -> Result<CounterfactualFork, String>;
}

impl StateCounterfactualExt for State {
    fn fork_counterfactual(&self, parent_commit_id: ORID) -> Result<CounterfactualFork, String> {
        Ok(CounterfactualFork::fork(self, parent_commit_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use origin_core::object::Claim;
    use origin_core::status::Status;

    #[test]
    fn test_counterfactual_branch_does_not_modify_real_root_100_percent() {
        let real_root = State::new();
        let parent_id = ORID::compute(ObjectKind::Commit, b"head_commit");

        let mut cf_fork = real_root.fork_counterfactual(parent_id).unwrap();

        // Mutate counterfactual fork state
        let mut txn = StateTxn::new(cf_fork.state.clone());
        let claim_id = ORID::compute(ObjectKind::Claim, b"hypothetical_claim");
        txn.add_claim(Claim {
            id: claim_id,
            statement: "Hypothetical counterfactual statement".to_string(),
            status: Status::Hypothesis,
            provenance_roots: vec![],
        });
        cf_fork.apply_txn(txn).unwrap();

        // Assert counterfactual fork HAS the claim
        assert_eq!(cf_fork.state.graph.len(), 1);

        // Assert real_root remains 100% UNTOUCHED
        assert_eq!(
            real_root.graph.len(),
            0,
            "Real root MUST NOT be modified by counterfactual branch operations"
        );
    }

    #[test]
    fn test_external_effect_capabilities_absent_by_construction() {
        let real_root = State::new();
        let parent_id = ORID::compute(ObjectKind::Commit, b"head_commit");
        let cf_fork = real_root.fork_counterfactual(parent_id).unwrap();

        let caps = cf_fork.capabilities();

        // Assert allowed capabilities are strictly simulated/read
        assert!(caps.contains(&Capability::ReadState));
        assert!(caps.contains(&Capability::SimulateHypothesis));
        assert!(caps.contains(&Capability::ExtractActiveSlice));

        // Assert no capability permits external intervention
        assert_eq!(caps.len(), 3);
    }

    #[test]
    fn test_fork_creation_p99_under_1ms() {
        let mut real_root = State::new();

        // Populate state with metadata
        for i in 0..100 {
            let id = ORID::compute(ObjectKind::Claim, format!("claim_{}", i).as_bytes());
            real_root.graph.insert(
                id,
                Claim {
                    id,
                    statement: format!("Statement {}", i),
                    status: Status::Supported,
                    provenance_roots: vec![],
                },
            );
        }

        let parent_id = ORID::compute(ObjectKind::Commit, b"head_commit");
        let sample_count = 1_000;
        let mut durations = Vec::with_capacity(sample_count);

        for _ in 0..sample_count {
            let start = Instant::now();
            let _cf = real_root.fork_counterfactual(parent_id).unwrap();
            durations.push(start.elapsed());
        }

        durations.sort();

        let p50 = durations[sample_count / 2];
        let p99 = durations[(sample_count * 99) / 100];

        println!("Fork Creation Latency: p50={:?}, p99={:?}", p50, p99);

        let max_allowed = std::time::Duration::from_millis(1);
        assert!(
            p99 < max_allowed,
            "p99 fork creation latency ({:?}) MUST be < 1ms",
            p99
        );
    }
}
