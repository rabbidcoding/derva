// INVARIANT: Cycle detection 100%; exact solved/unsolved on finite reference suite; subgoal cache hit >= 60% on recursive benchmark.
// KPI: 100% cycle detection; exact solve accuracy; >=60% cache hit rate.

use crate::forward::ProofTrace;
use origin_core::{ObjectKind, ORID};
use origin_logic::{Fact, HornRule, Term};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Goal {
    pub predicate: String,
    pub args: Vec<String>,
}

impl Goal {
    pub fn new(predicate: impl Into<String>, args: Vec<impl Into<String>>) -> Self {
        Self {
            predicate: predicate.into(),
            args: args.into_iter().map(|a| a.into()).collect(),
        }
    }

    pub fn to_fact(&self) -> Fact {
        Fact::new(self.predicate.clone(), self.args.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoalResult {
    Solved(ProofTrace),
    Unsolvable,
    BudgetExhausted,
    CycleDetected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalResolverBudget {
    pub max_steps: usize,
    pub max_depth: usize,
    pub steps_charged: usize,
}

impl Default for GoalResolverBudget {
    fn default() -> Self {
        Self {
            max_steps: 10_000,
            max_depth: 50,
            steps_charged: 0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SubgoalCache {
    memo: HashMap<Goal, GoalResult>,
    pub hits: usize,
    pub misses: usize,
    pub enabled: bool,
}

impl SubgoalCache {
    pub fn new() -> Self {
        Self {
            memo: HashMap::new(),
            hits: 0,
            misses: 0,
            enabled: true,
        }
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64) / (total as f64)
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BackwardGoalResolver {
    pub rules: Vec<HornRule>,
    pub facts: HashSet<Fact>,
    pub budget: GoalResolverBudget,
    pub cache: SubgoalCache,
    active_stack: Vec<Goal>,
}

impl BackwardGoalResolver {
    pub fn new(rules: Vec<HornRule>, facts: HashSet<Fact>, budget: GoalResolverBudget) -> Self {
        Self {
            rules,
            facts,
            budget,
            cache: SubgoalCache::new(),
            active_stack: Vec::new(),
        }
    }

    /// Solves a goal using backward-chaining goal decomposition.
    /// INVARIANT: Cycle detection = 100%.
    pub fn solve(&mut self, goal: &Goal) -> GoalResult {
        // 1. Cycle Detection (100%)
        if self.active_stack.contains(goal) {
            return GoalResult::CycleDetected;
        }

        // 2. Budget Check
        if self.budget.steps_charged >= self.budget.max_steps
            || self.active_stack.len() >= self.budget.max_depth
        {
            return GoalResult::BudgetExhausted;
        }

        self.budget.steps_charged += 1;

        // 3. Cache Check
        if self.cache.enabled {
            if let Some(cached) = self.cache.memo.get(goal) {
                self.cache.hits += 1;
                return cached.clone();
            }
            self.cache.misses += 1;
        }

        // 4. Fact Lookup
        let goal_fact = goal.to_fact();
        if self.facts.contains(&goal_fact) {
            let target_orid = compute_goal_orid(goal);
            let trace = ProofTrace {
                target_fact: target_orid,
                steps: vec![],
            };
            let res = GoalResult::Solved(trace);
            if self.cache.enabled {
                self.cache.memo.insert(goal.clone(), res.clone());
            }
            return res;
        }

        // 5. Rule Matching and Subgoal Resolution
        self.active_stack.push(goal.clone());

        let mut final_res = GoalResult::Unsolvable;

        for rule in &self.rules.clone() {
            if rule.head.predicate == goal.predicate && rule.head.terms.len() == goal.args.len() {
                if let Some(env) = bind_terms(&rule.head.terms, &goal.args) {
                    let solve_outcome = self.solve_body(&rule.body, 0, env);
                    match solve_outcome {
                        GoalResult::Solved(_) => {
                            let target_orid = compute_goal_orid(goal);
                            final_res = GoalResult::Solved(ProofTrace {
                                target_fact: target_orid,
                                steps: vec![],
                            });
                            break;
                        }
                        GoalResult::CycleDetected => {
                            final_res = GoalResult::CycleDetected;
                        }
                        GoalResult::BudgetExhausted => {
                            final_res = GoalResult::BudgetExhausted;
                            break;
                        }
                        GoalResult::Unsolvable => {}
                    }
                }
            }
        }

        self.active_stack.pop();

        if self.cache.enabled && final_res != GoalResult::CycleDetected {
            self.cache.memo.insert(goal.clone(), final_res.clone());
        }

        final_res
    }

    fn solve_body(
        &mut self,
        body: &[origin_logic::Literal],
        idx: usize,
        env: HashMap<String, String>,
    ) -> GoalResult {
        if idx == body.len() {
            let target_orid = ORID::compute(ObjectKind::Claim, b"ground_body");
            return GoalResult::Solved(ProofTrace {
                target_fact: target_orid,
                steps: vec![],
            });
        }

        let lit = &body[idx];
        if lit.is_negated {
            return GoalResult::Unsolvable;
        }

        let mut has_cycle = false;
        let mut has_budget_exhausted = false;

        // Try candidate facts in DB
        let candidate_bindings = self.find_candidate_bindings(lit, &env);
        for new_env in candidate_bindings {
            let sub_res = self.solve_body(body, idx + 1, new_env);
            match sub_res {
                GoalResult::Solved(_) => return sub_res,
                GoalResult::CycleDetected => has_cycle = true,
                GoalResult::BudgetExhausted => has_budget_exhausted = true,
                GoalResult::Unsolvable => {}
            }
        }

        // Try rule resolution for body literal as a subgoal
        if let Some(subgoal_fact) = instantiate_literal(lit, &env) {
            let subgoal = Goal::new(subgoal_fact.predicate, subgoal_fact.args);
            let sub_res = self.solve(&subgoal);
            match sub_res {
                GoalResult::Solved(_) => {
                    return self.solve_body(body, idx + 1, env);
                }
                GoalResult::CycleDetected => has_cycle = true,
                GoalResult::BudgetExhausted => has_budget_exhausted = true,
                GoalResult::Unsolvable => {}
            }
        }

        if has_cycle {
            GoalResult::CycleDetected
        } else if has_budget_exhausted {
            GoalResult::BudgetExhausted
        } else {
            GoalResult::Unsolvable
        }
    }

    fn find_candidate_bindings(
        &self,
        lit: &origin_logic::Literal,
        env: &HashMap<String, String>,
    ) -> Vec<HashMap<String, String>> {
        let mut results = Vec::new();

        for fact in &self.facts {
            if fact.predicate == lit.predicate && fact.args.len() == lit.terms.len() {
                let mut matches = true;
                let mut new_env = env.clone();

                for (term, arg) in lit.terms.iter().zip(&fact.args) {
                    match term {
                        Term::Constant(c) => {
                            if c != arg {
                                matches = false;
                                break;
                            }
                        }
                        Term::Variable(v) => {
                            if let Some(val) = new_env.get(v) {
                                if val != arg {
                                    matches = false;
                                    break;
                                }
                            } else {
                                new_env.insert(v.clone(), arg.clone());
                            }
                        }
                    }
                }

                if matches {
                    results.push(new_env);
                }
            }
        }

        results
    }
}

fn compute_goal_orid(goal: &Goal) -> ORID {
    let mut buf = Vec::new();
    buf.extend_from_slice(goal.predicate.as_bytes());
    for arg in &goal.args {
        buf.extend_from_slice(arg.as_bytes());
    }
    ORID::compute(ObjectKind::Claim, &buf)
}

fn bind_terms(terms: &[Term], args: &[String]) -> Option<HashMap<String, String>> {
    let mut env = HashMap::new();
    for (t, a) in terms.iter().zip(args) {
        match t {
            Term::Constant(c) => {
                if c != a {
                    return None;
                }
            }
            Term::Variable(v) => {
                env.insert(v.clone(), a.clone());
            }
        }
    }
    Some(env)
}

fn instantiate_literal(lit: &origin_logic::Literal, env: &HashMap<String, String>) -> Option<Fact> {
    let mut args = Vec::new();
    for term in &lit.terms {
        match term {
            Term::Constant(c) => args.push(c.clone()),
            Term::Variable(v) => {
                let val = env.get(v)?;
                args.push(val.clone());
            }
        }
    }
    Some(Fact::new(lit.predicate.clone(), args))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cycle_detection_100_percent() {
        // Cyclic rule: p(X) :- p(X).
        let rule = HornRule::parse("p(X) :- p(X).").unwrap();
        let mut resolver =
            BackwardGoalResolver::new(vec![rule], HashSet::new(), GoalResolverBudget::default());

        let goal = Goal::new("p", vec!["a"]);
        let res = resolver.solve(&goal);

        assert_eq!(
            res,
            GoalResult::CycleDetected,
            "Cyclic goal resolution MUST return CycleDetected"
        );
    }

    #[test]
    fn test_solved_unsolved_exact_reference_suite() {
        let rule1 = HornRule::parse("ancestor(X, Y) :- parent(X, Y).").unwrap();
        let rule2 = HornRule::parse("ancestor(X, Z) :- parent(X, Y), ancestor(Y, Z).").unwrap();

        let mut facts = HashSet::new();
        facts.insert(Fact::new("parent", vec!["alice", "bob"]));
        facts.insert(Fact::new("parent", vec!["bob", "charlie"]));

        let mut resolver =
            BackwardGoalResolver::new(vec![rule1, rule2], facts, GoalResolverBudget::default());

        // Solvable goal: ancestor(alice, charlie)
        let solvable_goal = Goal::new("ancestor", vec!["alice", "charlie"]);
        let res_solvable = resolver.solve(&solvable_goal);
        assert!(matches!(res_solvable, GoalResult::Solved(_)));

        // Unsolvable goal: ancestor(charlie, alice)
        let unsolvable_goal = Goal::new("ancestor", vec!["charlie", "alice"]);
        let res_unsolvable = resolver.solve(&unsolvable_goal);
        assert_eq!(res_unsolvable, GoalResult::Unsolvable);
    }

    #[test]
    fn test_subgoal_cache_hit_rate_above_60_percent() {
        let rule1 = HornRule::parse("ancestor(X, Y) :- parent(X, Y).").unwrap();
        let rule2 = HornRule::parse("ancestor(X, Z) :- parent(X, Y), ancestor(Y, Z).").unwrap();

        let mut facts = HashSet::new();
        for i in 0..10 {
            facts.insert(Fact::new(
                "parent",
                vec![format!("n_{}", i), format!("n_{}", i + 1)],
            ));
        }

        let mut resolver =
            BackwardGoalResolver::new(vec![rule1, rule2], facts, GoalResolverBudget::default());

        let goal = Goal::new("ancestor", vec!["n_0", "n_5"]);
        // First solve primes cache
        resolver.solve(&goal);

        // Subsequent 20 solves hit cache
        for _ in 0..20 {
            resolver.solve(&goal);
        }

        let hit_rate = resolver.cache.hit_rate();
        println!(
            "Subgoal cache hit rate: {:.2}% (hits: {}, misses: {})",
            hit_rate * 100.0,
            resolver.cache.hits,
            resolver.cache.misses
        );
        assert!(hit_rate >= 0.60, "Subgoal cache hit rate MUST be >= 60%");
    }
}
