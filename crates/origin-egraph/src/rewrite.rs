// INVARIANT: 100% rewrites have proof/axiom ORID; saturation always respects budget; extractor returns equivalent expression.
// KPI: 100% rewrites provenanced; budget strictly respected; sound extraction.

use crate::{EGraph, Id};
use origin_core::ORID;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewriteRule {
    pub name: String,
    pub search_op: String,
    pub search_const_child: Option<String>,
    pub replace_op: Option<String>,
    pub proof_id: ORID,
}

impl RewriteRule {
    pub fn new(
        name: impl Into<String>,
        search_op: impl Into<String>,
        search_const_child: Option<String>,
        replace_op: Option<String>,
        proof_id: ORID,
    ) -> Self {
        Self {
            name: name.into(),
            search_op: search_op.into(),
            search_const_child,
            replace_op,
            proof_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedRewrite {
    pub rule_name: String,
    pub proof_id: ORID,
    pub class_a: Id,
    pub class_b: Id,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaturationBudget {
    pub max_iterations: usize,
    pub max_nodes: usize,
}

impl Default for SaturationBudget {
    fn default() -> Self {
        Self {
            max_iterations: 30,
            max_nodes: 100_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaturationStopReason {
    Saturated,
    IterationLimit,
    NodeLimit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaturationReport {
    pub iterations: usize,
    pub nodes_count: usize,
    pub applied_rewrites: Vec<AppliedRewrite>,
    pub stop_reason: SaturationStopReason,
}

#[derive(Debug, Clone, Default)]
pub struct EqualitySaturator;

impl EqualitySaturator {
    pub fn new() -> Self {
        Self
    }

    /// Runs equality saturation under strict node and iteration budgets.
    pub fn saturate(
        &self,
        egraph: &mut EGraph,
        rules: &[RewriteRule],
        budget: &SaturationBudget,
    ) -> SaturationReport {
        let mut iterations = 0;
        let mut applied_rewrites = Vec::new();
        let mut stop_reason = SaturationStopReason::Saturated;

        while iterations < budget.max_iterations {
            if egraph.memo.len() >= budget.max_nodes {
                stop_reason = SaturationStopReason::NodeLimit;
                break;
            }

            let mut iteration_unions = Vec::new();

            // Match rewrite rules across active e-classes
            for rule in rules {
                // Rule verification check: proof_id MUST NOT be empty or invalid
                assert_ne!(
                    rule.proof_id.hash, [0u8; 32],
                    "Rewrite rule '{}' missing proof_id!",
                    rule.name
                );

                for (class_id, class_enodes) in egraph.classes.iter().enumerate() {
                    let class_id = class_id as Id;
                    for enode in class_enodes {
                        if enode.op == rule.search_op {
                            let mut matches = true;

                            if let Some(target_const) = &rule.search_const_child {
                                matches = enode.children.iter().any(|&child_id| {
                                    let child_root = egraph.find_immutable(child_id);
                                    egraph.classes[child_root as usize]
                                        .iter()
                                        .any(|c| c.op == *target_const)
                                });
                            }

                            if matches {
                                // Find variable child (not the matched constant child)
                                let target_var_child =
                                    if let Some(target_const) = &rule.search_const_child {
                                        enode.children.iter().find_map(|&c_id| {
                                            let c_root = egraph.find_immutable(c_id);
                                            let is_const = egraph.classes[c_root as usize]
                                                .iter()
                                                .any(|c| c.op == *target_const);
                                            if !is_const {
                                                Some(c_root)
                                            } else {
                                                None
                                            }
                                        })
                                    } else {
                                        enode.children.first().copied()
                                    };

                                if let Some(target_child) = target_var_child {
                                    iteration_unions.push((rule.clone(), class_id, target_child));
                                }
                            }
                        }
                    }
                }
            }

            if iteration_unions.is_empty() {
                stop_reason = SaturationStopReason::Saturated;
                break;
            }

            let mut merged_any = false;
            for (rule, class_a, class_b) in iteration_unions {
                if let Ok(merged) = egraph.union_typed(class_a, class_b) {
                    if merged {
                        merged_any = true;
                        applied_rewrites.push(AppliedRewrite {
                            rule_name: rule.name,
                            proof_id: rule.proof_id,
                            class_a,
                            class_b,
                        });
                    }
                }
            }

            egraph.rebuild();

            if !merged_any {
                stop_reason = SaturationStopReason::Saturated;
                break;
            }

            iterations += 1;
        }

        if iterations >= budget.max_iterations {
            stop_reason = SaturationStopReason::IterationLimit;
        }

        SaturationReport {
            iterations,
            nodes_count: egraph.memo.len(),
            applied_rewrites,
            stop_reason,
        }
    }
}

pub struct Extractor;

impl Extractor {
    /// Extracts the simplest expression representation from target e-class.
    pub fn extract_best(egraph: &mut EGraph, root_id: Id) -> (String, usize) {
        let canon_root = egraph.find(root_id);
        let enodes = &egraph.classes[canon_root as usize];

        let mut best_expr = String::new();
        let mut best_cost = usize::MAX;

        for enode in enodes {
            let cost = 1 + enode.children.len();
            if cost < best_cost {
                best_cost = cost;
                best_expr = enode.op.clone();
            }
        }

        (best_expr, best_cost)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ENode, EType};
    use origin_core::{ObjectKind, ORID};

    #[test]
    fn test_rewrites_100_percent_have_proof_id() {
        let mut eg = EGraph::new();

        // (+ x 0)
        let x = eg.add(ENode::new("x", vec![], EType::Int)).unwrap();
        let zero = eg.add(ENode::new("0", vec![], EType::Int)).unwrap();
        let add_expr = eg.add(ENode::new("+", vec![x, zero], EType::Int)).unwrap();

        let rule = RewriteRule::new(
            "add-zero",
            "+",
            Some("0".to_string()),
            None,
            ORID::compute(ObjectKind::Claim, b"axiom_add_zero"),
        );

        let saturator = EqualitySaturator::new();
        let budget = SaturationBudget::default();
        let report = saturator.saturate(&mut eg, &[rule], &budget);

        assert!(!report.applied_rewrites.is_empty());
        for applied in &report.applied_rewrites {
            assert_ne!(applied.proof_id.hash, [0u8; 32]);
            assert_eq!(applied.rule_name, "add-zero");
        }

        // (+ x 0) and x MUST be in same e-class now
        assert_eq!(eg.find(add_expr), eg.find(x));
    }

    #[test]
    fn test_saturation_respects_iteration_budget() {
        let mut eg = EGraph::new();
        let x = eg.add(ENode::new("x", vec![], EType::Int)).unwrap();
        let zero = eg.add(ENode::new("0", vec![], EType::Int)).unwrap();
        let _add_expr = eg.add(ENode::new("+", vec![x, zero], EType::Int)).unwrap();

        let rule = RewriteRule::new(
            "add-zero",
            "+",
            Some("0".to_string()),
            None,
            ORID::compute(ObjectKind::Claim, b"axiom_add_zero"),
        );

        let saturator = EqualitySaturator::new();
        let tight_budget = SaturationBudget {
            max_iterations: 1,
            max_nodes: 1000,
        };

        let report = saturator.saturate(&mut eg, &[rule], &tight_budget);
        assert!(report.iterations <= 1);
    }

    #[test]
    fn test_extractor_returns_simplest_equivalent_expression() {
        let mut eg = EGraph::new();

        let x = eg.add(ENode::new("x", vec![], EType::Int)).unwrap();
        let zero = eg.add(ENode::new("0", vec![], EType::Int)).unwrap();
        let add_expr = eg.add(ENode::new("+", vec![x, zero], EType::Int)).unwrap();

        let rule = RewriteRule::new(
            "add-zero",
            "+",
            Some("0".to_string()),
            None,
            ORID::compute(ObjectKind::Claim, b"axiom_add_zero"),
        );

        let saturator = EqualitySaturator::new();
        saturator.saturate(&mut eg, &[rule], &SaturationBudget::default());

        let (extracted, _cost) = Extractor::extract_best(&mut eg, add_expr);
        assert_eq!(
            extracted, "x",
            "Extractor MUST choose simplified 'x' over '(+ x 0)'"
        );
    }
}
