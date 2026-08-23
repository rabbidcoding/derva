#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO Subsystem: origin-evidence
// Runtime evidence object structures, source identity, acquisition methods, trust domains, provenance hypergraph, and correlation deduplicator.

pub mod correlation;
pub mod evidence;
pub mod provenance;

pub use correlation::CorrelationDeduplicator;
pub use evidence::{EvidenceError, EvidenceRecord};
pub use provenance::{Derivation, LineageProof, ProvenanceError, ProvenanceHypergraph};

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
