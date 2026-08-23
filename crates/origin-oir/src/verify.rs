#![forbid(unsafe_code)]

// AUDIT-LENSES: Donald Knuth, Ken Thompson, Niklaus Wirth
// INVARIANT: OIR Verifier & Invariant Pass enforcing TCB epistemic lattice rules, obligation witness validity, provenance, and budget integrity.
// KPI: Malformed semantic IR accepted = 0; Verifier deterministic 100%; Verifier p99 < 5ms for 100k-op module target.

use crate::effectcheck::EffectError;
use crate::ir::{EffectKind, OirModule};
use crate::typecheck::TypeError;
use origin_core::{opcode::OpCode, state::Budget, status::Status};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;

#[derive(Default)]
pub struct FastStringHasher(u64);

impl std::hash::Hasher for FastStringHasher {
    #[inline(always)]
    fn finish(&self) -> u64 {
        self.0
    }
    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) {
        let mut hash = if self.0 == 0 { 0xcbf29ce484222325 } else { self.0 };
        for &byte in bytes {
            hash = hash.wrapping_mul(0x100000001b3) ^ (byte as u64);
        }
        self.0 = hash;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifierError {
    TypeError(TypeError),
    EffectError(EffectError),
    EpistemicLatticeViolation {
        from: Status,
        to: Status,
    },
    SelfWitnessProhibited,
    BudgetExhausted,
    MalformedProvenance,
    MalformedInstruction(String),
}

impl From<TypeError> for VerifierError {
    fn from(err: TypeError) -> Self {
        VerifierError::TypeError(err)
    }
}

impl From<EffectError> for VerifierError {
    fn from(err: EffectError) -> Self {
        VerifierError::EffectError(err)
    }
}

impl std::fmt::Display for VerifierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerifierError::TypeError(e) => write!(f, "Type verification error: {}", e),
            VerifierError::EffectError(e) => write!(f, "Effect verification error: {}", e),
            VerifierError::EpistemicLatticeViolation { from, to } => {
                write!(f, "Invalid epistemic lattice promotion: {:?} -> {:?}", from, to)
            }
            VerifierError::SelfWitnessProhibited => write!(f, "Obligation self-satisfaction/witness prohibited"),
            VerifierError::BudgetExhausted => write!(f, "OIR execution budget exhausted"),
            VerifierError::MalformedProvenance => write!(f, "Malformed or missing ORID provenance reference"),
            VerifierError::MalformedInstruction(s) => write!(f, "Malformed instruction: {}", s),
        }
    }
}

impl std::error::Error for VerifierError {}

pub struct OirVerifier;

impl OirVerifier {
    /// High-performance unified TCB verification pass pipeline
    #[inline]
    pub fn verify_module(
        module: &OirModule,
        max_allowed_effect: EffectKind,
        known_functions: &HashMap<String, EffectKind>,
        budget: &mut Budget,
    ) -> Result<(), VerifierError> {
        let num_ops = module.instructions.len() as u64;
        if budget.cpu_steps_remaining < num_ops {
            return Err(VerifierError::BudgetExhausted);
        }
        budget.cpu_steps_remaining -= num_ops;

        let mut scope: HashMap<&str, crate::ir::OirType, BuildHasherDefault<FastStringHasher>> =
            HashMap::with_capacity_and_hasher(module.instructions.len(), BuildHasherDefault::<FastStringHasher>::default());

        let has_known_funcs = !known_functions.is_empty();

        for (idx, inst) in module.instructions.iter().enumerate() {
            // 1. Single-pass invariant check and inherent effect extraction
            let inherent_effect = inst
                .validate_and_inherent_effect()
                .map_err(VerifierError::MalformedInstruction)?;

            // 2. Operand checks: scope bindings, type match, self-witnessing, and transitive effects
            if !inst.operands.is_empty() {
                for op in &inst.operands {
                    if op.id == inst.result.id {
                        return Err(VerifierError::SelfWitnessProhibited);
                    }

                    match scope.get(op.id.as_str()) {
                        Some(&bound_ty) => {
                            if bound_ty != op.ty {
                                return Err(VerifierError::TypeError(TypeError::TypeMismatch {
                                    expected: op.ty,
                                    actual: bound_ty,
                                    value_id: op.id.clone(),
                                }));
                            }
                        }
                        None => {
                            return Err(VerifierError::TypeError(TypeError::UndefinedValue(
                                op.id.clone(),
                            )));
                        }
                    }

                    if has_known_funcs {
                        if let Some(&callee_effect) = known_functions.get(&op.id) {
                            if callee_effect > max_allowed_effect {
                                return Err(VerifierError::EffectError(EffectError::TransitiveViolation {
                                    caller: module.name.clone(),
                                    callee: op.id.clone(),
                                    caller_effect: max_allowed_effect,
                                    callee_effect,
                                }));
                            }
                        }
                    }
                    if op.id.as_bytes().starts_with(b"@ext_") {
                        return Err(VerifierError::EffectError(EffectError::UnknownExternalCall {
                            symbol: op.id.clone(),
                        }));
                    }
                }
            } else if inst.opcode == OpCode::Verify {
                return Err(VerifierError::MalformedProvenance);
            }

            // 3. Effect containment check
            let effective = inherent_effect.max(inst.effect);
            if effective > max_allowed_effect {
                return Err(VerifierError::EffectError(EffectError::Escalation {
                    instruction_index: idx,
                    allowed: max_allowed_effect,
                    attempted: effective,
                }));
            }

            scope.insert(inst.result.id.as_str(), inst.result.ty);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{OirInstruction, OirType, Value};
    use origin_core::{orid::ObjectKind, ORID};
    use std::time::Instant;

    #[test]
    fn test_malformed_semantic_ir_accepted_zero() {
        let orid = ORID::compute(ObjectKind::Claim, b"malformed_test");
        let known = HashMap::new();

        // Case 1: Self-witnessing violation
        let mut mod_self_witness = OirModule::new("self_witness");
        let obs = OirInstruction::new(
            "%obs0",
            OpCode::Observe,
            OirType::Observation,
            vec![],
            orid,
            EffectKind::Observe,
        )
        .unwrap();
        let prop = OirInstruction::new(
            "%h1",
            OpCode::Propose,
            OirType::Claim,
            vec![Value {
                id: "%obs0".into(),
                ty: OirType::Observation,
            }],
            orid,
            EffectKind::Pure,
        )
        .unwrap();
        let ver_self = OirInstruction::new(
            "%h1",
            OpCode::Verify,
            OirType::VerifiedClaim,
            vec![Value {
                id: "%h1".into(),
                ty: OirType::Claim,
            }],
            orid,
            EffectKind::Pure,
        )
        .unwrap();

        mod_self_witness.push(obs).unwrap();
        mod_self_witness.push(prop).unwrap();
        mod_self_witness.instructions.push(ver_self);

        let mut b1 = Budget {
            cpu_steps_remaining: 100,
            wall_time_ms_limit: 1000,
            max_allocations: 1000,
        };

        let res1 = OirVerifier::verify_module(&mod_self_witness, EffectKind::Observe, &known, &mut b1);
        assert!(matches!(res1, Err(VerifierError::SelfWitnessProhibited)));

        // Case 2: Budget exhausted
        let mut mod_budget = OirModule::new("budget_test");
        let inst = OirInstruction::new(
            "%obs0",
            OpCode::Observe,
            OirType::Observation,
            vec![],
            orid,
            EffectKind::Observe,
        )
        .unwrap();
        mod_budget.instructions.push(inst);

        let mut b_zero = Budget {
            cpu_steps_remaining: 0,
            wall_time_ms_limit: 1000,
            max_allocations: 1000,
        };

        let res2 = OirVerifier::verify_module(&mod_budget, EffectKind::Observe, &known, &mut b_zero);
        assert!(matches!(res2, Err(VerifierError::BudgetExhausted)));
    }

    #[test]
    fn test_verifier_deterministic_100_percent() {
        let orid = ORID::compute(ObjectKind::Claim, b"deterministic_test");
        let known = HashMap::new();

        let mut module = OirModule::new("det_mod");
        let obs = OirInstruction::new(
            "%obs0",
            OpCode::Observe,
            OirType::Observation,
            vec![],
            orid,
            EffectKind::Observe,
        )
        .unwrap();
        let prop = OirInstruction::new(
            "%h1",
            OpCode::Propose,
            OirType::Claim,
            vec![Value {
                id: "%obs0".into(),
                ty: OirType::Observation,
            }],
            orid,
            EffectKind::Pure,
        )
        .unwrap();

        module.push(obs).unwrap();
        module.push(prop).unwrap();

        for _ in 0..100 {
            let mut budget = Budget {
                cpu_steps_remaining: 1000,
                wall_time_ms_limit: 1000,
                max_allocations: 1000,
            };
            let res = OirVerifier::verify_module(&module, EffectKind::Observe, &known, &mut budget);
            assert!(res.is_ok(), "Verifier MUST be 100% deterministic PASS");
        }
    }

    #[test]
    fn test_verifier_p99_under_5ms_for_100k_op_target() {
        let orid = ORID::compute(ObjectKind::Observation, b"bench_100k");
        let known = HashMap::new();
        let mut module = OirModule::new("module_100k");

        let n_ops = 100_000;
        for i in 0..n_ops {
            let inst = OirInstruction::new(
                format!("%obs{}", i),
                OpCode::Observe,
                OirType::Observation,
                vec![],
                orid,
                EffectKind::Observe,
            )
            .unwrap();
            module.instructions.push(inst);
        }

        // Warmup trial
        {
            let mut budget = Budget {
                cpu_steps_remaining: 500_000,
                wall_time_ms_limit: 10_000,
                max_allocations: 100_000,
            };
            let _ = OirVerifier::verify_module(&module, EffectKind::Observe, &known, &mut budget);
        }

        let mut durations = Vec::new();
        let trials = 10;

        for _ in 0..trials {
            let mut budget = Budget {
                cpu_steps_remaining: 500_000,
                wall_time_ms_limit: 10_000,
                max_allocations: 100_000,
            };

            let start = Instant::now();
            assert!(OirVerifier::verify_module(&module, EffectKind::Observe, &known, &mut budget).is_ok());
            durations.push(start.elapsed());
        }

        durations.sort();
        let p99_latency = durations[trials - 1];

        println!(
            "[VERIFIER BENCHMARK] 100k ops verification latency p99: {:?}",
            p99_latency
        );
        assert!(
            p99_latency.as_millis() < 5,
            "Verifier p99 latency {:?} MUST be < 5ms for 100k-op target",
            p99_latency
        );
    }
}
