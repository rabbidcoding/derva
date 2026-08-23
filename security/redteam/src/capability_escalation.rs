// AUDIT-LENSES: Thompson, Guido, Berners-Lee
// INVARIANT: Red-team regression test verifying lattice capability escalation denial; 0 privilege bypasses allowed.

use origin_core::opcode::OpCode;
use origin_core::{ObjectKind, ORID};
use origin_oir::effectcheck::{EffectError, OirEffectChecker};
use origin_oir::ir::{EffectKind, OirInstruction, OirModule, OirType, Value};
use std::collections::HashMap;

pub fn test_capability_escalation_rejection() -> bool {
    let orid = ORID::compute(ObjectKind::Claim, b"escalation_attack");
    let mut module = OirModule::new("escalation_attack");

    // Construct instruction attempting Commit under Pure effect context
    let inst = OirInstruction::new(
        "%c1",
        OpCode::Commit,
        OirType::Commitment,
        vec![Value {
            id: "%obs0".into(),
            ty: OirType::Observation,
        }],
        orid,
        EffectKind::Commit,
    )
    .unwrap();

    module.instructions.push(inst);

    let known = HashMap::new();
    let check_result = OirEffectChecker::check_module_effects(&module, EffectKind::Observe, &known);

    assert!(
        matches!(check_result, Err(EffectError::Escalation { .. })),
        "EffectChecker MUST reject module attempting Commit under Observe privilege capability"
    );

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redteam_test_capability_escalation() {
        assert!(test_capability_escalation_rejection());
    }
}
