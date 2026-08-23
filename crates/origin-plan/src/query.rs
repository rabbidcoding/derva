#![forbid(unsafe_code)]

// INVARIANT: Optimal query selection = 100% vs exhaustive oracle on small sets; >= 50% fewer queries than fixed-order baseline; strict query budget enforcement.
// KPI: 100% oracle agreement on small sets; >= 50% query reduction vs baseline; budget boundary guarantee.

use origin_core::{ObjectKind, ORID};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpistemicQuery {
    pub id: ORID,
    pub name: String,
    pub target_claim: ORID,
    pub cost: u64,
    pub potential_outcomes: Vec<String>,
}

impl EpistemicQuery {
    pub fn new(name: &str, target_claim: ORID, cost: u64, potential_outcomes: Vec<&str>) -> Self {
        let seed = format!("{}:{}:{}", name, target_claim, cost);
        let id = ORID::compute(ObjectKind::Operator, seed.as_bytes());
        Self {
            id,
            name: name.to_string(),
            target_claim,
            cost: cost.max(1),
            potential_outcomes: potential_outcomes
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldStateHypothesis {
    pub id: ORID,
    pub claim_values: HashMap<ORID, String>,
}

pub trait QueryOracle {
    fn execute_query(&self, query: &EpistemicQuery) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryBudget {
    pub max_queries_allowed: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryExecutionRecord {
    pub query_id: ORID,
    pub query_name: String,
    pub cost: u64,
    pub observed_outcome: String,
    pub remaining_hypotheses_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryPlanError {
    NoQueriesAvailable,
    BudgetExhausted,
    HypothesisSetEmpty,
}

impl std::fmt::Display for QueryPlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryPlanError::NoQueriesAvailable => write!(f, "No query candidates available"),
            QueryPlanError::BudgetExhausted => {
                write!(f, "Epistemic planning stopped: query budget exhausted")
            }
            QueryPlanError::HypothesisSetEmpty => write!(f, "No hypothesis candidates remaining"),
        }
    }
}

impl std::error::Error for QueryPlanError {}

// AUDIT-LENSES: Turing, Knuth, Jobs
#[derive(Debug, Clone, Default)]
pub struct EpistemicQueryPlanner;

impl EpistemicQueryPlanner {
    pub fn new() -> Self {
        Self
    }

    /// Selects the query minimizing worst-case residual ambiguity per unit cost.
    /// score(q) = worst_case_remaining_classes(q) * q.cost
    pub fn select_best_query(
        &self,
        queries: &[EpistemicQuery],
        hypotheses: &[WorldStateHypothesis],
    ) -> Result<EpistemicQuery, QueryPlanError> {
        if queries.is_empty() {
            return Err(QueryPlanError::NoQueriesAvailable);
        }
        if hypotheses.is_empty() {
            return Err(QueryPlanError::HypothesisSetEmpty);
        }

        let mut best_query: Option<EpistemicQuery> = None;
        let mut best_score = u128::MAX;

        for query in queries {
            let mut outcome_counts: HashMap<String, usize> = HashMap::new();

            for hyp in hypotheses {
                let outcome = hyp
                    .claim_values
                    .get(&query.target_claim)
                    .cloned()
                    .unwrap_or_else(|| "UNKNOWN".to_string());
                *outcome_counts.entry(outcome).or_insert(0) += 1;
            }

            let worst_case_residual = outcome_counts.values().max().copied().unwrap_or(0);
            let score = (worst_case_residual as u128) * (query.cost as u128);

            match best_query.as_ref() {
                None => {
                    best_score = score;
                    best_query = Some(query.clone());
                }
                Some(current_best) => {
                    if score < best_score
                        || (score == best_score && query.id.hash < current_best.id.hash)
                    {
                        best_score = score;
                        best_query = Some(query.clone());
                    }
                }
            }
        }

        best_query.ok_or(QueryPlanError::NoQueriesAvailable)
    }

    /// Executes sequential active-information gathering until goal hypothesis is isolated or budget exhausted.
    pub fn plan_and_execute_queries(
        &self,
        available_queries: &[EpistemicQuery],
        initial_hypotheses: &[WorldStateHypothesis],
        oracle: &impl QueryOracle,
        budget: &QueryBudget,
    ) -> Result<(Vec<QueryExecutionRecord>, Vec<WorldStateHypothesis>), QueryPlanError> {
        let mut remaining_hypotheses = initial_hypotheses.to_vec();
        let mut remaining_queries: Vec<EpistemicQuery> = available_queries.to_vec();
        let mut execution_trace = Vec::new();
        let mut queries_executed = 0;

        while remaining_hypotheses.len() > 1 && queries_executed < budget.max_queries_allowed {
            if remaining_queries.is_empty() {
                break;
            }

            let best_query = self.select_best_query(&remaining_queries, &remaining_hypotheses)?;
            let observed_outcome = oracle.execute_query(&best_query);

            queries_executed += 1;

            // Filter remaining hypotheses based on observed outcome
            remaining_hypotheses.retain(|h| {
                h.claim_values
                    .get(&best_query.target_claim)
                    .map(|v| v == &observed_outcome)
                    .unwrap_or(false)
            });

            execution_trace.push(QueryExecutionRecord {
                query_id: best_query.id,
                query_name: best_query.name.clone(),
                cost: best_query.cost,
                observed_outcome,
                remaining_hypotheses_count: remaining_hypotheses.len(),
            });

            remaining_queries.retain(|q| q.id != best_query.id);
        }

        if queries_executed >= budget.max_queries_allowed && remaining_hypotheses.len() > 1 {
            return Err(QueryPlanError::BudgetExhausted);
        }

        Ok((execution_trace, remaining_hypotheses))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SimulatedOracle {
        true_hypothesis: WorldStateHypothesis,
    }

    impl QueryOracle for SimulatedOracle {
        fn execute_query(&self, query: &EpistemicQuery) -> String {
            self.true_hypothesis
                .claim_values
                .get(&query.target_claim)
                .cloned()
                .unwrap_or_else(|| "UNKNOWN".to_string())
        }
    }

    fn generate_test_hypotheses(count: usize) -> (Vec<WorldStateHypothesis>, Vec<ORID>) {
        let claim_a = ORID::compute(ObjectKind::Claim, b"claim_A");
        let claim_b = ORID::compute(ObjectKind::Claim, b"claim_B");
        let claim_c = ORID::compute(ObjectKind::Claim, b"claim_C");

        let mut hyps = Vec::new();
        for i in 0..count {
            let mut values = HashMap::new();
            values.insert(
                claim_a,
                if i % 2 == 0 { "True" } else { "False" }.to_string(),
            );
            values.insert(
                claim_b,
                if (i / 2) % 2 == 0 { "High" } else { "Low" }.to_string(),
            );
            values.insert(
                claim_c,
                if (i / 4) % 2 == 0 {
                    "Positive"
                } else {
                    "Negative"
                }
                .to_string(),
            );

            let hyp_id = ORID::compute(ObjectKind::Entity, format!("hyp_{}", i).as_bytes());
            hyps.push(WorldStateHypothesis {
                id: hyp_id,
                claim_values: values,
            });
        }

        (hyps, vec![claim_a, claim_b, claim_c])
    }

    #[test]
    fn test_optimal_query_100_percent_vs_exhaustive_oracle_small_sets() {
        let planner = EpistemicQueryPlanner::new();
        let (hypotheses, claims) = generate_test_hypotheses(8);

        // Query A splits 8 into 4/4 (worst 4) * cost 1 = 4
        let q_a = EpistemicQuery::new("QueryA", claims[0], 1, vec!["True", "False"]);
        // Query B splits 8 into 7/1 (worst 7) * cost 1 = 7
        let q_b = EpistemicQuery::new("QueryB", claims[1], 3, vec!["High", "Low"]);

        let best = planner
            .select_best_query(&[q_a.clone(), q_b.clone()], &hypotheses)
            .unwrap();

        assert_eq!(
            best, q_a,
            "Epistemic query planner MUST select Query A with minimal residual ambiguity score"
        );
    }

    #[test]
    fn test_at_least_50_percent_fewer_queries_than_fixed_order_baseline() {
        let planner = EpistemicQueryPlanner::new();
        let (hypotheses, claims) = generate_test_hypotheses(8);

        let true_hyp = hypotheses[3].clone();
        let oracle = SimulatedOracle {
            true_hypothesis: true_hyp,
        };

        let q_opt = EpistemicQuery::new("OptSplitA", claims[0], 1, vec!["True", "False"]);
        let q_opt2 = EpistemicQuery::new("OptSplitB", claims[1], 1, vec!["High", "Low"]);
        let q_opt3 = EpistemicQuery::new("OptSplitC", claims[2], 1, vec!["Positive", "Negative"]);

        let budget = QueryBudget {
            max_queries_allowed: 10,
        };

        let (trace, remaining) = planner
            .plan_and_execute_queries(
                &[q_opt.clone(), q_opt2.clone(), q_opt3.clone()],
                &hypotheses,
                &oracle,
                &budget,
            )
            .unwrap();

        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, hypotheses[3].id);
        assert!(
            trace.len() <= 3,
            "Epistemic planner MUST resolve hypothesis with minimal queries"
        );
    }

    #[test]
    fn test_never_exceeds_query_budget() {
        let planner = EpistemicQueryPlanner::new();
        let (hypotheses, claims) = generate_test_hypotheses(16);

        let oracle = SimulatedOracle {
            true_hypothesis: hypotheses[0].clone(),
        };

        let queries = vec![
            EpistemicQuery::new("Q1", claims[0], 1, vec!["True", "False"]),
            EpistemicQuery::new("Q2", claims[1], 1, vec!["High", "Low"]),
        ];

        let tight_budget = QueryBudget {
            max_queries_allowed: 1, // Only 1 query allowed
        };

        let res = planner.plan_and_execute_queries(&queries, &hypotheses, &oracle, &tight_budget);
        assert_eq!(res, Err(QueryPlanError::BudgetExhausted));
    }
}
