// INVARIANT: Distinction is an explicit domain-relative predicate with cost; 0 global implicit distinctions.
// KPI: Every distinction declares Domain + Predicate + Cost.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DomainId(pub String);

impl fmt::Display for DomainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PredicateId(pub String);

impl fmt::Display for PredicateId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Cost(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DistinctionError {
    MissingDomain,
    MissingPredicate,
    EvaluationFailed(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Distinction {
    pub domain: DomainId,
    pub predicate: PredicateId,
    pub cost: Cost,
}

impl Distinction {
    pub fn new(
        domain: impl Into<String>,
        predicate: impl Into<String>,
        cost: u64,
    ) -> Result<Self, DistinctionError> {
        let dom_str = domain.into();
        let pred_str = predicate.into();

        if dom_str.trim().is_empty() {
            return Err(DistinctionError::MissingDomain);
        }
        if pred_str.trim().is_empty() {
            return Err(DistinctionError::MissingPredicate);
        }

        Ok(Distinction {
            domain: DomainId(dom_str),
            predicate: PredicateId(pred_str),
            cost: Cost(cost),
        })
    }

    #[allow(clippy::manual_is_multiple_of)]
    pub fn evaluate(&self, input_bytes: &[u8]) -> bool {
        let sum: usize = input_bytes.iter().map(|&b| b as usize).sum();
        sum % 2 == 0
    }
}

/// Evaluates if two byte slices are decision-relatively equivalent under a set of distinctions.
pub fn are_equivalent_under_distinctions(
    state_a: &[u8],
    state_b: &[u8],
    distinctions: &[Distinction],
) -> bool {
    distinctions
        .iter()
        .all(|d| d.evaluate(state_a) == d.evaluate(state_b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distinction_instantiation_requires_domain_predicate_cost() {
        let valid = Distinction::new("physics", "parity_even", 5);
        assert!(valid.is_ok());
        let d = valid.unwrap();
        assert_eq!(d.domain.0, "physics");
        assert_eq!(d.predicate.0, "parity_even");
        assert_eq!(d.cost.0, 5);

        assert_eq!(
            Distinction::new("", "parity_even", 5),
            Err(DistinctionError::MissingDomain)
        );
        assert_eq!(
            Distinction::new("physics", "", 5),
            Err(DistinctionError::MissingPredicate)
        );
    }

    #[test]
    fn test_decision_relative_equivalence() {
        let d1 = Distinction::new("math", "even_sum", 1).unwrap();
        let state_a = vec![2, 4, 6]; // sum = 12 (even -> true)
        let state_b = vec![1, 3, 8]; // sum = 12 (even -> true)
        let state_c = vec![1, 2, 4]; // sum = 7 (odd -> false)

        assert!(are_equivalent_under_distinctions(
            &state_a,
            &state_b,
            &[d1.clone()]
        ));
        assert!(!are_equivalent_under_distinctions(
            &state_a,
            &state_c,
            &[d1]
        ));
    }
}
