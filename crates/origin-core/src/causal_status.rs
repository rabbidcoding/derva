// INVARIANT: Explicit Causal Status Type Algebra. Cannot promote to verified causal without intervention witness.
// KPI: False causal promotion count = 0 in synthetic known-truth test suite.

use crate::orid::ORID;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CausalStatus {
    Observational,
    AssumedCausal,
    Interventional,
    Mechanistic,
    VerifiedCausal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CausalWitnessKind {
    Assumption,
    Intervention,
    MechanisticDerivation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CausalWitness {
    pub kind: CausalWitnessKind,
    pub witness_orid: ORID,
    pub provenance_roots: Vec<ORID>,
    pub assumptions: Vec<String>,
}

impl CausalWitness {
    pub fn new(
        kind: CausalWitnessKind,
        witness_orid: ORID,
        provenance_roots: Vec<ORID>,
        assumptions: Vec<String>,
    ) -> Self {
        Self {
            kind,
            witness_orid,
            provenance_roots,
            assumptions,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CausalError {
    IllegalPromotion,
    MissingWitness,
    MissingProvenance,
    EmptyAssumptions,
}

pub fn causal_promote(
    current: CausalStatus,
    witness: &CausalWitness,
) -> Result<CausalStatus, CausalError> {
    if witness.provenance_roots.is_empty() {
        return Err(CausalError::MissingProvenance);
    }

    match (current, witness.kind) {
        (CausalStatus::Observational, CausalWitnessKind::Assumption) => {
            if witness.assumptions.is_empty() {
                return Err(CausalError::EmptyAssumptions);
            }
            Ok(CausalStatus::AssumedCausal)
        }
        (CausalStatus::Observational, CausalWitnessKind::Intervention) => {
            Ok(CausalStatus::Interventional)
        }
        (CausalStatus::AssumedCausal, CausalWitnessKind::MechanisticDerivation) => {
            Ok(CausalStatus::Mechanistic)
        }
        (CausalStatus::Interventional, CausalWitnessKind::MechanisticDerivation) => {
            Ok(CausalStatus::VerifiedCausal)
        }
        (CausalStatus::Mechanistic, CausalWitnessKind::Intervention) => {
            Ok(CausalStatus::VerifiedCausal)
        }
        _ => Err(CausalError::IllegalPromotion),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orid::ObjectKind;

    #[test]
    fn test_zero_observational_to_verified_causal_without_chain() {
        let dummy_orid = ORID::compute(ObjectKind::Evidence, b"witness");
        let witness_interv = CausalWitness::new(
            CausalWitnessKind::Intervention,
            dummy_orid,
            vec![dummy_orid],
            vec![],
        );

        // Direct Observational + Intervention gives Interventional, NEVER VerifiedCausal
        let res = causal_promote(CausalStatus::Observational, &witness_interv);
        assert_eq!(res, Ok(CausalStatus::Interventional));

        let witness_mech = CausalWitness::new(
            CausalWitnessKind::MechanisticDerivation,
            dummy_orid,
            vec![dummy_orid],
            vec![],
        );
        // Interventional + Mechanistic gives VerifiedCausal
        let res_v = causal_promote(CausalStatus::Interventional, &witness_mech);
        assert_eq!(res_v, Ok(CausalStatus::VerifiedCausal));
    }

    #[test]
    fn test_missing_provenance_rejected() {
        let dummy_orid = ORID::compute(ObjectKind::Evidence, b"witness");
        let invalid_witness = CausalWitness::new(
            CausalWitnessKind::Intervention,
            dummy_orid,
            vec![], // Empty provenance
            vec![],
        );

        let res = causal_promote(CausalStatus::Observational, &invalid_witness);
        assert_eq!(res, Err(CausalError::MissingProvenance));
    }

    #[test]
    fn test_known_truth_synthetic_suite_zero_false_promotions() {
        let dummy_orid = ORID::compute(ObjectKind::Evidence, b"synthetic");
        let kinds = [
            CausalWitnessKind::Assumption,
            CausalWitnessKind::Intervention,
            CausalWitnessKind::MechanisticDerivation,
        ];
        let statuses = [
            CausalStatus::Observational,
            CausalStatus::AssumedCausal,
            CausalStatus::Interventional,
            CausalStatus::Mechanistic,
            CausalStatus::VerifiedCausal,
        ];

        let mut false_promotions = 0;
        for &s in &statuses {
            for &k in &kinds {
                let w = CausalWitness::new(
                    k,
                    dummy_orid,
                    vec![dummy_orid],
                    vec!["assumption_1".to_string()],
                );
                if let Ok(next) = causal_promote(s, &w) {
                    // False promotion check: Observational directly becoming VerifiedCausal is illegal
                    if s == CausalStatus::Observational && next == CausalStatus::VerifiedCausal {
                        false_promotions += 1;
                    }
                }
            }
        }

        assert_eq!(
            false_promotions, 0,
            "False causal promotions count must be zero"
        );
    }
}
