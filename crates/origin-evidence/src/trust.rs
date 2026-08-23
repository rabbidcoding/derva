// INVARIANT: Trust score modifies search/execution priority only; trust = 1.0 NEVER bypasses derivation to grant VERIFIED status.
// KPI: trust=1.0 never creates VERIFIED without derivation; policy changes versioned by ORID; raw provenance is policy-invariant.

use crate::evidence::EvidenceRecord;
use origin_core::{Canonical, ObjectKind, Status, ORID};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct TrustPriority {
    pub scheduling_score: f64,
    pub source_reliability: f64,
    pub transport_integrity: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustPolicy {
    pub version: u64,
    pub source_scores: BTreeMap<String, f64>,
    pub domain_weights: BTreeMap<String, f64>,
}

impl TrustPolicy {
    pub fn new(version: u64) -> Self {
        Self {
            version,
            source_scores: BTreeMap::new(),
            domain_weights: BTreeMap::new(),
        }
    }

    pub fn set_source_score(&mut self, source: impl Into<String>, score: f64) {
        let clamped = score.clamp(0.0, 1.0);
        self.source_scores.insert(source.into(), clamped);
    }

    pub fn set_domain_weight(&mut self, domain: impl Into<String>, weight: f64) {
        let clamped = weight.clamp(0.0, 1.0);
        self.domain_weights.insert(domain.into(), clamped);
    }

    pub fn id(&self) -> ORID {
        let mut buf = Vec::new();
        self.encode_canonical(&mut buf);
        ORID::compute(ObjectKind::Artifact, &buf)
    }

    /// Evaluates execution/search priority for evidence.
    /// INVARIANT: Does NOT mutate epistemic status or grant VERIFIED status.
    pub fn rank(&self, evidence: &EvidenceRecord) -> TrustPriority {
        let source_rel = self
            .source_scores
            .get(&evidence.source_identity)
            .cloned()
            .unwrap_or(0.5);

        let domain_w = self
            .domain_weights
            .get(&evidence.trust_domain)
            .cloned()
            .unwrap_or(0.5);

        let transport_int = if evidence.raw_object_orid.is_some() {
            1.0
        } else {
            0.5
        };

        let scheduling_score = (source_rel * 0.4) + (domain_w * 0.4) + (transport_int * 0.2);

        TrustPriority {
            scheduling_score,
            source_reliability: source_rel,
            transport_integrity: transport_int,
        }
    }

    /// Evaluates if an evidence item can achieve VERIFIED status under this policy.
    /// INVARIANT: Even if trust = 1.0, returns Status::Supported (or Unknown) if raw_object_orid or derivation is missing.
    pub fn evaluate_epistemic_status(
        &self,
        evidence: &EvidenceRecord,
        has_formal_derivation: bool,
    ) -> Status {
        if has_formal_derivation && evidence.can_support_verified() {
            Status::Verified
        } else if evidence.validate().is_ok() {
            Status::Supported
        } else {
            Status::Unknown
        }
    }
}

impl Canonical for TrustPolicy {
    fn encode_canonical(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.version.to_be_bytes());

        out.extend_from_slice(&(self.source_scores.len() as u64).to_be_bytes());
        for (k, v) in &self.source_scores {
            let k_bytes = k.as_bytes();
            out.extend_from_slice(&(k_bytes.len() as u64).to_be_bytes());
            out.extend_from_slice(k_bytes);
            out.extend_from_slice(&v.to_be_bytes());
        }

        out.extend_from_slice(&(self.domain_weights.len() as u64).to_be_bytes());
        for (k, v) in &self.domain_weights {
            let k_bytes = k.as_bytes();
            out.extend_from_slice(&(k_bytes.len() as u64).to_be_bytes());
            out.extend_from_slice(k_bytes);
            out.extend_from_slice(&v.to_be_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::EvidenceRecord;
    use origin_core::ObjectKind;

    #[test]
    fn test_trust_one_point_zero_never_creates_verified_without_derivation() {
        let mut policy = TrustPolicy::new(1);
        policy.set_source_score("oracle_alpha", 1.0);
        policy.set_domain_weight("trust_max", 1.0);

        let ev = EvidenceRecord::new(
            Some(ORID::compute(ObjectKind::Claim, b"raw_data")),
            "oracle_alpha",
            "ingest::direct",
            1700000000,
            "correlation_a",
            "trust_max",
        )
        .unwrap();

        let priority = policy.rank(&ev);
        assert_eq!(priority.source_reliability, 1.0);
        assert_eq!(priority.scheduling_score, 1.0);

        // Without formal derivation, status CANNOT be VERIFIED even with trust = 1.0
        let status_no_deriv = policy.evaluate_epistemic_status(&ev, false);
        assert_ne!(status_no_deriv, Status::Verified);
        assert_eq!(status_no_deriv, Status::Supported);

        // With formal derivation, status can be VERIFIED
        let status_with_deriv = policy.evaluate_epistemic_status(&ev, true);
        assert_eq!(status_with_deriv, Status::Verified);
    }

    #[test]
    fn test_policy_changes_versioned_by_orid() {
        let mut policy_v1 = TrustPolicy::new(1);
        policy_v1.set_source_score("source_a", 0.8);

        let mut policy_v2 = TrustPolicy::new(2);
        policy_v2.set_source_score("source_a", 0.95);

        assert_ne!(policy_v1.id(), policy_v2.id());
    }

    #[test]
    fn test_different_trust_policies_do_not_alter_raw_provenance_id() {
        let mut policy_a = TrustPolicy::new(1);
        policy_a.set_source_score("source_x", 0.1);

        let mut policy_b = TrustPolicy::new(1);
        policy_b.set_source_score("source_x", 1.0);

        let ev = EvidenceRecord::new(
            Some(ORID::compute(ObjectKind::Claim, b"raw")),
            "source_x",
            "ingest::direct",
            1700000000,
            "corr",
            "trust",
        )
        .unwrap();

        let ev_id_before = ev.id();
        let _p_a = policy_a.rank(&ev);
        let _p_b = policy_b.rank(&ev);
        let ev_id_after = ev.id();

        assert_eq!(ev_id_before, ev_id_after);
    }
}
