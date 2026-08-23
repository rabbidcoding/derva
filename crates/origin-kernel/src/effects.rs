// INVARIANT: All side-effecting opcodes require capability verification and produce authoritative effect receipts (principal+capability+commit).
// KPI: 0 unauthorized side effects; 100% effecting operations logged.

use crate::capability::{
    Capability, CapabilityError, CommitEffect, Intervene, Pure, QueryExternal, ReadOnly,
};
use origin_core::{OpCode, ORID};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectReceipt {
    pub opcode: OpCode,
    pub principal: String,
    pub capability_id: ORID,
    pub commit_id: ORID,
    pub timestamp: u64,
}

#[derive(Debug, Default)]
pub struct EffectGate {
    pub effect_log: Vec<EffectReceipt>,
}

impl EffectGate {
    pub fn new() -> Self {
        Self {
            effect_log: Vec::new(),
        }
    }

    /// Pure operation execution — 0 capability required, zero side-effects allowed.
    pub fn execute_pure<F, R>(&self, _cap: &Capability<Pure>, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        f()
    }

    /// Read-Only operation execution — scope checked.
    pub fn execute_read_only<F, R>(
        &self,
        cap: &Capability<ReadOnly>,
        target_scope: &str,
        f: F,
    ) -> Result<R, CapabilityError>
    where
        F: FnOnce() -> R,
    {
        if cap.scope != target_scope && cap.scope != "*" {
            return Err(CapabilityError::ScopeMismatch {
                expected: target_scope.to_string(),
                found: cap.scope.clone(),
            });
        }
        Ok(f())
    }

    /// Query External operation — requires Capability<QueryExternal>
    pub fn execute_query_external(
        &mut self,
        cap: &Capability<QueryExternal>,
        commit_id: &ORID,
        timestamp: u64,
    ) -> Result<EffectReceipt, CapabilityError> {
        if cap.principal.is_empty() {
            return Err(CapabilityError::Unauthorized("Empty principal".to_string()));
        }

        let receipt = EffectReceipt {
            opcode: OpCode::Query,
            principal: cap.principal.clone(),
            capability_id: cap.token_id,
            commit_id: *commit_id,
            timestamp,
        };

        self.effect_log.push(receipt.clone());
        Ok(receipt)
    }

    /// Intervene operation — requires Capability<Intervene>
    pub fn execute_intervene(
        &mut self,
        cap: &Capability<Intervene>,
        commit_id: &ORID,
        timestamp: u64,
    ) -> Result<EffectReceipt, CapabilityError> {
        if cap.principal.is_empty() {
            return Err(CapabilityError::Unauthorized("Empty principal".to_string()));
        }

        let receipt = EffectReceipt {
            opcode: OpCode::Intervene,
            principal: cap.principal.clone(),
            capability_id: cap.token_id,
            commit_id: *commit_id,
            timestamp,
        };

        self.effect_log.push(receipt.clone());
        Ok(receipt)
    }

    /// Commit operation — requires Capability<CommitEffect>
    pub fn execute_commit(
        &mut self,
        cap: &Capability<CommitEffect>,
        commit_id: &ORID,
        timestamp: u64,
    ) -> Result<EffectReceipt, CapabilityError> {
        if cap.principal.is_empty() {
            return Err(CapabilityError::Unauthorized("Empty principal".to_string()));
        }

        let receipt = EffectReceipt {
            opcode: OpCode::Commit,
            principal: cap.principal.clone(),
            capability_id: cap.token_id,
            commit_id: *commit_id,
            timestamp,
        };

        self.effect_log.push(receipt.clone());
        Ok(receipt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::CapabilityMint;
    use origin_core::ObjectKind;

    #[test]
    fn test_adversarial_sequence_unauthorized_effects_zero() {
        let mut gate = EffectGate::new();
        let commit_id = ORID::compute(ObjectKind::Commit, b"c1");

        let cap_intervene = CapabilityMint::mint_intervene("admin", "global");
        let receipt = gate
            .execute_intervene(&cap_intervene, &commit_id, 1000)
            .unwrap();

        assert_eq!(receipt.opcode, OpCode::Intervene);
        assert_eq!(receipt.principal, "admin");
        assert_eq!(receipt.commit_id, commit_id);
        assert_eq!(gate.effect_log.len(), 1);

        // Verify empty principal fails with Unauthorized error
        let cap_invalid = CapabilityMint::mint_intervene("", "scope");
        assert!(gate
            .execute_intervene(&cap_invalid, &commit_id, 1001)
            .is_err());
    }

    #[test]
    fn test_pure_and_readonly_cannot_escalate_effects() {
        let gate = EffectGate::new();
        let cap_pure = CapabilityMint::mint_pure("agent");
        let res = gate.execute_pure(&cap_pure, || 42);
        assert_eq!(res, 42);

        let cap_read = CapabilityMint::mint_read_only("agent", "scope_a");
        assert!(gate.execute_read_only(&cap_read, "scope_b", || 10).is_err());
        assert_eq!(
            gate.execute_read_only(&cap_read, "scope_a", || 10).unwrap(),
            10
        );
    }
}
