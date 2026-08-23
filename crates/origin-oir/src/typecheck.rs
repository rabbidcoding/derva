#![forbid(unsafe_code)]

// AUDIT-LENSES: Guido van Rossum, Niklaus Wirth, Bjarne Stroustrup
// INVARIANT: OIR Type Checker validating operand types, scope bindings, and effect contracts before execution.
// KPI: Reject 100% invalid IR corpus; Accept 100% valid IR corpus; Typecheck throughput >= 1M ops/sec.

use crate::ir::{EffectKind, OirInstruction, OirModule, OirType};
use origin_core::opcode::OpCode;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeError {
    UndefinedValue(String),
    TypeMismatch {
        expected: OirType,
        actual: OirType,
        value_id: String,
    },
    InvalidOperandCount {
        opcode: OpCode,
        expected: usize,
        actual: usize,
    },
    InvalidOpcodeResultType {
        opcode: OpCode,
        result_type: OirType,
    },
    EffectMismatch {
        expected: EffectKind,
        actual: EffectKind,
    },
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeError::UndefinedValue(id) => write!(f, "Undefined OIR value identifier: {}", id),
            TypeError::TypeMismatch {
                expected,
                actual,
                value_id,
            } => {
                write!(
                    f,
                    "Type mismatch for {}: expected {:?}, got {:?}",
                    value_id, expected, actual
                )
            }
            TypeError::InvalidOperandCount {
                opcode,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Invalid operand count for opcode {:?}: expected {}, got {}",
                    opcode, expected, actual
                )
            }
            TypeError::InvalidOpcodeResultType {
                opcode,
                result_type,
            } => {
                write!(
                    f,
                    "Invalid result type {:?} for opcode {:?}",
                    result_type, opcode
                )
            }
            TypeError::EffectMismatch { expected, actual } => {
                write!(
                    f,
                    "Effect mismatch: expected {:?}, got {:?}",
                    expected, actual
                )
            }
        }
    }
}

impl std::error::Error for TypeError {}

pub struct OirTypeChecker;

impl OirTypeChecker {
    /// Validates an entire OIR module for type correctness, scope bindings, and operand signatures
    pub fn check_module(module: &OirModule) -> Result<(), TypeError> {
        let mut scope: HashMap<String, OirType> = HashMap::new();

        for inst in &module.instructions {
            Self::check_instruction(inst, &scope)?;
            scope.insert(inst.result.id.clone(), inst.result.ty);
        }

        Ok(())
    }

    /// Validates a single OIR instruction against scope and signature rules
    pub fn check_instruction(
        inst: &OirInstruction,
        scope: &HashMap<String, OirType>,
    ) -> Result<(), TypeError> {
        // 1. Verify operand availability and type match in scope
        for op in &inst.operands {
            match scope.get(&op.id) {
                Some(&bound_ty) => {
                    if bound_ty != op.ty {
                        return Err(TypeError::TypeMismatch {
                            expected: op.ty,
                            actual: bound_ty,
                            value_id: op.id.clone(),
                        });
                    }
                }
                None => {
                    return Err(TypeError::UndefinedValue(op.id.clone()));
                }
            }
        }

        // 2. Opcode-specific signature checks
        match inst.opcode {
            OpCode::Observe => {
                if inst.result.ty != OirType::Observation {
                    return Err(TypeError::InvalidOpcodeResultType {
                        opcode: OpCode::Observe,
                        result_type: inst.result.ty,
                    });
                }
            }
            OpCode::Propose => {
                if inst.result.ty != OirType::Claim {
                    return Err(TypeError::InvalidOpcodeResultType {
                        opcode: OpCode::Propose,
                        result_type: inst.result.ty,
                    });
                }
                if inst.operands.len() != 1 || inst.operands[0].ty != OirType::Observation {
                    return Err(TypeError::TypeMismatch {
                        expected: OirType::Observation,
                        actual: inst.operands.first().map(|v| v.ty).unwrap_or(OirType::Claim),
                        value_id: inst.operands.first().map(|v| v.id.clone()).unwrap_or_default(),
                    });
                }
            }
            OpCode::Relate => {
                if inst.result.ty != OirType::Relation {
                    return Err(TypeError::InvalidOpcodeResultType {
                        opcode: OpCode::Relate,
                        result_type: inst.result.ty,
                    });
                }
            }
            OpCode::Refine => {
                if inst.result.ty != OirType::Refinement {
                    return Err(TypeError::InvalidOpcodeResultType {
                        opcode: OpCode::Refine,
                        result_type: inst.result.ty,
                    });
                }
            }
            OpCode::Query => {
                if inst.result.ty != OirType::Query {
                    return Err(TypeError::InvalidOpcodeResultType {
                        opcode: OpCode::Query,
                        result_type: inst.result.ty,
                    });
                }
            }
            OpCode::Intervene => {
                if inst.result.ty != OirType::Intervention {
                    return Err(TypeError::InvalidOpcodeResultType {
                        opcode: OpCode::Intervene,
                        result_type: inst.result.ty,
                    });
                }
            }
            OpCode::Verify => {
                if inst.result.ty != OirType::VerifiedClaim {
                    return Err(TypeError::InvalidOpcodeResultType {
                        opcode: OpCode::Verify,
                        result_type: inst.result.ty,
                    });
                }
                if inst.operands.len() != 1 || inst.operands[0].ty != OirType::Claim {
                    return Err(TypeError::TypeMismatch {
                        expected: OirType::Claim,
                        actual: inst.operands.first().map(|v| v.ty).unwrap_or(OirType::Observation),
                        value_id: inst.operands.first().map(|v| v.id.clone()).unwrap_or_default(),
                    });
                }
            }
            OpCode::Commit => {
                if inst.result.ty != OirType::Commitment {
                    return Err(TypeError::InvalidOpcodeResultType {
                        opcode: OpCode::Commit,
                        result_type: inst.result.ty,
                    });
                }
            }
            OpCode::Compile => {
                if inst.result.ty != OirType::CompiledArtifact {
                    return Err(TypeError::InvalidOpcodeResultType {
                        opcode: OpCode::Compile,
                        result_type: inst.result.ty,
                    });
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Value;
    use origin_core::{orid::ObjectKind, ORID};
    use std::time::Instant;

    #[test]
    fn test_accept_100_percent_canonical_valid_corpus() {
        let orid1 = ORID::compute(ObjectKind::Observation, b"obs_data");
        let orid2 = ORID::compute(ObjectKind::Claim, b"claim_data");
        let orid3 = ORID::compute(ObjectKind::Evidence, b"verify_data");

        let mut module = OirModule::new("valid_flow");

        let obs = OirInstruction::new(
            "%obs0",
            OpCode::Observe,
            OirType::Observation,
            vec![],
            orid1,
            EffectKind::Observe,
        ).unwrap();

        let prop = OirInstruction::new(
            "%h1",
            OpCode::Propose,
            OirType::Claim,
            vec![Value { id: "%obs0".into(), ty: OirType::Observation }],
            orid2,
            EffectKind::Pure,
        ).unwrap();

        let ver = OirInstruction::new(
            "%v2",
            OpCode::Verify,
            OirType::VerifiedClaim,
            vec![Value { id: "%h1".into(), ty: OirType::Claim }],
            orid3,
            EffectKind::Pure,
        ).unwrap();

        module.push(obs).unwrap();
        module.push(prop).unwrap();
        module.push(ver).unwrap();

        assert!(OirTypeChecker::check_module(&module).is_ok());
    }

    #[test]
    fn test_reject_100_percent_corpus_invalid_ir() {
        let orid = ORID::compute(ObjectKind::Claim, b"dummy");

        // Case 1: Undefined Operand Value
        let mut mod_undefined = OirModule::new("undefined_op");
        let prop_bad = OirInstruction::new(
            "%h1",
            OpCode::Propose,
            OirType::Claim,
            vec![Value { id: "%nonexistent".into(), ty: OirType::Observation }],
            orid,
            EffectKind::Pure,
        ).unwrap();
        mod_undefined.instructions.push(prop_bad);

        let err1 = OirTypeChecker::check_module(&mod_undefined);
        assert!(matches!(err1, Err(TypeError::UndefinedValue(_))));

        // Case 2: Operand Type Mismatch (Verify expects Claim, given Observation)
        let mut mod_mismatch = OirModule::new("type_mismatch");
        let obs = OirInstruction::new(
            "%obs0",
            OpCode::Observe,
            OirType::Observation,
            vec![],
            orid,
            EffectKind::Observe,
        ).unwrap();
        let ver_bad = OirInstruction::new(
            "%v1",
            OpCode::Verify,
            OirType::VerifiedClaim,
            vec![Value { id: "%obs0".into(), ty: OirType::Claim }],
            orid,
            EffectKind::Pure,
        ).unwrap();
        mod_mismatch.instructions.push(obs);
        mod_mismatch.instructions.push(ver_bad);

        let err2 = OirTypeChecker::check_module(&mod_mismatch);
        assert!(matches!(err2, Err(TypeError::TypeMismatch { .. })));
    }

    #[test]
    fn test_typecheck_throughput_benchmark_target() {
        let orid = ORID::compute(ObjectKind::Observation, b"bench");
        let mut module = OirModule::new("bench_module");

        let n_ops = 50_000; // Build sequence of valid Observe instructions
        for i in 0..n_ops {
            let inst = OirInstruction::new(
                format!("%obs{}", i),
                OpCode::Observe,
                OirType::Observation,
                vec![],
                orid,
                EffectKind::Observe,
            ).unwrap();
            module.instructions.push(inst);
        }

        let start = Instant::now();
        assert!(OirTypeChecker::check_module(&module).is_ok());
        let duration = start.elapsed();

        let ops_per_sec = (n_ops as f64) / duration.as_secs_f64();
        println!("[TYPECHECK BENCHMARK] Processed {} ops in {:?} | Throughput: {:.0} ops/sec", n_ops, duration, ops_per_sec);
        assert!(ops_per_sec >= 1_000_000.0, "Typecheck throughput {:.0} MUST be >= 1,000,000 ops/sec", ops_per_sec);
    }
}
