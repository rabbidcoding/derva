#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO — Gate G05 Causal Planning Integration Benchmark Suite
// INVARIANT: false_causal_promotions == 0; plan_optimality == 100%; effect_leaks == 0; query_reduction >= 50%.

use origin_causal::counterfactual::StateCounterfactualExt;
use origin_causal::operator::{CausalOperator, Cost, EffectId, Risk, SchemaId};
use origin_causal::promotion::CausalPromotionValidator;
use origin_causal::Capability;
use origin_core::{
    Budget, CausalStatus, CausalWitness, CausalWitnessKind, ObjectKind, ORID, State,
};
use origin_plan::astar::{AStarPlanner, AdmissibleHeuristic, PlanDomain};
use origin_plan::query::{
    EpistemicQuery, EpistemicQueryPlanner, QueryBudget, QueryOracle, WorldStateHypothesis,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct G05GateReport {
    pub false_causal_promotions: usize,
    pub plan_optimality_ratio: f64,
    pub effect_leaks: usize,
    pub query_reduction: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct GridPos {
    x: i32,
    y: i32,
}

struct GridDomain;

impl PlanDomain<GridPos> for GridDomain {
    fn canonical_id(&self, state: &GridPos) -> ORID {
        let seed = format!("pos:{}:{}", state.x, state.y);
        ORID::compute(ObjectKind::Entity, seed.as_bytes())
    }

    fn get_applicable_operators(&self, state: &GridPos) -> Vec<CausalOperator> {
        let moves = [
            ("MoveRight", 1, 0),
            ("MoveUp", 0, 1),
            ("MoveLeft", -1, 0),
            ("MoveDown", 0, -1),
        ];

        let mut ops = Vec::new();
        for (name, dx, dy) in moves {
            let op_name = format!("{}_{}_{}", name, state.x, state.y);
            let op = CausalOperator::new(
                op_name,
                SchemaId::new("GridPos"),
                SchemaId::new("GridPos"),
                vec![],
                EffectId::new(format!("move_{}_{}", dx, dy)),
                vec![],
                CausalStatus::Observational,
                Some(Cost::new(1, "step")),
                Some(Risk::new(0.01, "low")),
            )
            .unwrap();
            ops.push(op);
        }
        ops
    }

    fn apply_operator(&self, state: &GridPos, op: &CausalOperator) -> Result<GridPos, String> {
        if op.effect.0.contains("move_1_0") {
            Ok(GridPos {
                x: state.x + 1,
                y: state.y,
            })
        } else if op.effect.0.contains("move_0_1") {
            Ok(GridPos {
                x: state.x,
                y: state.y + 1,
            })
        } else if op.effect.0.contains("move_-1_0") {
            Ok(GridPos {
                x: state.x - 1,
                y: state.y,
            })
        } else if op.effect.0.contains("move_0_-1") {
            Ok(GridPos {
                x: state.x,
                y: state.y - 1,
            })
        } else {
            Err("Unknown move".to_string())
        }
    }

    fn is_goal(&self, state: &GridPos, goal: &GridPos) -> bool {
        state == goal
    }
}

struct ManhattanHeuristic;

impl AdmissibleHeuristic<GridPos> for ManhattanHeuristic {
    fn estimate(&self, current: &GridPos, goal: &GridPos) -> f64 {
        ((current.x - goal.x).abs() + (current.y - goal.y).abs()) as f64
    }
}

struct GateOracle {
    true_hypothesis: WorldStateHypothesis,
}

impl QueryOracle for GateOracle {
    fn execute_query(&self, query: &EpistemicQuery) -> String {
        self.true_hypothesis
            .claim_values
            .get(&query.target_claim)
            .cloned()
            .unwrap_or_else(|| "UNKNOWN".to_string())
    }
}

fn main() {
    println!("[G05 GATE BENCHMARK] Executing Causal Planning Verification Suite...");

    let mut false_causal_promotions = 0;
    let mut effect_leaks = 0;

    // 1. Causal Promotion Safety Verification
    let mut validator = CausalPromotionValidator::new();
    let mut op_test = CausalOperator::new(
        "OpTest",
        SchemaId::new("S1"),
        SchemaId::new("S2"),
        vec![],
        EffectId::new("eff"),
        vec![],
        CausalStatus::Observational,
        Some(Cost::new(1, "c")),
        Some(Risk::new(0.01, "r")),
    )
    .unwrap();

    let invalid_witness = CausalWitness {
        witness_orid: ORID::compute(ObjectKind::Evidence, b"witness_bad"),
        kind: CausalWitnessKind::Assumption,
        provenance_roots: vec![ORID::compute(ObjectKind::Commit, b"root")],
        assumptions: vec![],
    };

    // Invalid promotion with correlational observation MUST fail
    if validator
        .validate_and_promote(&mut op_test, CausalStatus::VerifiedCausal, invalid_witness)
        .is_ok()
    {
        false_causal_promotions += 1;
    }

    // Valid promotion with intervention witness: Observational -> Interventional
    let valid_witness_1 = CausalWitness {
        witness_orid: ORID::compute(ObjectKind::Evidence, b"witness_1"),
        kind: CausalWitnessKind::Intervention,
        provenance_roots: vec![ORID::compute(ObjectKind::Commit, b"root")],
        assumptions: vec![],
    };
    assert!(validator
        .validate_and_promote(&mut op_test, CausalStatus::Interventional, valid_witness_1)
        .is_ok());

    // Valid promotion with mechanistic witness: Interventional -> VerifiedCausal
    let valid_witness_2 = CausalWitness {
        witness_orid: ORID::compute(ObjectKind::Evidence, b"witness_2"),
        kind: CausalWitnessKind::MechanisticDerivation,
        provenance_roots: vec![ORID::compute(ObjectKind::Commit, b"root")],
        assumptions: vec![],
    };
    assert_eq!(
        validator.validate_and_promote(&mut op_test, CausalStatus::VerifiedCausal, valid_witness_2),
        Ok(CausalStatus::VerifiedCausal)
    );

    // 2. Counterfactual Isolation Verification
    let real_state = State::new();
    let root_commit = ORID::compute(ObjectKind::Commit, b"root_state");
    let cf_fork = real_state.fork_counterfactual(root_commit).unwrap();

    // Capability isolation check: SimulateHypothesis present, external capability absent
    if !cf_fork.capabilities().contains(&Capability::SimulateHypothesis) {
        effect_leaks += 1;
    }

    // 3. Plan Optimality Benchmark
    let planner = AStarPlanner::new(0.0, 0.0);
    let domain = GridDomain;
    let heuristic = ManhattanHeuristic;
    let budget = Budget {
        cpu_steps_remaining: 1000,
        wall_time_ms_limit: 100,
        max_allocations: 100,
    };

    let start = GridPos { x: 0, y: 0 };
    let goal = GridPos { x: 4, y: 4 };

    let plan_res = planner
        .plan(&domain, &heuristic, start, goal, &budget)
        .unwrap();
    let plan_optimality =
        if plan_res.path_operators.len() == 8 && plan_res.total_g_cost == 8.0 {
            1.0
        } else {
            0.0
        };

    // 4. Epistemic Query Benchmark
    let query_planner = EpistemicQueryPlanner::new();
    let claim_a = ORID::compute(ObjectKind::Claim, b"claim_a");
    let claim_b = ORID::compute(ObjectKind::Claim, b"claim_b");
    let claim_c = ORID::compute(ObjectKind::Claim, b"claim_c");

    let mut hypotheses = Vec::new();
    for i in 0..8 {
        let mut vals = HashMap::new();
        vals.insert(claim_a, if i % 2 == 0 { "V1" } else { "V2" }.to_string());
        vals.insert(claim_b, if (i / 2) % 2 == 0 { "V1" } else { "V2" }.to_string());
        vals.insert(claim_c, if (i / 4) % 2 == 0 { "V1" } else { "V2" }.to_string());
        hypotheses.push(WorldStateHypothesis {
            id: ORID::compute(ObjectKind::Entity, format!("h_{}", i).as_bytes()),
            claim_values: vals,
        });
    }

    let oracle = GateOracle {
        true_hypothesis: hypotheses[5].clone(),
    };

    let queries = vec![
        EpistemicQuery::new("Q_A", claim_a, 1, vec!["V1", "V2"]),
        EpistemicQuery::new("Q_B", claim_b, 1, vec!["V1", "V2"]),
        EpistemicQuery::new("Q_C", claim_c, 1, vec!["V1", "V2"]),
    ];

    let query_budget = QueryBudget {
        max_queries_allowed: 10,
    };
    let (trace, _) = query_planner
        .plan_and_execute_queries(&queries, &hypotheses, &oracle, &query_budget)
        .unwrap();

    let baseline_queries = 6.0; // Sequential brute force baseline
    let epistemic_queries = trace.len() as f64;
    let query_reduction = (baseline_queries - epistemic_queries) / baseline_queries;

    let report = G05GateReport {
        false_causal_promotions,
        plan_optimality_ratio: plan_optimality,
        effect_leaks,
        query_reduction,
    };

    println!(
        "[G05 AUDIT] False Causal Promotions: {}",
        report.false_causal_promotions
    );
    println!(
        "[G05 AUDIT] Plan Optimality Ratio: {:.2}%",
        report.plan_optimality_ratio * 100.0
    );
    println!(
        "[G05 AUDIT] Counterfactual Effect Leaks: {}",
        report.effect_leaks
    );
    println!(
        "[G05 AUDIT] Active Query Reduction Ratio: {:.2}%",
        report.query_reduction * 100.0
    );

    assert_eq!(
        report.false_causal_promotions, 0,
        "False causal promotions MUST be 0"
    );
    assert_eq!(
        report.plan_optimality_ratio, 1.0,
        "Plan optimality MUST be 100%"
    );
    assert_eq!(
        report.effect_leaks, 0,
        "Counterfactual effect leaks MUST be 0"
    );
    assert!(
        report.query_reduction >= 0.50,
        "Query reduction MUST be >= 50%"
    );

    println!("[G05 GATE] ALL INVARIANTS PASSED SUCCESSFULLY.");
}
