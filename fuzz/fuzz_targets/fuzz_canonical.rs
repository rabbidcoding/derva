#![no_main]
// AUDIT-LENSES: Ken Thompson, Donald Knuth, Dennis Ritchie
// INVARIANT: Fuzz target for Canonical Decoder & ORID parsing; 0 crashes allowed.

use libfuzzer_sys::fuzz_target;
use origin_core::{ObjectKind, ORID};

fuzz_target!(|data: &[u8]| {
    // Attempt computing ORID for arbitrary payload bytes across all object kinds
    let kinds = [
        ObjectKind::Claim,
        ObjectKind::Evidence,
        ObjectKind::Obligation,
        ObjectKind::StateNode,
        ObjectKind::Commit,
        ObjectKind::Artifact,
    ];

    for kind in kinds {
        let digest = ORID::compute(kind, data);
        assert_eq!(digest.kind, kind);
        assert_eq!(digest.hash.len(), 32);
    }
});
