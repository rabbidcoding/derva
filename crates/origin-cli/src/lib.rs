#![forbid(unsafe_code)]

// AUDIT-LENSES: Steve Jobs, Donald Knuth, Guido van Rossum, Bill Gates
// INVARIANT: ORIGIN-Ω ZERO CLI & Epistemic Debugger engine library.

pub mod debugger;

#[cfg(test)]
mod tests {
    use super::debugger::EpistemicDebugger;

    #[test]
    fn test_explain_why_100_percent_verified_claims_explainable() {
        let explanation = EpistemicDebugger::explain_why("test_claim_01");
        assert!(explanation.is_verified);
        assert!(!explanation.evidence_orids.is_empty());
        assert!(!explanation.verification_chain.is_empty());
    }

    #[test]
    fn test_replay_commit_reproducible_parity() {
        let res = EpistemicDebugger::replay_commit("commit_root_01", true);
        assert!(res.verified_parity);
        assert!(res.steps_replayed > 0);
    }
}
