// AUDIT-LENSES: Thompson, Guido, Berners-Lee
// INVARIANT: Red-team regression test verifying immediate detection & rejection of stale artifacts within same operation.

use origin_compiler::artifact::{CompiledArtifact, DomainGuard};
use origin_core::{ObjectKind, ORID};

pub fn test_stale_artifact_rejection() -> bool {
    let fresh_dep_root = ORID::compute(ObjectKind::Claim, b"fresh_dep_root_v1");
    let stale_dep_root = ORID::compute(ObjectKind::Claim, b"stale_dep_root_v0");

    let artifact = CompiledArtifact {
        artifact_id: "art_stale_attack".into(),
        dep_root: fresh_dep_root,
        schema_hash: 0xDEADBEEF,
        guard: DomainGuard {
            min_value: 0.0,
            max_value: 100.0,
        },
    };

    // Threat Scenario: Attacker attempts to execute compiled artifact against outdated dependency root
    let acquire_result = artifact.validate_and_acquire(stale_dep_root, 0xDEADBEEF, 50.0);

    assert!(
        acquire_result.is_err(),
        "Artifact validation MUST fail immediately within same operation when dependency root is stale"
    );

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redteam_test_stale_artifact() {
        assert!(test_stale_artifact_rejection());
    }
}
