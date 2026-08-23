#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO — Security Red-Team Regression Matrix
// INVARIANT: All 6 threat vectors (forged ORIDs, provenance laundering, capability escalation, data-instruction confusion, stale artifacts, malicious OIR) must yield 0 critical bypasses.

pub mod capability_escalation;
pub mod data_instruction_confusion;
pub mod forged_orid;
pub mod malicious_oir;
pub mod provenance;
pub mod stale_artifact;

pub fn run_all_redteam_tests() -> bool {
    forged_orid::test_forged_orid_rejection()
        && provenance::test_provenance_laundering_rejection()
        && capability_escalation::test_capability_escalation_rejection()
        && data_instruction_confusion::test_data_instruction_confusion_isolation()
        && stale_artifact::test_stale_artifact_rejection()
        && malicious_oir::test_malicious_oir_rejection()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_redteam_matrix_zero_bypasses() {
        assert!(run_all_redteam_tests(), "Full Red-Team Matrix MUST pass with 0 critical bypasses");
    }
}
