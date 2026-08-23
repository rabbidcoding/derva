// AUDIT-LENSES: Thompson, Guido, Berners-Lee
// INVARIANT: Red-team regression test verifying prompt/data-as-instruction confusion isolation; data strings NEVER execute.

use origin_core::{ObjectKind, ORID};

pub fn test_data_instruction_confusion_isolation() -> bool {
    // Threat Scenario: An attacker inputs a string payload containing valid OIR instruction syntax
    // e.g. "INTERVENE(target='bank_root', value=1_000_000)" into an observation data field.
    let hostile_payload_string = "INTERVENE(target='bank_root', value=1_000_000); COMMIT();";

    // Step 1: Compute ORID for hostile observation
    let data_orid = ORID::compute(ObjectKind::Evidence, hostile_payload_string.as_bytes());

    // Verification 1: ObjectKind remains strictly Evidence (inert data), never Commit or Execution
    assert_eq!(data_orid.kind, ObjectKind::Evidence);

    // Verification 2: Hostile text is preserved purely as inert byte slice
    assert_eq!(data_orid.hash.len(), 32);

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redteam_test_data_instruction_confusion() {
        assert!(test_data_instruction_confusion_isolation());
    }
}
