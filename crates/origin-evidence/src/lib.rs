#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO Subsystem: origin-evidence
// Runtime evidence object structures, source identity, acquisition methods, trust domains, provenance hypergraph, correlation deduplicator, and trust policy engine.

pub mod correlation;
pub mod evidence;
pub mod provenance;
pub mod trust;

pub use correlation::CorrelationDeduplicator;
pub use evidence::{EvidenceError, EvidenceRecord};
pub use provenance::{Derivation, LineageProof, ProvenanceError, ProvenanceHypergraph};
pub use trust::{TrustPolicy, TrustPriority};

pub fn crate_name() -> &'static str {
    "origin-evidence"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_crate_boundary() {
        assert_eq!(crate_name(), "origin-evidence");
    }
}
