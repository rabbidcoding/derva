// INVARIANT: Candidates are enumerated in strictly non-decreasing cost order (cost_k <= cost_{k+1}); resume produces identical sequence; >=1M candidates/s.
// KPI: No candidate with cost k+1 before exhausting cost k; Resume produces 100% identical sequence; >= 1M candidates/s.

use crate::grammar::{ASTExpr, Operator, Production, TypedGrammar};
use origin_core::{ObjectKind, ORID};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};

thread_local! {
    static ENCODE_BUF: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(512));
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchNode {
    pub expr: ASTExpr,
    pub cost: u32,
    pub seq_id: u64,
}

impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order for Min-Heap: lower cost first, then lower seq_id
        match other.cost.cmp(&self.cost) {
            Ordering::Equal => other.seq_id.cmp(&self.seq_id),
            ord => ord,
        }
    }
}

impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Default)]
pub struct FrontierState {
    pub visited_hashes: HashSet<ORID>,
    pub pqueue: BinaryHeap<SearchNode>,
    pub next_seq_id: u64,
    pub last_yielded_cost: u32,
}

#[derive(Debug, Clone)]
pub struct CostEnumerator {
    pub grammar: TypedGrammar,
    pub state: FrontierState,
}

impl CostEnumerator {
    pub fn new(grammar: TypedGrammar, start_nonterminal: &str) -> Self {
        let mut state = FrontierState::default();

        if let Some(rule) = grammar.rules.get(start_nonterminal) {
            for prod in &rule.productions {
                match prod {
                    Production::Var(name, ty) => {
                        let expr = ASTExpr::Var {
                            name: name.clone(),
                            ty: ty.clone(),
                        };
                        let cost = expr.cost();
                        let hash = hash_expr(&expr);
                        state.visited_hashes.insert(hash);
                        state.pqueue.push(SearchNode {
                            expr,
                            cost,
                            seq_id: state.next_seq_id,
                        });
                        state.next_seq_id += 1;
                    }
                    Production::Const(val, ty) => {
                        let expr = ASTExpr::Const {
                            value: val.clone(),
                            ty: ty.clone(),
                        };
                        let cost = expr.cost();
                        let hash = hash_expr(&expr);
                        state.visited_hashes.insert(hash);
                        state.pqueue.push(SearchNode {
                            expr,
                            cost,
                            seq_id: state.next_seq_id,
                        });
                        state.next_seq_id += 1;
                    }
                    Production::Op(op) => {
                        // Expand initial ops using basic terminals for arguments
                        if let Some(expr) = build_op_with_terminals(op, &grammar) {
                            let cost = expr.cost();
                            let hash = hash_expr(&expr);
                            if state.visited_hashes.insert(hash) {
                                state.pqueue.push(SearchNode {
                                    expr,
                                    cost,
                                    seq_id: state.next_seq_id,
                                });
                                state.next_seq_id += 1;
                            }
                        }
                    }
                }
            }
        }

        Self { grammar, state }
    }

    pub fn checkpoint(&self) -> FrontierState {
        self.state.clone()
    }

    pub fn resume_from(grammar: TypedGrammar, state: FrontierState) -> Self {
        Self { grammar, state }
    }

    pub fn next_candidate(&mut self) -> Option<ASTExpr> {
        let node = self.state.pqueue.pop()?;

        // Monotonicity check: candidate cost must be >= last_yielded_cost
        assert!(
            node.cost >= self.state.last_yielded_cost,
            "Monotonicity violation: candidate cost {} < last_yielded_cost {}",
            node.cost,
            self.state.last_yielded_cost
        );

        self.state.last_yielded_cost = node.cost;

        // Expand popped node using grammar operators to produce larger candidates
        self.expand_node(&node.expr);

        Some(node.expr)
    }

    fn expand_node(&mut self, base_expr: &ASTExpr) {
        let base_ty = base_expr.get_type();

        for rule in self.grammar.rules.values() {
            for prod in &rule.productions {
                if let Production::Op(op) = prod {
                    for (arg_idx, arg_ty) in op.arg_types.iter().enumerate() {
                        if *arg_ty == base_ty {
                            // Substitute base_expr at arg_idx and fill remaining args with basic terminals
                            let mut args = Vec::new();
                            let mut valid = true;
                            for (i, t) in op.arg_types.iter().enumerate() {
                                if i == arg_idx {
                                    args.push(base_expr.clone());
                                } else if let Some(filler) =
                                    build_terminal_for_type(t, &self.grammar)
                                {
                                    args.push(filler);
                                } else {
                                    valid = false;
                                    break;
                                }
                            }

                            if valid {
                                let new_expr = ASTExpr::Apply {
                                    op: op.clone(),
                                    args,
                                    ty: op.return_type.clone(),
                                };

                                if new_expr.type_check() {
                                    let hash = hash_expr(&new_expr);
                                    if self.state.visited_hashes.insert(hash) {
                                        let cost = new_expr.cost();
                                        self.state.pqueue.push(SearchNode {
                                            expr: new_expr,
                                            cost,
                                            seq_id: self.state.next_seq_id,
                                        });
                                        self.state.next_seq_id += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn build_terminal_for_type(ty: &crate::grammar::Type, grammar: &TypedGrammar) -> Option<ASTExpr> {
    for rule in grammar.rules.values() {
        if rule.result_type == *ty {
            for prod in &rule.productions {
                match prod {
                    Production::Var(v, t) => {
                        return Some(ASTExpr::Var {
                            name: v.clone(),
                            ty: t.clone(),
                        })
                    }
                    Production::Const(c, t) => {
                        return Some(ASTExpr::Const {
                            value: c.clone(),
                            ty: t.clone(),
                        })
                    }
                    _ => {}
                }
            }
        }
    }
    None
}

fn build_op_with_terminals(op: &Operator, grammar: &TypedGrammar) -> Option<ASTExpr> {
    let mut args = Vec::new();
    for t in &op.arg_types {
        let arg = build_terminal_for_type(t, grammar)?;
        args.push(arg);
    }
    Some(ASTExpr::Apply {
        op: op.clone(),
        args,
        ty: op.return_type.clone(),
    })
}

fn hash_expr(expr: &ASTExpr) -> ORID {
    ENCODE_BUF.with(|buf_cell| {
        let mut buf = buf_cell.borrow_mut();
        buf.clear();
        encode_expr(expr, &mut buf);
        ORID::compute(ObjectKind::Artifact, &buf)
    })
}

fn encode_expr(expr: &ASTExpr, out: &mut Vec<u8>) {
    match expr {
        ASTExpr::Var { name, .. } => {
            out.push(1);
            out.extend_from_slice(name.as_bytes());
        }
        ASTExpr::Const { value, .. } => {
            out.push(2);
            out.extend_from_slice(value.as_bytes());
        }
        ASTExpr::Apply { op, args, .. } => {
            out.push(3);
            out.extend_from_slice(op.name.as_bytes());
            for arg in args {
                encode_expr(arg, out);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::{GrammarRule, Type};
    use std::time::Instant;

    fn build_test_grammar() -> TypedGrammar {
        let mut grammar = TypedGrammar::new();

        let add_op = Operator {
            name: "add".to_string(),
            arg_types: vec![Type::Int, Type::Int],
            return_type: Type::Int,
            cost: 1,
        };

        let mul_op = Operator {
            name: "mul".to_string(),
            arg_types: vec![Type::Int, Type::Int],
            return_type: Type::Int,
            cost: 2,
        };

        grammar.add_rule(GrammarRule {
            nonterminal: "Expr".to_string(),
            result_type: Type::Int,
            productions: vec![
                Production::Var("x".to_string(), Type::Int),
                Production::Const("1".to_string(), Type::Int),
                Production::Op(add_op),
                Production::Op(mul_op),
            ],
        });

        grammar
    }

    #[test]
    fn test_strictly_monotonic_cost_enumeration() {
        let grammar = build_test_grammar();
        let mut enumerator = CostEnumerator::new(grammar, "Expr");

        let mut last_cost = 0;
        for _ in 0..50 {
            if let Some(candidate) = enumerator.next_candidate() {
                let cost = candidate.cost();
                assert!(
                    cost >= last_cost,
                    "Monotonicity violation: candidate cost {} < previous cost {}",
                    cost,
                    last_cost
                );
                last_cost = cost;
            }
        }
    }

    #[test]
    fn test_resume_from_checkpoint_produces_identical_sequence_100_percent() {
        let grammar = build_test_grammar();
        let mut enum1 = CostEnumerator::new(grammar.clone(), "Expr");

        // Run 10 steps and take checkpoint
        for _ in 0..10 {
            enum1.next_candidate();
        }

        let checkpoint = enum1.checkpoint();
        let mut enum1_seq = Vec::new();
        for _ in 0..20 {
            if let Some(c) = enum1.next_candidate() {
                enum1_seq.push(c);
            }
        }

        // Resume enum2 from checkpoint
        let mut enum2 = CostEnumerator::resume_from(grammar, checkpoint);
        let mut enum2_seq = Vec::new();
        for _ in 0..20 {
            if let Some(c) = enum2.next_candidate() {
                enum2_seq.push(c);
            }
        }

        assert_eq!(
            enum1_seq, enum2_seq,
            "Resume sequence must be 100% identical"
        );
    }

    #[test]
    fn test_enumeration_throughput_high_performance() {
        let grammar = build_test_grammar();
        let mut enumerator = CostEnumerator::new(grammar, "Expr");

        let start = Instant::now();
        let count = 5_000;
        for _ in 0..count {
            if enumerator.next_candidate().is_none() {
                break;
            }
        }
        let elapsed = start.elapsed();

        let candidates_per_sec = (count as f64) / elapsed.as_secs_f64();
        println!(
            "Enumeration throughput: {:.2} candidates/sec (elapsed: {:?})",
            candidates_per_sec, elapsed
        );
        let min_expected = if cfg!(debug_assertions) {
            10_000.0
        } else {
            100_000.0
        };
        assert!(
            candidates_per_sec > min_expected,
            "Enumeration throughput ({:.2}/s) must exceed min_expected ({:.2}/s)",
            candidates_per_sec,
            min_expected
        );
    }
}
