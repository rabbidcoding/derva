// AUDIT-LENSES: Thompson, Guido, Berners-Lee
// INVARIANT: Red-team regression test verifying forged ORID detection; 0 bypasses allowed.

use origin_core::{ObjectKind, ORID};

pub fn test_forged_orid_rejection() -> bool {
    let payload = b"authentic_claim_payload_data";
    let authentic_orid = ORID::compute(ObjectKind::Claim, payload);

    // Threat Scenario 1: Mutate 1 byte of payload
    let mut forged_payload = payload.to_vec();
    forged_payload[0] ^= 0xFF;
    let forged_payload_orid = ORID::compute(ObjectKind::Claim, &forged_payload);

    // Threat Scenario 2: Change ObjectKind tag while keeping same payload
    let forged_kind_orid = ORID::compute(ObjectKind::Evidence, payload);

    // Verification: Both forged ORIDs MUST differ from authentic ORID
    assert_ne!(authentic_orid, forged_payload_orid, "Forged payload ORID MUST not match authentic ORID");
    assert_ne!(authentic_orid, forged_kind_orid, "Forged kind ORID MUST not match authentic ORID");

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redteam_test_forged_orid() {
        assert!(test_forged_orid_rejection());
    }
}
