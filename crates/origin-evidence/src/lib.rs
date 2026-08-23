#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO Subsystem: origin-evidence
// Runtime evidence object structures, source identity, acquisition methods, and trust domains.

pub mod evidence;

pub use evidence::{EvidenceError, EvidenceRecord};

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
