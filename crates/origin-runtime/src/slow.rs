// AUDIT-LENSES: Alan Turing, Ken Thompson, Guido van Rossum, Niklaus Wirth
// INVARIANT: Orchestrate active slice -> reason -> query/plan -> verify -> commit pipeline with deterministic event logging & fail-closed budget control.
// KPI: 100% slow steps emit deterministic event records; budget exhaustion returns typed Stop with zero partial commits; no action before verification.

use origin_core::{ObjectKind, ORID};
use origin_kernel::budget::{ResourceBudget, StepCost};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepKind {
    ActiveSlice,
    Reason,
    QueryPlan,
    Verify,
    Commit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventRecord {
    pub step_index: usize,
    pub step_kind: StepKind,
    pub payload_orid: ORID,
    pub timestamp_ticks: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeliberationError {
    BudgetExhausted(String),
    VerificationFailed(String),
    CapabilityGateDenied(String),
}

impl std::fmt::Display for DeliberationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeliberationError::BudgetExhausted(msg) => write!(f, "Typed Stop (Budget Exhausted): {}", msg),
            DeliberationError::VerificationFailed(msg) => write!(f, "Verification Failed: {}", msg),
            DeliberationError::CapabilityGateDenied(msg) => write!(f, "Capability Gate Denied: {}", msg),
        }
    }
}

impl std::error::Error for DeliberationError {}

#[derive(Debug, Clone)]
pub struct Proposal {
    pub id: ORID,
    pub action_type: String,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct VerifiedAction {
    pub proposal_id: ORID,
    pub verification_proof: ORID,
}

#[derive(Debug, Default)]
pub struct SlowDeliberativeRuntime {
    pub event_log: Vec<EventRecord>,
    pub committed_actions: Vec<VerifiedAction>,
}

impl SlowDeliberativeRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    /// Complete deliberative pipeline execution
    pub fn deliberative_step(
        &mut self,
        goal: &str,
        budget: &mut ResourceBudget,
        verification_gate: impl Fn(&Proposal) -> bool,
        capability_gate: impl Fn(&Proposal) -> bool,
    ) -> Result<ORID, DeliberationError> {
        let initial_commits = self.committed_actions.len();

        // 1. Active Slice
        budget
            .charge(StepCost::CpuTicks(10))
            .map_err(|e| DeliberationError::BudgetExhausted(e.to_string()))?;
        let slice_orid = ORID::compute(ObjectKind::Claim, goal.as_bytes());
        self.emit_event(StepKind::ActiveSlice, slice_orid, budget.used_cpu_ticks);

        // 2. Reason
        budget
            .charge(StepCost::CpuTicks(20))
            .map_err(|e| DeliberationError::BudgetExhausted(e.to_string()))?;
        let proposal = Proposal {
            id: ORID::compute(ObjectKind::Evidence, format!("proposal:{}", goal).as_bytes()),
            action_type: "state_transition".into(),
            payload: goal.as_bytes().to_vec(),
        };
        self.emit_event(StepKind::Reason, proposal.id, budget.used_cpu_ticks);

        // 3. Query / Plan
        budget
            .charge(StepCost::Queries(1))
            .map_err(|e| DeliberationError::BudgetExhausted(e.to_string()))?;
        let plan_orid = ORID::compute(ObjectKind::Operator, proposal.payload.as_slice());
        self.emit_event(StepKind::QueryPlan, plan_orid, budget.used_cpu_ticks);

        // 4. Capability & Verification Gates (NO ACTION BEFORE VERIFICATION!)
        if !capability_gate(&proposal) {
            return Err(DeliberationError::CapabilityGateDenied(
                "Capability gate denied proposal action".into(),
            ));
        }

        budget
            .charge(StepCost::ConstraintCheck)
            .map_err(|e| DeliberationError::BudgetExhausted(e.to_string()))?;

        if !verification_gate(&proposal) {
            return Err(DeliberationError::VerificationFailed(
                "Formal verification gate rejected proposal".into(),
            ));
        }

        let proof_orid = ORID::compute(ObjectKind::Commit, b"proof_of_verification");
        self.emit_event(StepKind::Verify, proof_orid, budget.used_cpu_ticks);

        // 5. Commit (only reached after verification and budget checks pass!)
        let verified = VerifiedAction {
            proposal_id: proposal.id,
            verification_proof: proof_orid,
        };

        self.committed_actions.push(verified);
        self.emit_event(StepKind::Commit, proposal.id, budget.used_cpu_ticks);

        assert_eq!(
            self.committed_actions.len(),
            initial_commits + 1,
            "Exactly one action committed upon successful verification"
        );

        Ok(proposal.id)
    }

    fn emit_event(&mut self, step_kind: StepKind, payload_orid: ORID, ticks: u64) {
        let record = EventRecord {
            step_index: self.event_log.len(),
            step_kind,
            payload_orid,
            timestamp_ticks: ticks,
        };
        self.event_log.push(record);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_every_slow_step_emits_deterministic_event_record() {
        let mut runtime = SlowDeliberativeRuntime::new();
        let mut budget = ResourceBudget::unlimited();

        let res = runtime.deliberative_step(
            "reach_target_state",
            &mut budget,
            |_prop| true, // Allow verification
            |_prop| true, // Allow capability
        );

        assert!(res.is_ok());

        // Check exact sequence of emitted events
        let kinds: Vec<StepKind> = runtime.event_log.iter().map(|e| e.step_kind.clone()).collect();
        assert_eq!(
            kinds,
            vec![
                StepKind::ActiveSlice,
                StepKind::Reason,
                StepKind::QueryPlan,
                StepKind::Verify,
                StepKind::Commit,
            ]
        );

        // Check index continuity
        for (i, record) in runtime.event_log.iter().enumerate() {
            assert_eq!(record.step_index, i);
        }
    }

    #[test]
    fn test_budget_exhaustion_returns_typed_stop_no_partial_commit() {
        let mut runtime = SlowDeliberativeRuntime::new();
        // Budget limited to 15 ticks (fails on step 2 'Reason' which requests 20 ticks)
        let mut budget = ResourceBudget::with_limits(15, Duration::from_secs(10), 100, 100, 10);

        let res = runtime.deliberative_step(
            "exhaustion_test_goal",
            &mut budget,
            |_prop| true,
            |_prop| true,
        );

        match res {
            Err(DeliberationError::BudgetExhausted(msg)) => {
                println!("[TYPED STOP VERIFIED] {}", msg);
                assert!(msg.contains("cpu_ticks"));
            }
            _ => panic!("Must return typed BudgetExhausted error"),
        }

        // Zero partial commits!
        assert_eq!(
            runtime.committed_actions.len(),
            0,
            "NO partial commit allowed on budget exhaustion"
        );
    }

    #[test]
    fn test_no_action_executes_before_verification_gates() {
        let mut runtime = SlowDeliberativeRuntime::new();
        let mut budget = ResourceBudget::unlimited();

        // Verification gate fails!
        let res = runtime.deliberative_step(
            "unverified_goal",
            &mut budget,
            |_prop| false, // Reject verification!
            |_prop| true,
        );

        assert_eq!(
            res,
            Err(DeliberationError::VerificationFailed(
                "Formal verification gate rejected proposal".into()
            ))
        );

        // ZERO actions committed when verification fails!
        assert_eq!(
            runtime.committed_actions.len(),
            0,
            "Zero actions committed when verification fails"
        );

        // Event log stops at QueryPlan / attempt, no Commit event recorded
        let has_commit_event = runtime
            .event_log
            .iter()
            .any(|e| e.step_kind == StepKind::Commit);
        assert!(!has_commit_event, "No commit event emitted for rejected proposal");
    }
}
