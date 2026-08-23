// INVARIANT: Never materializes Omega; state equivalence is calculated locally w.r.t relevant set R.
// KPI: Active quotient reduces >= 10x states in >= 70% of redundant scenarios.

use crate::distinction::Distinction;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorldSig {
    pub bytes: Vec<u8>,
}

impl WorldSig {
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            bytes: bytes.into(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RelevantSet {
    pub distinctions: Vec<Distinction>,
}

impl RelevantSet {
    pub fn new(distinctions: Vec<Distinction>) -> Self {
        Self { distinctions }
    }

    pub fn are_equivalent(&self, state_a: &WorldSig, state_b: &WorldSig) -> bool {
        self.distinctions
            .iter()
            .all(|d| d.evaluate(&state_a.bytes) == d.evaluate(&state_b.bytes))
    }
}

pub fn equivalent(a: &WorldSig, b: &WorldSig, r: &RelevantSet) -> bool {
    r.are_equivalent(a, b)
}

/// Partition a sequence of observed states into active quotient equivalence classes [S/~_R].
/// Never materializes the unobserved global state space Omega.
pub fn partition_active_quotient(states: &[WorldSig], r: &RelevantSet) -> Vec<Vec<WorldSig>> {
    let mut classes: Vec<Vec<WorldSig>> = Vec::new();
    for s in states {
        let mut found = false;
        for cls in &mut classes {
            if equivalent(&cls[0], s, r) {
                cls.push(s.clone());
                found = true;
                break;
            }
        }
        if !found {
            classes.push(vec![s.clone()]);
        }
    }
    classes
}

pub fn compute_reduction_ratio(raw_count: usize, quotient_count: usize) -> f64 {
    if quotient_count == 0 {
        1.0
    } else {
        raw_count as f64 / quotient_count as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quotient_preserves_decisions_and_reduces_state_space() {
        let d1 = Distinction::new("math", "parity_even", 1).unwrap();
        let r = RelevantSet::new(vec![d1]);

        // Generate 1000 synthetic states with redundant parity signatures
        let mut raw_states = Vec::new();
        for i in 0..1000 {
            raw_states.push(WorldSig::new(vec![(i % 256) as u8]));
        }

        let classes = partition_active_quotient(&raw_states, &r);

        // Under a single binary parity distinction, 1000 states must reduce to exactly 2 equivalence classes
        assert_eq!(classes.len(), 2);

        let reduction_ratio = compute_reduction_ratio(raw_states.len(), classes.len());
        assert!(
            reduction_ratio >= 10.0,
            "Reduction ratio was {}",
            reduction_ratio
        );

        // Verify decision preservation: all members in a class have identical distinction evaluation
        for cls in &classes {
            let repr = &cls[0];
            for member in cls {
                assert!(equivalent(repr, member, &r));
            }
        }
    }
}
