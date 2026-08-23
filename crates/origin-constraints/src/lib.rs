#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO Subsystem: origin-constraints
// Constraint Solver ABI with proof/witness hooks and reference solver backend.

pub mod reference;

use origin_core::ORID;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constraint {
    VarEquals(String, i64),
    VarLessThan(String, i64),
    VarGreaterThan(String, i64),
    NotEquals(String, i64),
    LinearGe(Vec<(i64, String)>, i64),
}

impl Constraint {
    pub fn collect_variables(&self, set: &mut HashSet<String>) {
        match self {
            Constraint::VarEquals(v, _)
            | Constraint::VarLessThan(v, _)
            | Constraint::VarGreaterThan(v, _)
            | Constraint::NotEquals(v, _) => {
                set.insert(v.clone());
            }
            Constraint::LinearGe(terms, _) => {
                for (_, v) in terms {
                    set.insert(v.clone());
                }
            }
        }
    }

    pub fn evaluate(&self, env: &HashMap<String, i64>) -> bool {
        match self {
            Constraint::VarEquals(v, target) => {
                if let Some(val) = env.get(v) {
                    val == target
                } else {
                    true
                }
            }
            Constraint::VarLessThan(v, bound) => {
                if let Some(val) = env.get(v) {
                    val < bound
                } else {
                    true
                }
            }
            Constraint::VarGreaterThan(v, bound) => {
                if let Some(val) = env.get(v) {
                    val > bound
                } else {
                    true
                }
            }
            Constraint::NotEquals(v, target) => {
                if let Some(val) = env.get(v) {
                    val != target
                } else {
                    true
                }
            }
            Constraint::LinearGe(terms, rhs) => {
                let mut sum = 0i64;
                for (coeff, var) in terms {
                    if let Some(val) = env.get(var) {
                        sum += coeff * val;
                    } else {
                        return true;
                    }
                }
                sum >= *rhs
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Witness {
    pub assignments: HashMap<String, i64>,
    pub witness_id: ORID,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proof {
    pub unsat_core: Vec<Constraint>,
    pub proof_id: ORID,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetStop {
    pub reason: String,
    pub steps_taken: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolveResult<W> {
    Sat(W),
    Unsat(Proof),
    Unknown(BudgetStop),
}

pub use reference::ReferenceSolver;

pub fn crate_name() -> &'static str {
    "origin-constraints"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraints_boundary() {
        assert_eq!(crate_name(), "origin-constraints");
    }
}
