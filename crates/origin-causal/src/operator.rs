#![forbid(unsafe_code)]

// INVARIANT: 100% operators type-check domains/codomains; missing precondition => operator not executable; risk/cost mandatory for INTERVENE.
// KPI: 100% type-checked schemas; missing precondition returns false/error; Interventional requires explicit risk and cost.

use origin_core::{CausalStatus, ObjectKind, ORID};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaId(pub String);

impl SchemaId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PredicateId(pub String);

impl PredicateId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EffectId(pub String);

impl EffectId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cost {
    pub value: u64,
    pub currency_or_unit: String,
}

impl Cost {
    pub fn new(value: u64, unit: impl Into<String>) -> Self {
        Self {
            value,
            currency_or_unit: unit.into(),
        }
    }

    pub fn zero() -> Self {
        Self {
            value: 0,
            currency_or_unit: "step".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Risk {
    pub score: f64, // 0.0 (safe) to 1.0 (critical risk)
    pub description: String,
}

impl Risk {
    pub fn new(score: f64, description: impl Into<String>) -> Self {
        let clamped = score.clamp(0.0, 1.0);
        Self {
            score: clamped,
            description: description.into(),
        }
    }

    pub fn zero() -> Self {
        Self {
            score: 0.0,
            description: "none".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperatorError {
    SchemaMismatch {
        expected: SchemaId,
        actual: SchemaId,
    },
    UnsatisfiedPrecondition(PredicateId),
    MissingCostOrRiskForIntervention,
    EmptyEvidence,
}

impl std::fmt::Display for OperatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperatorError::SchemaMismatch { expected, actual } => {
                write!(
                    f,
                    "Schema mismatch: expected {:?}, got {:?}",
                    expected, actual
                )
            }
            OperatorError::UnsatisfiedPrecondition(p) => {
                write!(f, "Precondition not satisfied: {:?}", p)
            }
            OperatorError::MissingCostOrRiskForIntervention => {
                write!(
                    f,
                    "Risk and cost are mandatory for INTERVENE (Interventional) operators"
                )
            }
            OperatorError::EmptyEvidence => {
                write!(f, "Evidence list cannot be empty for causal operators")
            }
        }
    }
}

impl std::error::Error for OperatorError {}

// AUDIT-LENSES: Lovelace, Guido, Ritchie
#[derive(Debug, Clone, PartialEq)]
pub struct CausalOperator {
    pub id: ORID,
    pub name: String,
    pub domain: SchemaId,
    pub codomain: SchemaId,
    pub preconditions: Vec<PredicateId>,
    pub effect: EffectId,
    pub evidence: Vec<ORID>,
    pub status: CausalStatus,
    pub cost: Option<Cost>,
    pub risk: Option<Risk>,
}

impl CausalOperator {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: impl Into<String>,
        domain: SchemaId,
        codomain: SchemaId,
        preconditions: Vec<PredicateId>,
        effect: EffectId,
        evidence: Vec<ORID>,
        status: CausalStatus,
        cost: Option<Cost>,
        risk: Option<Risk>,
    ) -> Result<Self, OperatorError> {
        let name_str = name.into();
        let id = ORID::compute(ObjectKind::Operator, name_str.as_bytes());

        // KPI: Risk/cost mandatory for INTERVENE (Interventional)
        if status == CausalStatus::Interventional && (cost.is_none() || risk.is_none()) {
            return Err(OperatorError::MissingCostOrRiskForIntervention);
        }

        Ok(Self {
            id,
            name: name_str,
            domain,
            codomain,
            preconditions,
            effect,
            evidence,
            status,
            cost,
            risk,
        })
    }

    /// Verifies domain/codomain schema type compatibility.
    /// INVARIANT: 100% operators type-check domains/codomains.
    pub fn type_check(
        &self,
        input_schema: &SchemaId,
        output_schema: &SchemaId,
    ) -> Result<(), OperatorError> {
        if self.domain != *input_schema {
            return Err(OperatorError::SchemaMismatch {
                expected: self.domain.clone(),
                actual: input_schema.clone(),
            });
        }
        if self.codomain != *output_schema {
            return Err(OperatorError::SchemaMismatch {
                expected: self.codomain.clone(),
                actual: output_schema.clone(),
            });
        }
        Ok(())
    }

    /// Evaluates if all preconditions are satisfied by active state facts.
    /// INVARIANT: Missing precondition => operator not executable.
    pub fn is_executable(&self, active_facts: &HashSet<PredicateId>) -> bool {
        self.preconditions
            .iter()
            .all(|pre| active_facts.contains(pre))
    }

    pub fn validate_execution(
        &self,
        active_facts: &HashSet<PredicateId>,
    ) -> Result<(), OperatorError> {
        for pre in &self.preconditions {
            if !active_facts.contains(pre) {
                return Err(OperatorError::UnsatisfiedPrecondition(pre.clone()));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_100_percent_operators_type_check_schemas() {
        let domain = SchemaId::new("StateA");
        let codomain = SchemaId::new("StateB");
        let wrong_domain = SchemaId::new("StateX");

        let op = CausalOperator::new(
            "transform",
            domain.clone(),
            codomain.clone(),
            vec![],
            EffectId::new("effect_1"),
            vec![],
            CausalStatus::Observational,
            None,
            None,
        )
        .unwrap();

        // Matching schemas MUST pass
        assert_eq!(op.type_check(&domain, &codomain), Ok(()));

        // Mismatched domain MUST fail with SchemaMismatch
        let res = op.type_check(&wrong_domain, &codomain);
        assert!(matches!(res, Err(OperatorError::SchemaMismatch { .. })));
    }

    #[test]
    fn test_missing_precondition_makes_operator_not_executable() {
        let pre1 = PredicateId::new("has_fuel");
        let pre2 = PredicateId::new("ignition_on");

        let op = CausalOperator::new(
            "start_engine",
            SchemaId::new("OffState"),
            SchemaId::new("RunningState"),
            vec![pre1.clone(), pre2.clone()],
            EffectId::new("engine_running"),
            vec![],
            CausalStatus::Observational,
            None,
            None,
        )
        .unwrap();

        let mut active_facts = HashSet::new();
        active_facts.insert(pre1.clone());

        // Missing pre2 => is_executable MUST be false
        assert!(
            !op.is_executable(&active_facts),
            "Operator MUST NOT be executable when precondition is missing"
        );
        assert_eq!(
            op.validate_execution(&active_facts),
            Err(OperatorError::UnsatisfiedPrecondition(pre2.clone()))
        );

        // Add pre2 => is_executable MUST be true
        active_facts.insert(pre2.clone());
        assert!(op.is_executable(&active_facts));
        assert_eq!(op.validate_execution(&active_facts), Ok(()));
    }

    #[test]
    fn test_risk_and_cost_mandatory_for_intervene() {
        let domain = SchemaId::new("S1");
        let codomain = SchemaId::new("S2");

        // Attempting to construct Interventional operator WITHOUT cost or risk MUST fail
        let res_no_cost = CausalOperator::new(
            "intervene_action",
            domain.clone(),
            codomain.clone(),
            vec![],
            EffectId::new("e1"),
            vec![],
            CausalStatus::Interventional,
            None, // Missing cost!
            Some(Risk::new(0.5, "med_risk")),
        );
        assert_eq!(
            res_no_cost.err(),
            Some(OperatorError::MissingCostOrRiskForIntervention)
        );

        let res_no_risk = CausalOperator::new(
            "intervene_action",
            domain.clone(),
            codomain.clone(),
            vec![],
            EffectId::new("e1"),
            vec![],
            CausalStatus::Interventional,
            Some(Cost::new(10, "usd")),
            None, // Missing risk!
        );
        assert_eq!(
            res_no_risk.err(),
            Some(OperatorError::MissingCostOrRiskForIntervention)
        );

        // Constructing Interventional operator WITH both cost and risk MUST succeed
        let res_valid = CausalOperator::new(
            "intervene_action",
            domain,
            codomain,
            vec![],
            EffectId::new("e1"),
            vec![],
            CausalStatus::Interventional,
            Some(Cost::new(10, "usd")),
            Some(Risk::new(0.2, "low_risk")),
        );
        assert!(res_valid.is_ok());
    }
}
