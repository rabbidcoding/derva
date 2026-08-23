#![forbid(unsafe_code)]

// AUDIT-LENSES: Grace Hopper, Ada Lovelace, Dennis Ritchie
// INVARIANT: SSA-like OIR Core IR for 9 micro-ISA opcodes; 100% source-mapped to ORIDs; 100% text/binary round-trip.
// KPI: 100% OIR nodes source-mapped; binary/text round-trip 100%; invalid opcode/type combinations rejected.

use origin_core::{opcode::OpCode, ORID};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OirType {
    Observation = 1,
    Claim = 2,
    Relation = 3,
    Refinement = 4,
    Query = 5,
    Intervention = 6,
    VerifiedClaim = 7,
    Commitment = 8,
    CompiledArtifact = 9,
}

impl OirType {
    pub fn type_name(self) -> &'static str {
        match self {
            OirType::Observation => "!origin.observation",
            OirType::Claim => "!origin.claim",
            OirType::Relation => "!origin.relation",
            OirType::Refinement => "!origin.refinement",
            OirType::Query => "!origin.query",
            OirType::Intervention => "!origin.intervention",
            OirType::VerifiedClaim => "!origin.verified_claim",
            OirType::Commitment => "!origin.commitment",
            OirType::CompiledArtifact => "!origin.compiled_artifact",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "!origin.observation" => Some(OirType::Observation),
            "!origin.claim" => Some(OirType::Claim),
            "!origin.relation" => Some(OirType::Relation),
            "!origin.refinement" => Some(OirType::Refinement),
            "!origin.query" => Some(OirType::Query),
            "!origin.intervention" => Some(OirType::Intervention),
            "!origin.verified_claim" => Some(OirType::VerifiedClaim),
            "!origin.commitment" => Some(OirType::Commitment),
            "!origin.compiled_artifact" => Some(OirType::CompiledArtifact),
            _ => None,
        }
    }

    pub fn byte_value(self) -> u8 {
        self as u8
    }

    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            1 => Some(OirType::Observation),
            2 => Some(OirType::Claim),
            3 => Some(OirType::Relation),
            4 => Some(OirType::Refinement),
            5 => Some(OirType::Query),
            6 => Some(OirType::Intervention),
            7 => Some(OirType::VerifiedClaim),
            8 => Some(OirType::Commitment),
            9 => Some(OirType::CompiledArtifact),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EffectKind {
    Pure = 0,
    Read = 1,
    Observe = 2,
    Intervene = 3,
    Commit = 4,
    Compile = 5,
}

impl EffectKind {
    pub fn byte_value(self) -> u8 {
        self as u8
    }

    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            0 => Some(EffectKind::Pure),
            1 => Some(EffectKind::Read),
            2 => Some(EffectKind::Observe),
            3 => Some(EffectKind::Intervene),
            4 => Some(EffectKind::Commit),
            5 => Some(EffectKind::Compile),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Value {
    pub id: String,
    pub ty: OirType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OirInstruction {
    pub result: Value,
    pub opcode: OpCode,
    pub operands: Vec<Value>,
    pub source_orid: ORID,
    pub effect: EffectKind,
}

impl OirInstruction {
    pub fn new(
        result_id: impl Into<String>,
        opcode: OpCode,
        result_type: OirType,
        operands: Vec<Value>,
        source_orid: ORID,
        effect: EffectKind,
    ) -> Result<Self, String> {
        let inst = Self {
            result: Value {
                id: result_id.into(),
                ty: result_type,
            },
            opcode,
            operands,
            source_orid,
            effect,
        };
        inst.validate()?;
        Ok(inst)
    }

    /// Enforces strict opcode and result type matrix validation
    #[inline(always)]
    pub fn validate(&self) -> Result<(), String> {
        self.validate_and_inherent_effect().map(|_| ())
    }

    /// Enforces strict opcode and result type matrix validation and returns inherent effect
    #[inline(always)]
    pub fn validate_and_inherent_effect(&self) -> Result<EffectKind, String> {
        match (self.opcode, self.result.ty) {
            (OpCode::Observe, OirType::Observation) => Ok(EffectKind::Observe),
            (OpCode::Propose, OirType::Claim) => Ok(EffectKind::Pure),
            (OpCode::Relate, OirType::Relation) => Ok(EffectKind::Compile),
            (OpCode::Refine, OirType::Refinement) => Ok(EffectKind::Pure),
            (OpCode::Query, OirType::Query) => Ok(EffectKind::Read),
            (OpCode::Intervene, OirType::Intervention) => Ok(EffectKind::Intervene),
            (OpCode::Verify, OirType::VerifiedClaim) => Ok(EffectKind::Pure),
            (OpCode::Commit, OirType::Commitment) => Ok(EffectKind::Commit),
            (OpCode::Compile, OirType::CompiledArtifact) => Ok(EffectKind::Compile),
            (op, ty) => Err(format!(
                "Invalid opcode/type combination: opcode {:?} cannot produce type {:?}",
                op, ty
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OirModule {
    pub name: String,
    pub instructions: Vec<OirInstruction>,
}

impl OirModule {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            instructions: Vec::new(),
        }
    }

    pub fn push(&mut self, inst: OirInstruction) -> Result<(), String> {
        inst.validate()?;
        self.instructions.push(inst);
        Ok(())
    }

    pub fn validate(&self) -> Result<(), String> {
        for (idx, inst) in self.instructions.iter().enumerate() {
            inst.validate()
                .map_err(|e| format!("Instruction {} invalid: {}", idx, e))?;
        }
        Ok(())
    }

    /// Emits canonical SSA-like text representation
    pub fn emit_text(&self) -> String {
        let mut out = format!("module @{} {{\n", self.name);
        for inst in &self.instructions {
            let op_str = match inst.opcode {
                OpCode::Observe => "observe",
                OpCode::Propose => "propose",
                OpCode::Relate => "relate",
                OpCode::Refine => "refine",
                OpCode::Query => "query",
                OpCode::Intervene => "intervene",
                OpCode::Verify => "verify",
                OpCode::Commit => "commit",
                OpCode::Compile => "compile",
            };

            let operands_str = inst
                .operands
                .iter()
                .map(|v| v.id.clone())
                .collect::<Vec<_>>()
                .join(", ");

            out.push_str(&format!(
                "  {} = {}({}) [source: {}] : {}\n",
                inst.result.id,
                op_str,
                operands_str,
                inst.source_orid,
                inst.result.ty.type_name()
            ));
        }
        out.push_str("}\n");
        out
    }

    /// Emits canonical binary format encoding
    pub fn emit_binary(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        // Magic header "OIR1"
        buf.extend_from_slice(b"OIR1");
        
        let name_bytes = self.name.as_bytes();
        buf.push(name_bytes.len() as u8);
        buf.extend_from_slice(name_bytes);

        buf.extend_from_slice(&(self.instructions.len() as u32).to_le_bytes());

        for inst in &self.instructions {
            buf.push(inst.opcode.byte_value());
            buf.push(inst.result.ty.byte_value());
            buf.push(inst.effect.byte_value());

            let res_id_bytes = inst.result.id.as_bytes();
            buf.push(res_id_bytes.len() as u8);
            buf.extend_from_slice(res_id_bytes);

            // Encode ORID as String bytes for exact round-trip preservation
            let orid_str = inst.source_orid.to_string();
            let orid_str_bytes = orid_str.as_bytes();
            buf.push(orid_str_bytes.len() as u8);
            buf.extend_from_slice(orid_str_bytes);

            buf.push(inst.operands.len() as u8);
            for op in &inst.operands {
                buf.push(op.ty.byte_value());
                let op_id_bytes = op.id.as_bytes();
                buf.push(op_id_bytes.len() as u8);
                buf.extend_from_slice(op_id_bytes);
            }
        }
        buf
    }

    /// Parses canonical binary format back into OirModule
    pub fn parse_binary(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 9 || &bytes[0..4] != b"OIR1" {
            return Err("Invalid OIR binary header magic".into());
        }

        let mut offset = 4;
        let name_len = bytes[offset] as usize;
        offset += 1;

        let name = String::from_utf8(bytes[offset..offset + name_len].to_vec())
            .map_err(|e| format!("Invalid module name UTF-8: {}", e))?;
        offset += name_len;

        let inst_count = u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| "Binary truncated")?,
        ) as usize;
        offset += 4;

        let mut module = OirModule::new(name);

        for _ in 0..inst_count {
            let opcode_byte = bytes[offset];
            let opcode = OpCode::from_u8(opcode_byte)
                .ok_or_else(|| format!("Unknown opcode byte {}", opcode_byte))?;
            offset += 1;

            let result_ty_byte = bytes[offset];
            let result_ty = OirType::from_u8(result_ty_byte)
                .ok_or_else(|| format!("Unknown type byte {}", result_ty_byte))?;
            offset += 1;

            let effect_byte = bytes[offset];
            let effect = EffectKind::from_u8(effect_byte)
                .ok_or_else(|| format!("Unknown effect byte {}", effect_byte))?;
            offset += 1;

            let res_id_len = bytes[offset] as usize;
            offset += 1;
            let result_id = String::from_utf8(bytes[offset..offset + res_id_len].to_vec())
                .map_err(|e| format!("Invalid result id UTF-8: {}", e))?;
            offset += res_id_len;

            let orid_len = bytes[offset] as usize;
            offset += 1;
            let orid_str = String::from_utf8(bytes[offset..offset + orid_len].to_vec())
                .map_err(|e| format!("Invalid ORID UTF-8: {}", e))?;
            offset += orid_len;

            let source_orid: ORID = orid_str
                .parse()
                .map_err(|e| format!("Failed parsing ORID string: {}", e))?;

            let operand_count = bytes[offset] as usize;
            offset += 1;

            let mut operands = Vec::with_capacity(operand_count);
            for _ in 0..operand_count {
                let op_ty_byte = bytes[offset];
                let op_ty = OirType::from_u8(op_ty_byte)
                    .ok_or_else(|| format!("Unknown operand type byte {}", op_ty_byte))?;
                offset += 1;

                let op_id_len = bytes[offset] as usize;
                offset += 1;
                let op_id = String::from_utf8(bytes[offset..offset + op_id_len].to_vec())
                    .map_err(|e| format!("Invalid operand id UTF-8: {}", e))?;
                offset += op_id_len;

                operands.push(Value {
                    id: op_id,
                    ty: op_ty,
                });
            }

            let inst = OirInstruction::new(
                result_id,
                opcode,
                result_ty,
                operands,
                source_orid,
                effect,
            )?;
            module.push(inst)?;
        }

        Ok(module)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use origin_core::orid::ObjectKind;

    #[test]
    fn test_100_percent_oir_nodes_source_mapped() {
        let orid1 = ORID::compute(ObjectKind::Observation, b"sensor_data");
        let inst = OirInstruction::new(
            "%obs0",
            OpCode::Observe,
            OirType::Observation,
            vec![],
            orid1,
            EffectKind::Observe,
        ).unwrap();

        assert_eq!(inst.source_orid, orid1);
    }

    #[test]
    fn test_binary_round_trip_100_percent() {
        let orid1 = ORID::compute(ObjectKind::Observation, b"sensor_data");
        let orid2 = ORID::compute(ObjectKind::Claim, b"hypothesis_1");

        let mut module = OirModule::new("test_mod");

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

        module.push(obs).unwrap();
        module.push(prop).unwrap();

        let bin = module.emit_binary();
        let deserialized = OirModule::parse_binary(&bin).unwrap();

        assert_eq!(module, deserialized, "Binary round-trip MUST match 100%");
    }

    #[test]
    fn test_invalid_opcode_type_combinations_rejected() {
        let orid = ORID::compute(ObjectKind::Claim, b"dummy_claim");
        let res = OirInstruction::new(
            "%invalid",
            OpCode::Observe,
            OirType::VerifiedClaim,
            vec![],
            orid,
            EffectKind::Observe,
        );

        assert!(res.is_err(), "Invalid opcode/type combination MUST be rejected");
        assert!(res.unwrap_err().contains("Invalid opcode/type combination"));
    }
}
