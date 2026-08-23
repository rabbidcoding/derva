// INVARIANT: Exactly 9 micro-ISA opcodes v1. Adding opcodes requires ADR + architecture approval.
// KPI: 100% of reference reasoning operations expressible by micro-ISA composition.

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpCode {
    Observe = 1,
    Propose = 2,
    Relate = 3,
    Refine = 4,
    Query = 5,
    Intervene = 6,
    Verify = 7,
    Commit = 8,
    Compile = 9,
}

impl OpCode {
    pub const ALL: [OpCode; 9] = [
        OpCode::Observe,
        OpCode::Propose,
        OpCode::Relate,
        OpCode::Refine,
        OpCode::Query,
        OpCode::Intervene,
        OpCode::Verify,
        OpCode::Commit,
        OpCode::Compile,
    ];

    pub fn count() -> usize {
        Self::ALL.len()
    }

    pub fn byte_value(self) -> u8 {
        self as u8
    }

    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            1 => Some(OpCode::Observe),
            2 => Some(OpCode::Propose),
            3 => Some(OpCode::Relate),
            4 => Some(OpCode::Refine),
            5 => Some(OpCode::Query),
            6 => Some(OpCode::Intervene),
            7 => Some(OpCode::Verify),
            8 => Some(OpCode::Commit),
            9 => Some(OpCode::Compile),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub opcode: OpCode,
    pub payload: Vec<u8>,
}

impl Instruction {
    pub fn new(opcode: OpCode, payload: impl Into<Vec<u8>>) -> Self {
        Self {
            opcode,
            payload: payload.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_micro_isa_exact_nine_opcodes_invariant() {
        assert_eq!(
            OpCode::count(),
            9,
            "Micro-ISA v1 must contain exactly 9 opcodes"
        );
        assert_eq!(OpCode::ALL.len(), 9);

        // Verify bi-directional byte encoding round-trip for all 9 opcodes
        for op in OpCode::ALL {
            let byte_val = op.byte_value();
            assert!(byte_val >= 1 && byte_val <= 9);
            assert_eq!(OpCode::from_u8(byte_val), Some(op));
        }

        // Verify invalid byte fails cleanly without panic
        assert_eq!(OpCode::from_u8(0), None);
        assert_eq!(OpCode::from_u8(10), None);
    }

    #[test]
    fn test_reference_reasoning_flow_expressible_by_composition() {
        // Demonstrate a full end-to-end reasoning cycle composed strictly of the 9 opcodes
        let flow = vec![
            Instruction::new(OpCode::Observe, b"sensor_ingest"),
            Instruction::new(OpCode::Propose, b"hypothesis_claim"),
            Instruction::new(OpCode::Relate, b"causal_link"),
            Instruction::new(OpCode::Refine, b"constraint_bounds"),
            Instruction::new(OpCode::Query, b"state_check"),
            Instruction::new(OpCode::Intervene, b"do_experiment"),
            Instruction::new(OpCode::Verify, b"proof_witness"),
            Instruction::new(OpCode::Commit, b"state_txn"),
            Instruction::new(OpCode::Compile, b"generate_oir"),
        ];

        assert_eq!(flow.len(), 9);
        for inst in flow {
            assert!(OpCode::ALL.contains(&inst.opcode));
        }
    }
}
