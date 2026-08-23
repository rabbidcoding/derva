#![forbid(unsafe_code)]

use origin_core::{ObjectKind, Status, ORID};
use origin_evidence::{CorrelationDeduplicator, Derivation, EvidenceRecord, ProvenanceHypergraph, TrustPolicy};
use origin_verify::{ContradictionEngine, ObligationResolution, ObligationRuntime};

#[test]
fn test_g03_adversarial_suite_1e6_cases_zero_illegal_promotions() {
    let mut provenance = ProvenanceHypergraph::new();
    let deduplicator = CorrelationDeduplicator::new();
    let mut contradiction_engine = ContradictionEngine::new();
    let mut trust_policy = TrustPolicy::new(1);
    trust_policy.set_source_score("adversarial_source", 1.0);
    trust_policy.set_domain_weight("adversarial_domain", 1.0);

    let root_orid = ORID::compute(ObjectKind::Evidence, b"truth_root_node");

    // Case 1: 1,000,000 synthetic adversarial cases testing promotion bounds
    let total_cases = 1_000_000;
    let mut illegal_promotions = 0;
    let mut circular_acceptances = 0;

    for i in 0..total_cases {
        let is_unsupported_case = i % 5 == 0;
        let is_self_witness_case = i % 5 == 1;
        let is_cycle_case = i % 5 == 2;
        let is_contradiction_case = i % 5 == 3;

        let ev_raw_orid = if is_unsupported_case {
            None
        } else {
            Some(root_orid)
        };

        let ev = EvidenceRecord::new(
            ev_raw_orid,
            "adversarial_source",
            "ingest::adversarial",
            1700000000 + (i as u64),
            "adversarial_domain",
            "trust_max",
        )
        .unwrap();

        // 1. Check illegal VERIFIED promotion without raw ORID
        let status = trust_policy.evaluate_epistemic_status(&ev, true);
        if is_unsupported_case && status == Status::Verified {
            illegal_promotions += 1;
        }

        // 2. Check self-witnessing obligation
        if is_self_witness_case {
            let target_orid = ev.id();
            let mut obligation = ObligationRuntime::new(target_orid, "adversarial_check", 1000, 3600);
            let res = obligation.resolve(ObligationResolution {
                witness: target_orid,
                verifier: "adversarial_bot".to_string(),
                at: 1050,
            });
            if res.is_ok() || obligation.is_resolved() {
                illegal_promotions += 1;
            }
        }

        // 3. Check cycle insertion
        if is_cycle_case {
            let n1 = ORID::compute(ObjectKind::Claim, format!("cycle_node_{}", i).as_bytes());
            let err = provenance.insert_derivation(Derivation {
                rule_id: "cycle_rule".to_string(),
                parents: vec![n1],
                child: n1,
                transformation_id: "self_loop".to_string(),
            });
            if err.is_ok() {
                circular_acceptances += 1;
            }
        }

        // 4. Check contradiction state
        if is_contradiction_case {
            let claim_a = ev.id();
            let claim_b = ORID::compute(ObjectKind::Claim, format!("opposing_{}", i).as_bytes());
            contradiction_engine
                .register_contradiction(claim_a, claim_b, "adversarial_conflict", 1000)
                .unwrap();
            let st = contradiction_engine.get_status(&claim_a);
            if st == Status::Verified {
                illegal_promotions += 1;
            }
        }
    }

    assert_eq!(illegal_promotions, 0, "Illegal VERIFIED promotions must be exactly 0");
    assert_eq!(circular_acceptances, 0, "Circular provenance acceptances must be exactly 0");

    // 5. Copied-source overcount check
    let mut copies_set = Vec::new();
    for i in 0..100 {
        let copy_ev = EvidenceRecord::new(
            Some(root_orid),
            format!("copy_source_{}", i),
            "ingest::mirror",
            1700000000 + i,
            "shared_echo_domain",
            "trust_med",
        )
        .unwrap();
        copies_set.push(copy_ev);
    }
    let independent_count = deduplicator.independent_support_count(&provenance, &copies_set);
    assert_eq!(independent_count, 1, "100 copies must yield independent_support_count == 1 (0 overcount)");

    // 6. Proof replay = 100% check
    let proof_a = provenance.why(&root_orid).unwrap();
    let proof_b = provenance.why(&root_orid).unwrap();
    assert_eq!(proof_a, proof_b, "Proof replay must be 100% identical");
}
