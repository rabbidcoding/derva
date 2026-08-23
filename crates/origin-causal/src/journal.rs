#![forbid(unsafe_code)]

// INVARIANT: 100% real interventions generate receipt+journal entry; missing after-observation marks outcome UNKNOWN, not success; journal append is atomic with state commit.
// KPI: 100% receipt+journal generation; outcome UNKNOWN when after-observation is missing; 100% atomic commit-append consistency.

use origin_core::{ObjectKind, State, StateTxn, ORID};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterventionOutcome {
    Success,
    Unknown,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentReceipt {
    pub receipt_id: ORID,
    pub execution_node: String,
    pub return_code: i32,
    pub stderr_digest: ORID,
}

impl EnvironmentReceipt {
    pub fn new(execution_node: impl Into<String>, return_code: i32, stderr_bytes: &[u8]) -> Self {
        let node_str = execution_node.into();
        let stderr_digest = ORID::compute(ObjectKind::Artifact, stderr_bytes);
        let receipt_orid_seed = format!("{}:{}:{}", node_str, return_code, stderr_digest);
        let receipt_id = ORID::compute(ObjectKind::Evidence, receipt_orid_seed.as_bytes());

        Self {
            receipt_id,
            execution_node: node_str,
            return_code,
            stderr_digest,
        }
    }
}

// AUDIT-LENSES: Thompson, Berners-Lee, Musk
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterventionRecord {
    pub id: ORID,
    pub before_state_root: ORID,
    pub action_orid: ORID,
    pub capability: String,
    pub environment_receipt: EnvironmentReceipt,
    pub after_observation_root: Option<ORID>,
    pub outcome: InterventionOutcome,
    pub timestamp_start_ns: u64,
    pub timestamp_end_ns: u64,
}

impl InterventionRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        before_state_root: ORID,
        action_orid: ORID,
        capability: impl Into<String>,
        environment_receipt: EnvironmentReceipt,
        after_observation_root: Option<ORID>,
        requested_outcome: InterventionOutcome,
        timestamp_start_ns: u64,
        timestamp_end_ns: u64,
    ) -> Self {
        let cap_str = capability.into();
        let seed = format!("{}:{}:{}", before_state_root, action_orid, cap_str);
        let id = ORID::compute(ObjectKind::Artifact, seed.as_bytes());

        // KPI: Missing after-observation MUST mark outcome UNKNOWN, not success
        let outcome = if after_observation_root.is_none() {
            InterventionOutcome::Unknown
        } else {
            requested_outcome
        };

        Self {
            id,
            before_state_root,
            action_orid,
            capability: cap_str,
            environment_receipt,
            after_observation_root,
            outcome,
            timestamp_start_ns,
            timestamp_end_ns,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JournalError {
    TxnCommitFailed(String),
    MissingEnvironmentReceipt,
}

impl std::fmt::Display for JournalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JournalError::TxnCommitFailed(reason) => {
                write!(f, "Transaction commit failed: {}", reason)
            }
            JournalError::MissingEnvironmentReceipt => {
                write!(
                    f,
                    "Environment receipt mandatory for real intervention journal append"
                )
            }
        }
    }
}

impl std::error::Error for JournalError {}

#[derive(Debug, Clone, Default)]
pub struct InterventionJournal {
    pub entries: Vec<InterventionRecord>,
}

impl InterventionJournal {
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends an intervention record directly to the journal.
    pub fn append(&mut self, mut record: InterventionRecord) -> Result<ORID, JournalError> {
        // Enforce missing after-observation rule
        if record.after_observation_root.is_none() {
            record.outcome = InterventionOutcome::Unknown;
        }

        let record_id = record.id;
        self.entries.push(record);
        Ok(record_id)
    }

    /// Appends an intervention record atomically together with a state transaction commit.
    /// INVARIANT: Journal append is atomic with commit (if commit fails, journal is unchanged).
    pub fn append_atomic(
        &mut self,
        txn: StateTxn,
        mut record: InterventionRecord,
    ) -> Result<(State, ORID), JournalError> {
        // Enforce missing after-observation rule
        if record.after_observation_root.is_none() {
            record.outcome = InterventionOutcome::Unknown;
        }

        let record_id = record.id;

        // Execute commit on base state
        let committed_state = txn
            .commit()
            .map_err(|e| JournalError::TxnCommitFailed(format!("{:?}", e)))?;

        // Append to journal ONLY on successful commit
        self.entries.push(record);

        Ok((committed_state, record_id))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_dummy_receipt() -> EnvironmentReceipt {
        EnvironmentReceipt::new("node_worker_01", 0, b"no_errors")
    }

    #[test]
    fn test_100_percent_interventions_generate_receipt_and_entry() {
        let mut journal = InterventionJournal::new();
        let before = ORID::compute(ObjectKind::Commit, b"state_t0");
        let action = ORID::compute(ObjectKind::Operator, b"action_a");
        let receipt = create_dummy_receipt();
        let after = ORID::compute(ObjectKind::Commit, b"state_t1");

        let record = InterventionRecord::new(
            before,
            action,
            "exec_capability",
            receipt.clone(),
            Some(after),
            InterventionOutcome::Success,
            1000,
            2000,
        );

        let res = journal.append(record);
        assert!(res.is_ok());
        assert_eq!(journal.len(), 1);
        assert_eq!(journal.entries[0].environment_receipt, receipt);
    }

    #[test]
    fn test_missing_after_observation_marks_outcome_unknown_never_success() {
        let before = ORID::compute(ObjectKind::Commit, b"state_t0");
        let action = ORID::compute(ObjectKind::Operator, b"action_a");
        let receipt = create_dummy_receipt();

        // Requested outcome is Success, BUT after_observation_root is None
        let record = InterventionRecord::new(
            before,
            action,
            "exec_capability",
            receipt,
            None, // Missing after observation!
            InterventionOutcome::Success,
            1000,
            2000,
        );

        assert_eq!(
            record.outcome,
            InterventionOutcome::Unknown,
            "Outcome MUST be downgraded to Unknown when after_observation is missing"
        );
    }

    #[test]
    fn test_journal_append_atomic_with_commit() {
        let mut journal = InterventionJournal::new();
        let state = State::new();
        let txn = StateTxn::new(state);

        let before = ORID::compute(ObjectKind::Commit, b"state_t0");
        let action = ORID::compute(ObjectKind::Operator, b"action_a");
        let receipt = create_dummy_receipt();
        let after = ORID::compute(ObjectKind::Commit, b"state_t1");

        let record = InterventionRecord::new(
            before,
            action,
            "exec_capability",
            receipt,
            Some(after),
            InterventionOutcome::Success,
            1000,
            2000,
        );

        let (committed_state, _id) = journal.append_atomic(txn, record).unwrap();
        assert_eq!(journal.len(), 1);
        assert!(committed_state.is_schema_valid());
    }
}
