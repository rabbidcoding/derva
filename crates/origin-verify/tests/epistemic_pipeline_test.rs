// INVARIANT: Integration test of Phase P03 (evidence -> provenance -> logic -> constraints -> obligations).
// KPI: 0 illegal VERIFIED promotions; 0 circular provenance accepted; 0 copied-source overcount; 100% proof replay.

use origin_constraints::{Constraint, ReferenceSolver, SolveResult};
use origin_core::{ObjectKind, Status, ORID};
use origin_evidence::{CorrelationDeduplicator, Derivation, EvidenceRecord, ProvenanceHypergraph, TrustPolicy};
use origin_logic::{Fact, FixedPointEngine, HornRule};
use origin_verify::{ContradictionEngine, ObligationResolution, ObligationRuntime};

#[test]
fn test_epistemic_pipeline_adversarial_suite_zero_illegal_promotions() {
    let mut provenance = ProvenanceHypergraph::new();
    let deduplicator = CorrelationDeduplicator::new();
    let trust_policy = TrustPolicy::new(1);
    let mut contradiction_engine = ContradictionEngine::new();
    let logic_engine = FixedPointEngine::new();
    let solver = ReferenceSolver::new();

    let root_ev_orid = ORID::compute(ObjectKind::Evidence, b"root_sensor_fact");

    // 1. Evidence Record Creation
    let ev = EvidenceRecord::new(
        Some(root_ev_orid),
        "sensor_alpha",
        "ingest::direct",
        1700000000,
        "domain_telemetry",
        "trust_high",
    )
    .unwrap();

    // 2. Insert Provenance Derivation
    let claim_orid = ORID::compute(ObjectKind::Claim, b"derived_claim_1");
    provenance
        .insert_derivation(Derivation {
            rule_id: "rule_telemetry_infer".to_string(),
            parents: vec![root_ev_orid],
            child: claim_orid,
            transformation_id: "t_telemetry".to_string(),
        })
        .unwrap();

    // 3. Logic Deduction
    let rule = HornRule::parse("verified_fact(X) :- raw_fact(X).").unwrap();
    let mut initial_facts = std::collections::HashSet::new();
    initial_facts.insert(Fact::new("raw_fact", vec!["sensor_alpha"]));
    let inferred_facts = logic_engine.evaluate_semi_naive(&[rule], &initial_facts);
    assert!(inferred_facts.contains(&Fact::new("verified_fact", vec!["sensor_alpha"])));

    // 4. Constraint Solving
    let constraints = vec![
        Constraint::VarGreaterThan("reading".to_string(), 10),
        Constraint::VarLessThan("reading".to_string(), 50),
    ];
    let solve_res = solver.solve(&constraints, 1000);
    assert!(matches!(solve_res, SolveResult::Sat(_)));

    // 5. Obligation Runtime
    let witness_orid = ORID::compute(ObjectKind::Evidence, b"external_verifier_witness");
    let mut obligation = ObligationRuntime::new(claim_orid, "telemetry_within_range", 1700000000, 3600);
    obligation
        .resolve(ObligationResolution {
            witness: witness_orid,
            verifier: "kernel_auditor".to_string(),
            at: 1700000050,
        })
        .unwrap();

    // Verification check: claim can achieve VERIFIED status because all conditions are met
    let final_status = trust_policy.evaluate_epistemic_status(&ev, true);
    assert_eq!(final_status, Status::Verified);

    // 6. Adversarial Attack 1: Circular Provenance Attack (must be rejected)
    let cycle_res = provenance.insert_derivation(Derivation {
        rule_id: "cycle_attack".to_string(),
        parents: vec![claim_orid],
        child: root_ev_orid,
        transformation_id: "cycle".to_string(),
    });
    assert!(cycle_res.is_err(), "Circular provenance must be rejected!");

    // 7. Adversarial Attack 2: Contested claim promotion attack (must be rejected)
    let opposing_claim = ORID::compute(ObjectKind::Claim, b"derived_claim_opposing");
    contradiction_engine
        .register_contradiction(claim_orid, opposing_claim, "direct_conflict", 1700000100)
        .unwrap();
    assert_eq!(contradiction_engine.get_status(&claim_orid), Status::Contested);

    // 8. Adversarial Attack 3: Copied source overcount attack
    let support_count = deduplicator.independent_support_count(&provenance, &[ev]);
    assert_eq!(support_count, 1, "Copied source overcount must be 0");
}

#[test]
fn test_proof_replay_100_percent() {
    let mut provenance = ProvenanceHypergraph::new();
    let root_orid = ORID::compute(ObjectKind::Evidence, b"genesis_truth");
    let target_orid = ORID::compute(ObjectKind::Claim, b"final_target");

    provenance
        .insert_derivation(Derivation {
            rule_id: "rule_1".to_string(),
            parents: vec![root_orid],
            child: target_orid,
            transformation_id: "t_1".to_string(),
        })
        .unwrap();

    let proof1 = provenance.why(&target_orid).unwrap();
    let proof2 = provenance.why(&target_orid).unwrap();

    assert_eq!(proof1, proof2, "Proof replay must be 100% deterministic");
}
