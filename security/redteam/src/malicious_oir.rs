// AUDIT-LENSES: Thompson, Guido, Berners-Lee
// INVARIANT: Red-team regression test verifying rejection of malicious OIR modules; 0 invalid SSA graphs accepted.

use origin_core::opcode::OpCode;
use origin_core::state::Budget;
use origin_core::{ObjectKind, ORID};
use origin_oir::ir::{EffectKind, OirInstruction, OirModule, OirType, Value};
use origin_oir::verify::{OirVerifier, VerifierError};
use std::collections::HashMap;

pub fn test_malicious_oir_rejection() -> bool {
    let orid = ORID::compute(ObjectKind::Claim, b"malicious_ssa");
    let mut module = OirModule::new("malicious_ssa_use");

    // Threat Scenario: Instruction references an undefined SSA register operand (%undefined_var)
    let inst = OirInstruction::new(
        "%c1",
        OpCode::Commit,
        OirType::Commitment,
        vec![Value {
            id: "%undefined_var".into(),
            ty: OirType::Observation,
        }],
        orid,
        EffectKind::Commit,
    )
    .unwrap();

    module.instructions.push(inst);

    let known = HashMap::new();
    let mut budget = Budget {
        cpu_steps_remaining: 1000,
        wall_time_ms_limit: 1000,
        max_allocations: 1000,
    };

    let verify_result = OirVerifier::verify_module(&module, EffectKind::Commit, &known, &mut budget);
    assert!(
        matches!(verify_result, Err(VerifierError::TypeError(_))),
        "OIR Verifier MUST reject malicious module with undefined SSA register uses"
    );

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redteam_test_malicious_oir() {
        assert!(test_malicious_oir_rejection());
    }
}
