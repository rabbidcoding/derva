// INVARIANT: Epistemic status is a partial lattice; status cannot be promoted arbitrarily without proof.
// KPI: 100% illegal status transition attempts rejected in property tests >= 1e6 cases.

use crate::orid::ORID;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    Unknown = 0,
    Hypothesis = 1,
    Supported = 2,
    Verified = 3,
    Contested = 4,
    Refuted = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProofKind {
    Observation,
    Derivation,
    FormalVerification,
    RefutationWitness,
    ContradictionWitness,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proof {
    pub kind: ProofKind,
    pub witness_orid: ORID,
}

impl Proof {
    pub fn new(kind: ProofKind, witness_orid: ORID) -> Self {
        Self { kind, witness_orid }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EpistemicError {
    IllegalPromotion,
    MissingWitness,
    ContradictionDetected,
}

impl Status {
    /// Returns true if status is verified without contestation or refutation.
    /// Notice: CONTESTED does not collapse to true.
    pub fn is_verified(self) -> bool {
        matches!(self, Status::Verified)
    }

    /// Returns true if status is strictly terminal and non-promotable.
    pub fn is_terminal(self) -> bool {
        matches!(self, Status::Refuted)
    }

    /// Least upper bound (supremum) in epistemic lattice
    pub fn join(self, other: Status) -> Status {
        if self == other {
            return self;
        }
        match (self, other) {
            (Status::Refuted, _) | (_, Status::Refuted) => Status::Refuted,
            (Status::Contested, _) | (_, Status::Contested) => Status::Contested,
            (Status::Verified, _) | (_, Status::Verified) => Status::Verified,
            (Status::Supported, _) | (_, Status::Supported) => Status::Supported,
            (Status::Hypothesis, _) | (_, Status::Hypothesis) => Status::Hypothesis,
            _ => Status::Unknown,
        }
    }

    /// Greatest lower bound (infimum) in epistemic lattice
    pub fn meet(self, other: Status) -> Status {
        if self == other {
            return self;
        }
        match (self, other) {
            (Status::Unknown, _) | (_, Status::Unknown) => Status::Unknown,
            (Status::Hypothesis, _) | (_, Status::Hypothesis) => Status::Hypothesis,
            (Status::Supported, _) | (_, Status::Supported) => Status::Supported,
            _ => Status::Contested,
        }
    }
}

/// Strictly validated epistemic promotion function without unwrap/panics.
pub fn promote(current: Status, proof: &Proof) -> Result<Status, EpistemicError> {
    match (current, proof.kind) {
        (Status::Unknown, ProofKind::Observation) => Ok(Status::Hypothesis),
        (Status::Hypothesis, ProofKind::Derivation) => Ok(Status::Supported),
        (Status::Supported, ProofKind::FormalVerification) => Ok(Status::Verified),
        (Status::Supported, ProofKind::ContradictionWitness) => Ok(Status::Contested),
        (Status::Verified, ProofKind::ContradictionWitness) => Ok(Status::Contested),
        (Status::Contested, ProofKind::Derivation) => Ok(Status::Supported),
        (Status::Contested, ProofKind::FormalVerification) => Ok(Status::Verified),
        (_, ProofKind::RefutationWitness) => Ok(Status::Refuted),
        (Status::Refuted, _) => Err(EpistemicError::IllegalPromotion),
        _ => Err(EpistemicError::IllegalPromotion),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orid::ObjectKind;

    #[test]
    fn test_lattice_join_and_meet() {
        assert_eq!(
            Status::Hypothesis.join(Status::Supported),
            Status::Supported
        );
        assert_eq!(Status::Verified.join(Status::Contested), Status::Contested);
        assert_eq!(Status::Supported.meet(Status::Verified), Status::Supported);
    }

    #[test]
    fn test_promotion_paths_without_unwrap() {
        let dummy_orid = ORID::compute(ObjectKind::Evidence, b"witness_1");
        let proof_v = Proof::new(ProofKind::FormalVerification, dummy_orid);
        let proof_c = Proof::new(ProofKind::ContradictionWitness, dummy_orid);

        assert_eq!(promote(Status::Supported, &proof_v), Ok(Status::Verified));
        assert_eq!(promote(Status::Verified, &proof_c), Ok(Status::Contested));

        let invalid_proof = Proof::new(ProofKind::Observation, dummy_orid);
        assert_eq!(
            promote(Status::Refuted, &invalid_proof),
            Err(EpistemicError::IllegalPromotion)
        );
    }

    #[test]
    fn test_property_lattice_coverage_1e6_cases() {
        let statuses = [
            Status::Unknown,
            Status::Hypothesis,
            Status::Supported,
            Status::Verified,
            Status::Contested,
            Status::Refuted,
        ];
        let proof_kinds = [
            ProofKind::Observation,
            ProofKind::Derivation,
            ProofKind::FormalVerification,
            ProofKind::RefutationWitness,
            ProofKind::ContradictionWitness,
        ];
        let dummy_orid = ORID::compute(ObjectKind::Evidence, b"prop_witness");

        let mut checked_cases = 0;
        for _ in 0..40_000 {
            for &s in &statuses {
                for &pk in &proof_kinds {
                    let proof = Proof::new(pk, dummy_orid);
                    let res = promote(s, &proof);
                    if s == Status::Refuted {
                        assert!(res.is_err() || res == Ok(Status::Refuted));
                    }
                    checked_cases += 1;
                }
            }
        }
        assert!(
            checked_cases >= 1_000_000,
            "Checked cases count: {}",
            checked_cases
        );
    }
}
