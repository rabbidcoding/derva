// AUDIT-LENSES: Thompson, Guido, Berners-Lee
// INVARIANT: Red-team regression test verifying provenance laundering rejection; 0 orphan commits accepted.

use origin_core::{ObjectKind, ORID};
use origin_store::commit::{CommitDag, CommitNode};

pub fn test_provenance_laundering_rejection() -> bool {
    let mut dag = CommitDag::new();
    let policy = ORID::compute(ObjectKind::Artifact, b"policy_v1");

    // Real genesis commit
    let genesis = dag.insert(CommitNode::new(
        vec![],
        ORID::compute(ObjectKind::Claim, b"genesis"),
        policy,
        "admin",
        1,
    ));

    // Threat Scenario: Inject orphan commit referencing fake parent ORID
    let fake_parent = ORID::compute(ObjectKind::Commit, b"non_existent_fake_parent_orid");
    let laundered_commit = CommitNode::new(
        vec![fake_parent],
        ORID::compute(ObjectKind::Claim, b"laundered_claim"),
        policy,
        "attacker",
        2,
    );
    let laundered_id = dag.insert(laundered_commit);

    // Verification: Ancestor replay stack trace MUST fail when looking up missing parent
    let replay_result = dag.replay_ancestor_sequence(&laundered_id);
    assert!(
        replay_result.is_err(),
        "Replaying ancestor sequence of orphan commit MUST return error"
    );

    // Genesis commit replay MUST remain valid and uncorrupted
    let genesis_replay = dag.replay_ancestor_sequence(&genesis).unwrap();
    assert_eq!(genesis_replay, vec![genesis]);

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redteam_test_provenance_laundering() {
        assert!(test_provenance_laundering_rejection());
    }
}
