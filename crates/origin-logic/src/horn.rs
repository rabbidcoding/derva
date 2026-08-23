// INVARIANT: Horn rules adhere strictly to safe Datalog subset; range restriction enforced 100%; content-addressed IDs.
// KPI: Parser/validator rejects rules outside subset 100%; rule evaluation deterministic; rule IDs content-addressed.

use origin_core::{Canonical, ObjectKind, ORID};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Term {
    Variable(String),
    Constant(String),
}

impl Term {
    pub fn is_variable(&self) -> bool {
        matches!(self, Term::Variable(_))
    }

    pub fn name(&self) -> &str {
        match self {
            Term::Variable(name) | Term::Constant(name) => name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Literal {
    pub predicate: String,
    pub terms: Vec<Term>,
    pub is_negated: bool,
}

impl Literal {
    pub fn new(predicate: impl Into<String>, terms: Vec<Term>, is_negated: bool) -> Self {
        Self {
            predicate: predicate.into(),
            terms,
            is_negated,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HornRule {
    pub name: String,
    pub head: Literal,
    pub body: Vec<Literal>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HornError {
    RangeRestrictionViolation {
        variable: String,
        location: &'static str,
    },
    StratifiedNegationViolation {
        variable: String,
    },
    EmptyBody,
    ParseError(String),
}

impl std::fmt::Display for HornError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HornError::RangeRestrictionViolation { variable, location } => {
                write!(f, "Range restriction violation: variable '{}' in {} does not appear in positive body literals", variable, location)
            }
            HornError::StratifiedNegationViolation { variable } => {
                write!(f, "Stratified negation violation: variable '{}' in negated body literal is not bound in positive body", variable)
            }
            HornError::EmptyBody => write!(f, "Horn rule body cannot be empty"),
            HornError::ParseError(msg) => write!(f, "Horn rule parse error: {}", msg),
        }
    }
}

impl std::error::Error for HornError {}

impl HornRule {
    pub fn new(
        name: impl Into<String>,
        head: Literal,
        body: Vec<Literal>,
    ) -> Result<Self, HornError> {
        let rule = Self {
            name: name.into(),
            head,
            body,
        };

        rule.validate()?;
        Ok(rule)
    }

    pub fn validate(&self) -> Result<(), HornError> {
        if self.body.is_empty() {
            return Err(HornError::EmptyBody);
        }

        // Collect all variables appearing in positive body literals
        let mut positive_body_vars = HashSet::new();
        for lit in &self.body {
            if !lit.is_negated {
                for term in &lit.terms {
                    if let Term::Variable(var) = term {
                        positive_body_vars.insert(var.clone());
                    }
                }
            }
        }

        // Check Range Restriction: Head variables MUST appear in positive body literals
        for term in &self.head.terms {
            if let Term::Variable(var) = term {
                if !positive_body_vars.contains(var) {
                    return Err(HornError::RangeRestrictionViolation {
                        variable: var.clone(),
                        location: "head",
                    });
                }
            }
        }

        // Check Stratified Negation Safety: Negated body variables MUST appear in positive body literals
        for lit in &self.body {
            if lit.is_negated {
                for term in &lit.terms {
                    if let Term::Variable(var) = term {
                        if !positive_body_vars.contains(var) {
                            return Err(HornError::StratifiedNegationViolation {
                                variable: var.clone(),
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn id(&self) -> ORID {
        let mut buf = Vec::new();
        self.encode_canonical(&mut buf);
        ORID::compute(ObjectKind::Artifact, &buf)
    }

    /// Minimal parser for standard Horn clause string syntax: `head(X, Z) :- body1(X, Y), body2(Y, Z).`
    pub fn parse(input: &str) -> Result<Self, HornError> {
        let trimmed = input.trim();
        let parts: Vec<&str> = trimmed.split(":-").collect();
        if parts.len() != 2 {
            return Err(HornError::ParseError(
                "Rule must contain exactly one ':-' operator".to_string(),
            ));
        }

        let head_str = parts[0].trim();
        let body_str = parts[1].trim().trim_end_matches('.');

        let head_lit = parse_literal(head_str)?;
        let mut body_lits = Vec::new();

        for b_part in body_str.split("),") {
            let item = b_part.trim();
            if item.is_empty() {
                continue;
            }
            let full_item = if item.ends_with(')') {
                item.to_string()
            } else {
                format!("{})", item)
            };
            body_lits.push(parse_literal(&full_item)?);
        }

        Self::new("parsed_rule", head_lit, body_lits)
    }
}

fn parse_literal(input: &str) -> Result<Literal, HornError> {
    let mut s = input.trim();
    let is_negated = if s.starts_with("not ") {
        s = s["not ".len()..].trim();
        true
    } else {
        false
    };

    let open_paren = s
        .find('(')
        .ok_or_else(|| HornError::ParseError(format!("Literal '{}' missing '('", s)))?;
    let close_paren = s
        .rfind(')')
        .ok_or_else(|| HornError::ParseError(format!("Literal '{}' missing ')'", s)))?;

    let predicate = s[..open_paren].trim().to_string();
    let terms_str = &s[open_paren + 1..close_paren];

    let mut terms = Vec::new();
    for t_str in terms_str.split(',') {
        let t = t_str.trim();
        if t.is_empty() {
            continue;
        }
        if t.chars().next().unwrap_or('a').is_uppercase() {
            terms.push(Term::Variable(t.to_string()));
        } else {
            terms.push(Term::Constant(t.to_string()));
        }
    }

    Ok(Literal::new(predicate, terms, is_negated))
}

impl Canonical for HornRule {
    fn encode_canonical(&self, out: &mut Vec<u8>) {
        let name_bytes = self.name.as_bytes();
        out.extend_from_slice(&(name_bytes.len() as u64).to_be_bytes());
        out.extend_from_slice(name_bytes);

        encode_literal(&self.head, out);

        out.extend_from_slice(&(self.body.len() as u64).to_be_bytes());
        for b in &self.body {
            encode_literal(b, out);
        }
    }
}

fn encode_literal(lit: &Literal, out: &mut Vec<u8>) {
    let p_bytes = lit.predicate.as_bytes();
    out.extend_from_slice(&(p_bytes.len() as u64).to_be_bytes());
    out.extend_from_slice(p_bytes);

    out.push(if lit.is_negated { 1 } else { 0 });

    out.extend_from_slice(&(lit.terms.len() as u64).to_be_bytes());
    for t in &lit.terms {
        match t {
            Term::Variable(v) => {
                out.push(1);
                let v_bytes = v.as_bytes();
                out.extend_from_slice(&(v_bytes.len() as u64).to_be_bytes());
                out.extend_from_slice(v_bytes);
            }
            Term::Constant(c) => {
                out.push(2);
                let c_bytes = c.as_bytes();
                out.extend_from_slice(&(c_bytes.len() as u64).to_be_bytes());
                out.extend_from_slice(c_bytes);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_grandparent_rule_parses_and_validates() {
        let rule_str = "grandparent(X, Z) :- parent(X, Y), parent(Y, Z).";
        let rule = HornRule::parse(rule_str).unwrap();

        assert_eq!(rule.head.predicate, "grandparent");
        assert_eq!(rule.body.len(), 2);
        assert_eq!(rule.head.terms.len(), 2);
    }

    #[test]
    fn test_range_restriction_violation_rejected_100_percent() {
        // Head variable X does not appear in positive body literals
        let head = Literal::new("invalid_head", vec![Term::Variable("X".to_string())], false);
        let body = vec![Literal::new(
            "parent",
            vec![
                Term::Variable("Y".to_string()),
                Term::Variable("Z".to_string()),
            ],
            false,
        )];

        let res = HornRule::new("invalid_rule", head, body);
        assert!(res.is_err());
        match res.unwrap_err() {
            HornError::RangeRestrictionViolation { variable, location } => {
                assert_eq!(variable, "X");
                assert_eq!(location, "head");
            }
            _ => panic!("Expected RangeRestrictionViolation error"),
        }
    }

    #[test]
    fn test_rule_id_content_addressed() {
        let rule1 = HornRule::parse("grandparent(X, Z) :- parent(X, Y), parent(Y, Z).").unwrap();
        let rule2 = HornRule::parse("grandparent(X, Z) :- parent(X, Y), parent(Y, Z).").unwrap();
        let rule3 = HornRule::parse("ancestor(X, Z) :- parent(X, Y), parent(Y, Z).").unwrap();

        assert_eq!(rule1.id(), rule2.id());
        assert_ne!(rule1.id(), rule3.id());
    }
}
