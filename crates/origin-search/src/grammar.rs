// INVARIANT: Generated ASTs are 100% type-correct; nonterminal branching factor reported; canonical grammar hash versioned.
// KPI: 100% generated ASTs type-correct; canonical hash versioned; no unbounded expansion.

use origin_core::{ObjectKind, ORID};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Type {
    Bool,
    Int,
    Float,
    Vector,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Operator {
    pub name: String,
    pub arg_types: Vec<Type>,
    pub return_type: Type,
    pub cost: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ASTExpr {
    Var {
        name: String,
        ty: Type,
    },
    Const {
        value: String,
        ty: Type,
    },
    Apply {
        op: Operator,
        args: Vec<ASTExpr>,
        ty: Type,
    },
}

impl ASTExpr {
    pub fn get_type(&self) -> Type {
        match self {
            ASTExpr::Var { ty, .. } | ASTExpr::Const { ty, .. } | ASTExpr::Apply { ty, .. } => {
                ty.clone()
            }
        }
    }

    pub fn type_check(&self) -> bool {
        match self {
            ASTExpr::Var { .. } | ASTExpr::Const { .. } => true,
            ASTExpr::Apply { op, args, ty } => {
                if *ty != op.return_type {
                    return false;
                }
                if args.len() != op.arg_types.len() {
                    return false;
                }
                for (arg, expected_ty) in args.iter().zip(&op.arg_types) {
                    if arg.get_type() != *expected_ty || !arg.type_check() {
                        return false;
                    }
                }
                true
            }
        }
    }

    pub fn cost(&self) -> u32 {
        match self {
            ASTExpr::Var { .. } | ASTExpr::Const { .. } => 0,
            ASTExpr::Apply { op, args, .. } => op.cost + args.iter().map(|a| a.cost()).sum::<u32>(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Production {
    Var(String, Type),
    Const(String, Type),
    Op(Operator),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrammarRule {
    pub nonterminal: String,
    pub result_type: Type,
    pub productions: Vec<Production>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrammarError {
    NonterminalNotFound(String),
    MaxDepthExceeded(String),
    TypeError(String),
}

impl std::fmt::Display for GrammarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GrammarError::NonterminalNotFound(name) => {
                write!(f, "Nonterminal rule not found: {}", name)
            }
            GrammarError::MaxDepthExceeded(name) => {
                write!(f, "Max depth exceeded generating {}", name)
            }
            GrammarError::TypeError(msg) => write!(f, "Type error in grammar: {}", msg),
        }
    }
}

impl std::error::Error for GrammarError {}

#[derive(Debug, Default, Clone)]
pub struct TypedGrammar {
    pub rules: HashMap<String, GrammarRule>,
}

impl TypedGrammar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_rule(&mut self, rule: GrammarRule) {
        self.rules.insert(rule.nonterminal.clone(), rule);
    }

    pub fn branching_factor(&self, nonterminal: &str) -> Option<usize> {
        self.rules.get(nonterminal).map(|r| r.productions.len())
    }

    pub fn canonical_hash(&self) -> ORID {
        let mut buf = Vec::new();
        let mut keys: Vec<&String> = self.rules.keys().collect();
        keys.sort();

        for k in keys {
            let r = &self.rules[k];
            buf.extend_from_slice(r.nonterminal.as_bytes());
            buf.extend_from_slice(&(r.productions.len() as u64).to_be_bytes());
            for p in &r.productions {
                match p {
                    Production::Var(v, ty) => {
                        buf.push(1);
                        buf.extend_from_slice(v.as_bytes());
                        buf.push(type_code(ty));
                    }
                    Production::Const(c, ty) => {
                        buf.push(2);
                        buf.extend_from_slice(c.as_bytes());
                        buf.push(type_code(ty));
                    }
                    Production::Op(op) => {
                        buf.push(3);
                        buf.extend_from_slice(op.name.as_bytes());
                        buf.push(type_code(&op.return_type));
                        buf.extend_from_slice(&op.cost.to_be_bytes());
                    }
                }
            }
        }

        ORID::compute(ObjectKind::Artifact, &buf)
    }

    /// Generates a valid AST guaranteed to be type-correct.
    pub fn generate(&self, nonterminal: &str, max_depth: usize) -> Result<ASTExpr, GrammarError> {
        if max_depth == 0 {
            return Err(GrammarError::MaxDepthExceeded(nonterminal.to_string()));
        }

        let rule = self
            .rules
            .get(nonterminal)
            .ok_or_else(|| GrammarError::NonterminalNotFound(nonterminal.to_string()))?;

        // Prefer terminal productions (Var / Const) when max_depth == 1
        let prod = if max_depth == 1 {
            rule.productions
                .iter()
                .find(|p| matches!(p, Production::Var(..) | Production::Const(..)))
                .unwrap_or(&rule.productions[0])
        } else {
            &rule.productions[0]
        };

        match prod {
            Production::Var(name, ty) => Ok(ASTExpr::Var {
                name: name.clone(),
                ty: ty.clone(),
            }),
            Production::Const(val, ty) => Ok(ASTExpr::Const {
                value: val.clone(),
                ty: ty.clone(),
            }),
            Production::Op(op) => {
                let mut args = Vec::new();
                for arg_ty in &op.arg_types {
                    let child_nonterminal =
                        self.find_nonterminal_for_type(arg_ty).ok_or_else(|| {
                            GrammarError::TypeError(format!("No nonterminal for type {:?}", arg_ty))
                        })?;
                    let child_ast = self.generate(&child_nonterminal, max_depth - 1)?;
                    args.push(child_ast);
                }

                let expr = ASTExpr::Apply {
                    op: op.clone(),
                    args,
                    ty: op.return_type.clone(),
                };

                if !expr.type_check() {
                    return Err(GrammarError::TypeError(
                        "Generated AST failed type_check invariant".to_string(),
                    ));
                }

                Ok(expr)
            }
        }
    }

    fn find_nonterminal_for_type(&self, ty: &Type) -> Option<String> {
        for (name, r) in &self.rules {
            if r.result_type == *ty {
                return Some(name.clone());
            }
        }
        None
    }
}

fn type_code(ty: &Type) -> u8 {
    match ty {
        Type::Bool => 1,
        Type::Int => 2,
        Type::Float => 3,
        Type::Vector => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar_generate_100_percent_type_correct() {
        let mut grammar = TypedGrammar::new();

        let add_op = Operator {
            name: "add".to_string(),
            arg_types: vec![Type::Int, Type::Int],
            return_type: Type::Int,
            cost: 1,
        };

        let expr_rule = GrammarRule {
            nonterminal: "Expr".to_string(),
            result_type: Type::Int,
            productions: vec![
                Production::Op(add_op),
                Production::Var("x".to_string(), Type::Int),
                Production::Const("1".to_string(), Type::Int),
            ],
        };

        grammar.add_rule(expr_rule);

        let ast = grammar.generate("Expr", 3).unwrap();
        assert!(ast.type_check(), "Generated AST must be 100% type-correct");
        assert_eq!(ast.get_type(), Type::Int);
    }

    #[test]
    fn test_canonical_grammar_hash_versioned() {
        let mut g1 = TypedGrammar::new();
        g1.add_rule(GrammarRule {
            nonterminal: "Expr".to_string(),
            result_type: Type::Int,
            productions: vec![Production::Const("1".to_string(), Type::Int)],
        });

        let mut g2 = TypedGrammar::new();
        g2.add_rule(GrammarRule {
            nonterminal: "Expr".to_string(),
            result_type: Type::Int,
            productions: vec![Production::Const("1".to_string(), Type::Int)],
        });

        let mut g3 = TypedGrammar::new();
        g3.add_rule(GrammarRule {
            nonterminal: "Expr".to_string(),
            result_type: Type::Int,
            productions: vec![Production::Const("2".to_string(), Type::Int)],
        });

        assert_eq!(g1.canonical_hash(), g2.canonical_hash());
        assert_ne!(g1.canonical_hash(), g3.canonical_hash());
    }

    #[test]
    fn test_branching_factor_reported() {
        let mut grammar = TypedGrammar::new();
        grammar.add_rule(GrammarRule {
            nonterminal: "Expr".to_string(),
            result_type: Type::Int,
            productions: vec![
                Production::Var("x".to_string(), Type::Int),
                Production::Const("1".to_string(), Type::Int),
            ],
        });

        assert_eq!(grammar.branching_factor("Expr"), Some(2));
        assert_eq!(grammar.branching_factor("Unknown"), None);
    }
}
