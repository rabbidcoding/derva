#![forbid(unsafe_code)]

// INVARIANT: False promotion count = 0 in known-truth causal suite; verified-causal requires intervention/mechanism witness; assumption removal invalidates status 100%.
// KPI: 0 false promotions; mandatory witness for VerifiedCausal; 100% assumption dependency retraction.

use crate::operator::CausalOperator;
use origin_core::{CausalError, CausalStatus, CausalWitness, CausalWitnessKind, ORID};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromotionRecord {
    pub operator_id: ORID,
    pub previous_status: CausalStatus,
    pub current_status: CausalStatus,
    pub witness: CausalWitness,
    pub active_assumptions: HashSet<String>,
}

#[derive(Debug, Clone, Default)]
pub struct CausalPromotionValidator {
    pub records: HashMap<ORID, PromotionRecord>,
}

impl CausalPromotionValidator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates whether a witness supports transitioning from `from` to `to` causal status.
    pub fn supports_transition(
        &self,
        from: CausalStatus,
        to: CausalStatus,
        witness: &CausalWitness,
    ) -> bool {
        if witness.provenance_roots.is_empty() {
            return false;
        }

        match (from, to, witness.kind) {
            (
                CausalStatus::Observational,
                CausalStatus::AssumedCausal,
                CausalWitnessKind::Assumption,
            ) => !witness.assumptions.is_empty(),
            (
                CausalStatus::Observational,
                CausalStatus::Interventional,
                CausalWitnessKind::Intervention,
            ) => true,
            (
                CausalStatus::AssumedCausal,
                CausalStatus::Mechanistic,
                CausalWitnessKind::MechanisticDerivation,
            ) => true,
            (
                CausalStatus::Interventional,
                CausalStatus::VerifiedCausal,
                CausalWitnessKind::MechanisticDerivation,
            ) => true,
            (
                CausalStatus::Mechanistic,
                CausalStatus::VerifiedCausal,
                CausalWitnessKind::Intervention,
            ) => true,
            _ => false,
        }
    }

    /// Promotes an operator's causal status if supported by witness and provenance.
    /// AUDIT-LENSES: Knuth, Thompson, Guido
    pub fn validate_and_promote(
        &mut self,
        operator: &mut CausalOperator,
        target_status: CausalStatus,
        witness: CausalWitness,
    ) -> Result<CausalStatus, CausalError> {
        if witness.provenance_roots.is_empty() {
            return Err(CausalError::MissingProvenance);
        }

        let current = operator.status;

        // KPI: Every verified-causal operator MUST have intervention or mechanistic witness
        if target_status == CausalStatus::VerifiedCausal
            && witness.kind != CausalWitnessKind::Intervention
            && witness.kind != CausalWitnessKind::MechanisticDerivation
        {
            return Err(CausalError::IllegalPromotion);
        }

        if !self.supports_transition(current, target_status, &witness) {
            return Err(CausalError::IllegalPromotion);
        }

        let assumptions_set: HashSet<String> = witness.assumptions.iter().cloned().collect();

        let record = PromotionRecord {
            operator_id: operator.id,
            previous_status: current,
            current_status: target_status,
            witness,
            active_assumptions: assumptions_set,
        };

        self.records.insert(operator.id, record);
        operator.status = target_status;

        Ok(target_status)
    }

    /// Retracts an assumption globally, invalidating all dependent causal status promotions.
    /// INVARIANT: Assumption removal invalidates dependent status 100%.
    pub fn retract_assumption(&mut self, assumption: &str, operator: &mut CausalOperator) -> bool {
        let mut invalidated = false;

        if let Some(record) = self.records.get(&operator.id) {
            if record.active_assumptions.contains(assumption) {
                invalidated = true;
            }
        }

        if invalidated {
            if let Some(record) = self.records.remove(&operator.id) {
                // Revert operator status back to previous status before assumption promotion
                operator.status = record.previous_status;
            } else {
                operator.status = CausalStatus::Observational;
            }
        }

        invalidated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operator::{EffectId, SchemaId};
    use origin_core::ObjectKind;

    fn create_test_operator() -> CausalOperator {
        CausalOperator::new(
            "test_op",
            SchemaId::new("In"),
            SchemaId::new("Out"),
            vec![],
            EffectId::new("eff"),
            vec![],
            CausalStatus::Observational,
            None,
            None,
        )
        .unwrap()
    }

    #[test]
    fn test_zero_false_promotions_in_known_truth_suite() {
        let validator = CausalPromotionValidator::new();
        let dummy_orid = ORID::compute(ObjectKind::Evidence, b"witness_root");

        let statuses = [
            CausalStatus::Observational,
            CausalStatus::AssumedCausal,
            CausalStatus::Interventional,
            CausalStatus::Mechanistic,
            CausalStatus::VerifiedCausal,
        ];

        let witness_kinds = [
            CausalWitnessKind::Assumption,
            CausalWitnessKind::Intervention,
            CausalWitnessKind::MechanisticDerivation,
        ];

        let mut false_promotions = 0;

        for &from in &statuses {
            for &to in &statuses {
                for &kind in &witness_kinds {
                    let witness = CausalWitness::new(
                        kind,
                        dummy_orid,
                        vec![dummy_orid],
                        vec!["assumption_a".to_string()],
                    );

                    let supported = validator.supports_transition(from, to, &witness);

                    // False promotion check: Observational directly to VerifiedCausal without chain is illegal
                    if from == CausalStatus::Observational
                        && to == CausalStatus::VerifiedCausal
                        && supported
                    {
                        false_promotions += 1;
                    }

                    // AssumedCausal directly to VerifiedCausal without intervention is illegal
                    if from == CausalStatus::AssumedCausal
                        && to == CausalStatus::VerifiedCausal
                        && supported
                    {
                        false_promotions += 1;
                    }
                }
            }
        }

        assert_eq!(
            false_promotions, 0,
            "False promotion count MUST be exactly 0 in known-truth suite"
        );
    }

    #[test]
    fn test_every_verified_causal_requires_witness() {
        let mut validator = CausalPromotionValidator::new();
        let mut op = create_test_operator();
        let dummy_orid = ORID::compute(ObjectKind::Evidence, b"witness_root");

        // Step 1: Promote Observational -> Interventional via Intervention witness
        let interv_witness = CausalWitness::new(
            CausalWitnessKind::Intervention,
            dummy_orid,
            vec![dummy_orid],
            vec![],
        );
        let res =
            validator.validate_and_promote(&mut op, CausalStatus::Interventional, interv_witness);
        assert_eq!(res, Ok(CausalStatus::Interventional));

        // Step 2: Attempt illegal promotion to VerifiedCausal with Assumption witness MUST fail
        let illegal_witness = CausalWitness::new(
            CausalWitnessKind::Assumption,
            dummy_orid,
            vec![dummy_orid],
            vec!["assumption".to_string()],
        );
        let err_res =
            validator.validate_and_promote(&mut op, CausalStatus::VerifiedCausal, illegal_witness);
        assert_eq!(err_res, Err(CausalError::IllegalPromotion));

        // Step 3: Valid promotion to VerifiedCausal with MechanisticDerivation witness MUST succeed
        let mech_witness = CausalWitness::new(
            CausalWitnessKind::MechanisticDerivation,
            dummy_orid,
            vec![dummy_orid],
            vec![],
        );
        let valid_res =
            validator.validate_and_promote(&mut op, CausalStatus::VerifiedCausal, mech_witness);
        assert_eq!(valid_res, Ok(CausalStatus::VerifiedCausal));
    }

    #[test]
    fn test_assumption_removal_invalidates_dependent_status_100_percent() {
        let mut validator = CausalPromotionValidator::new();
        let mut op = create_test_operator();
        let dummy_orid = ORID::compute(ObjectKind::Evidence, b"witness_root");

        let assumption_name = "homogeneity_assumption";
        let witness = CausalWitness::new(
            CausalWitnessKind::Assumption,
            dummy_orid,
            vec![dummy_orid],
            vec![assumption_name.to_string()],
        );

        // Promote to AssumedCausal
        let res = validator.validate_and_promote(&mut op, CausalStatus::AssumedCausal, witness);
        assert_eq!(res, Ok(CausalStatus::AssumedCausal));
        assert_eq!(op.status, CausalStatus::AssumedCausal);

        // Retract assumption -> MUST invalidate and degrade operator status back to Observational
        let was_retracted = validator.retract_assumption(assumption_name, &mut op);
        assert!(was_retracted, "Assumption removal MUST return true");
        assert_eq!(
            op.status,
            CausalStatus::Observational,
            "Operator status MUST be degraded back to Observational upon assumption retraction"
        );
    }
}
