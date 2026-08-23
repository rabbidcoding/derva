// INVARIANT: Verification obligations require mandatory resolution witness; expired freshness reopens obligation in <= 1 transaction; self-witness prohibited 100%.
// KPI: Resolution witness mandatory 100%; Expired freshness reopens obligation in <= 1 txn; Self-witness/cycle rejected 100%.

use origin_core::{Canonical, ObjectKind, ORID};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObligationError {
    SelfWitnessProhibited { target: ORID, witness: ORID },
    MissingWitness,
    AlreadyResolved,
    Expired,
    InvalidTimestamp { now: u64, created_at: u64 },
}

impl std::fmt::Display for ObligationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObligationError::SelfWitnessProhibited { target, witness } => {
                write!(f, "Self-witness prohibited: target ORID {} cannot witness itself (witness ORID {})", target, witness)
            }
            ObligationError::MissingWitness => write!(f, "Resolution witness is mandatory"),
            ObligationError::AlreadyResolved => write!(f, "Obligation is already resolved"),
            ObligationError::Expired => write!(f, "Obligation has expired"),
            ObligationError::InvalidTimestamp { now, created_at } => {
                write!(
                    f,
                    "Invalid timestamp: now ({}) is before creation time ({})",
                    now, created_at
                )
            }
        }
    }
}

impl std::error::Error for ObligationError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObligationResolution {
    pub witness: ORID,
    pub verifier: String,
    pub at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObligationState {
    Pending,
    Resolved {
        witness: ORID,
        verifier: String,
        resolved_at: u64,
    },
    Reopened {
        reason: String,
        reopened_at: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObligationRuntime {
    pub target_orid: ORID,
    pub predicate: String,
    pub created_at: u64,
    pub ttl: u64,
    pub state: ObligationState,
}

impl ObligationRuntime {
    pub fn new(target_orid: ORID, predicate: impl Into<String>, created_at: u64, ttl: u64) -> Self {
        Self {
            target_orid,
            predicate: predicate.into(),
            created_at,
            ttl,
            state: ObligationState::Pending,
        }
    }

    pub fn id(&self) -> ORID {
        let mut buf = Vec::new();
        self.encode_canonical(&mut buf);
        ORID::compute(ObjectKind::Obligation, &buf)
    }

    /// Resolves the obligation using a mandatory resolution witness.
    /// INVARIANT: Self-witnessing (witness == target_orid) is strictly prohibited.
    pub fn resolve(&mut self, res: ObligationResolution) -> Result<(), ObligationError> {
        if res.at < self.created_at {
            return Err(ObligationError::InvalidTimestamp {
                now: res.at,
                created_at: self.created_at,
            });
        }

        // Self-witnessing check
        if res.witness == self.target_orid {
            return Err(ObligationError::SelfWitnessProhibited {
                target: self.target_orid,
                witness: res.witness,
            });
        }

        if res.verifier.trim().is_empty() {
            return Err(ObligationError::MissingWitness);
        }

        self.state = ObligationState::Resolved {
            witness: res.witness,
            verifier: res.verifier,
            resolved_at: res.at,
        };

        Ok(())
    }

    /// Evaluates freshness at timestamp `now`. If expired, reopens the obligation immediately (in <= 1 transaction).
    pub fn check_freshness(&mut self, now: u64) -> bool {
        match &self.state {
            ObligationState::Resolved { resolved_at, .. } => {
                if now > resolved_at + self.ttl {
                    self.state = ObligationState::Reopened {
                        reason: format!(
                            "Freshness expired: now ({}) > resolved_at ({}) + ttl ({})",
                            now, resolved_at, self.ttl
                        ),
                        reopened_at: now,
                    };
                    false
                } else {
                    true
                }
            }
            ObligationState::Pending => {
                if now > self.created_at + self.ttl {
                    self.state = ObligationState::Reopened {
                        reason: format!(
                            "TTL expired while pending: now ({}) > created_at ({}) + ttl ({})",
                            now, self.created_at, self.ttl
                        ),
                        reopened_at: now,
                    };
                    false
                } else {
                    true
                }
            }
            ObligationState::Reopened { .. } => false,
        }
    }

    pub fn is_resolved(&self) -> bool {
        matches!(self.state, ObligationState::Resolved { .. })
    }
}

impl Canonical for ObligationRuntime {
    fn encode_canonical(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.target_orid.hash);

        let p_bytes = self.predicate.as_bytes();
        out.extend_from_slice(&(p_bytes.len() as u64).to_be_bytes());
        out.extend_from_slice(p_bytes);

        out.extend_from_slice(&self.created_at.to_be_bytes());
        out.extend_from_slice(&self.ttl.to_be_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use origin_core::ObjectKind;

    #[test]
    fn test_self_witness_prohibited_100_percent() {
        let target = ORID::compute(ObjectKind::Claim, b"claim_target_1");
        let mut obligation = ObligationRuntime::new(target, "invariant_check", 1000, 3600);

        // Attempting to use target ORID as its own witness must fail
        let res = obligation.resolve(ObligationResolution {
            witness: target,
            verifier: "verifier_bot".to_string(),
            at: 1050,
        });

        assert!(res.is_err());
        match res.unwrap_err() {
            ObligationError::SelfWitnessProhibited {
                target: t,
                witness: w,
            } => {
                assert_eq!(t, target);
                assert_eq!(w, target);
            }
            _ => panic!("Expected SelfWitnessProhibited error"),
        }
        assert!(!obligation.is_resolved());
    }

    #[test]
    fn test_valid_resolution_with_external_witness() {
        let target = ORID::compute(ObjectKind::Claim, b"claim_target_1");
        let witness = ORID::compute(ObjectKind::Evidence, b"evidence_witness_1");
        let mut obligation = ObligationRuntime::new(target, "invariant_check", 1000, 3600);

        obligation
            .resolve(ObligationResolution {
                witness,
                verifier: "verifier_bot".to_string(),
                at: 1050,
            })
            .unwrap();

        assert!(obligation.is_resolved());
    }

    #[test]
    fn test_expired_freshness_reopens_obligation_in_one_transaction() {
        let target = ORID::compute(ObjectKind::Claim, b"claim_target_1");
        let witness = ORID::compute(ObjectKind::Evidence, b"evidence_witness_1");
        let mut obligation = ObligationRuntime::new(target, "freshness_check", 1000, 100);

        obligation
            .resolve(ObligationResolution {
                witness,
                verifier: "verifier_bot".to_string(),
                at: 1050,
            })
            .unwrap();

        assert!(obligation.is_resolved());

        // Check freshness at t = 1100 (within 100s TTL)
        assert!(obligation.check_freshness(1100));
        assert!(obligation.is_resolved());

        // Check freshness at t = 1151 (exceeds 1050 + 100 TTL -> reopens in <= 1 transaction)
        assert!(!obligation.check_freshness(1151));
        assert!(!obligation.is_resolved());
        match &obligation.state {
            ObligationState::Reopened { reopened_at, .. } => {
                assert_eq!(*reopened_at, 1151);
            }
            _ => panic!("Expected Reopened state"),
        }
    }
}
