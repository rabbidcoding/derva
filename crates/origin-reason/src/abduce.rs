// INVARIANT: 100% observation satisfaction; exact minimality in small worlds; output status is ALWAYS Status::Hypothesis.
// KPI: 100% observation satisfaction; exact minimality; output status always Status::Hypothesis.

use crate::forward::ForwardReasoner;
use origin_core::Status;
use origin_logic::{Fact, HornRule};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Explanation {
    pub hypotheses: Vec<Fact>,
    pub cost: usize,
    pub status: Status,
    pub is_exact_minimal: bool,
}

impl Explanation {
    pub fn new(hypotheses: Vec<Fact>, is_exact_minimal: bool) -> Self {
        let cost = hypotheses.len();
        Self {
            hypotheses,
            cost,
            status: Status::Hypothesis, // Strictly enforced: ALWAYS Status::Hypothesis
            is_exact_minimal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbductionBudget {
    pub max_candidates: usize,
    pub max_cardinality: usize,
}

impl Default for AbductionBudget {
    fn default() -> Self {
        Self {
            max_candidates: 10_000,
            max_cardinality: 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbductionError {
    NoExplanationFound,
    BudgetExhausted,
}

#[derive(Debug, Clone, Default)]
pub struct AbductiveSearchEngine {
    pub rules: Vec<HornRule>,
    pub background_facts: HashSet<Fact>,
    pub candidate_pool: Vec<Fact>,
}

impl AbductiveSearchEngine {
    pub fn new(
        rules: Vec<HornRule>,
        background_facts: HashSet<Fact>,
        candidate_pool: Vec<Fact>,
    ) -> Self {
        Self {
            rules,
            background_facts,
            candidate_pool,
        }
    }

    /// Searches for the minimal set of hypotheses that explains all observations.
    /// INVARIANT: Returned explanation status is ALWAYS Status::Hypothesis (never Verified).
    pub fn search_min_explanation(
        &self,
        observations: &[Fact],
        budget: AbductionBudget,
    ) -> Result<Explanation, AbductionError> {
        let mut candidates_checked = 0;

        // Search by increasing subset cardinality (1, 2, ..., max_cardinality) for exact minimality
        for k in 1..=budget.max_cardinality.min(self.candidate_pool.len()) {
            let subsets = generate_subsets(&self.candidate_pool, k);

            for subset in subsets {
                candidates_checked += 1;
                if candidates_checked > budget.max_candidates {
                    return Err(AbductionError::BudgetExhausted);
                }

                // Check if subset explains all observations
                if self.explains_all(&subset, observations) {
                    let is_exact = candidates_checked <= budget.max_candidates;
                    let exp = Explanation::new(subset, is_exact);

                    // Safety invariant check: status MUST be Status::Hypothesis
                    assert_eq!(
                        exp.status,
                        Status::Hypothesis,
                        "CRITICAL: Abductive explanation MUST be Status::Hypothesis"
                    );

                    return Ok(exp);
                }
            }
        }

        Err(AbductionError::NoExplanationFound)
    }

    fn explains_all(&self, hypotheses: &[Fact], observations: &[Fact]) -> bool {
        let mut combined_facts: Vec<Fact> = self.background_facts.iter().cloned().collect();
        combined_facts.extend(hypotheses.iter().cloned());

        let mut reasoner = ForwardReasoner::new();
        reasoner.run_forward(&self.rules, &combined_facts);

        // All derived or known facts
        let derived_facts = reasoner.facts;

        // Verify 100% of observations are satisfied
        for obs in observations {
            let obs_id = compute_fact_orid(obs);
            if !derived_facts.contains_key(&obs_id) {
                return false;
            }
        }

        true
    }
}

fn generate_subsets(pool: &[Fact], k: usize) -> Vec<Vec<Fact>> {
    let mut results = Vec::new();
    let mut current = Vec::new();
    subsets_helper(pool, k, 0, &mut current, &mut results);
    results
}

fn subsets_helper(
    pool: &[Fact],
    k: usize,
    start: usize,
    current: &mut Vec<Fact>,
    results: &mut Vec<Vec<Fact>>,
) {
    if current.len() == k {
        results.push(current.clone());
        return;
    }

    for i in start..pool.len() {
        current.push(pool[i].clone());
        subsets_helper(pool, k, i + 1, current, results);
        current.pop();
    }
}

fn compute_fact_orid(fact: &Fact) -> origin_core::ORID {
    let mut buf = Vec::new();
    buf.extend_from_slice(fact.predicate.as_bytes());
    for arg in &fact.args {
        buf.extend_from_slice(arg.as_bytes());
    }
    origin_core::ORID::compute(origin_core::ObjectKind::Claim, &buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explanation_satisfies_observations_100_percent() {
        let rule = HornRule::parse("wet_grass() :- rain().").unwrap();
        let background = HashSet::new();
        let candidates = vec![
            Fact::new("rain", vec![] as Vec<&str>),
            Fact::new("sprinkler", vec![] as Vec<&str>),
        ];

        let engine = AbductiveSearchEngine::new(vec![rule], background, candidates);
        let obs = vec![Fact::new("wet_grass", vec![] as Vec<&str>)];

        let exp = engine
            .search_min_explanation(&obs, AbductionBudget::default())
            .unwrap();

        assert_eq!(exp.hypotheses.len(), 1);
        assert_eq!(exp.hypotheses[0], Fact::new("rain", vec![] as Vec<&str>));
    }

    #[test]
    fn test_exact_minimality_in_small_exhaustive_set() {
        let rule1 = HornRule::parse("alarm() :- burglary().").unwrap();
        let rule2 = HornRule::parse("alarm() :- earthquake().").unwrap();

        let engine = AbductiveSearchEngine::new(
            vec![rule1, rule2],
            HashSet::new(),
            vec![
                Fact::new("burglary", vec![] as Vec<&str>),
                Fact::new("earthquake", vec![] as Vec<&str>),
                Fact::new("wind", vec![] as Vec<&str>),
            ],
        );

        let obs = vec![Fact::new("alarm", vec![] as Vec<&str>)];
        let exp = engine
            .search_min_explanation(&obs, AbductionBudget::default())
            .unwrap();

        // Minimal explanation cardinality is 1
        assert_eq!(exp.cost, 1);
        assert!(exp.is_exact_minimal);
    }

    #[test]
    fn test_output_status_is_always_hypothesis_never_verified() {
        let rule = HornRule::parse("b() :- a().").unwrap();
        let engine = AbductiveSearchEngine::new(
            vec![rule],
            HashSet::new(),
            vec![Fact::new("a", vec![] as Vec<&str>)],
        );

        let obs = vec![Fact::new("b", vec![] as Vec<&str>)];
        let exp = engine
            .search_min_explanation(&obs, AbductionBudget::default())
            .unwrap();

        assert_eq!(exp.status, Status::Hypothesis);
        assert_ne!(exp.status, Status::Verified);
    }
}
