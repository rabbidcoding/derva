# ORIGIN-Ω ZERO — OIR Core Intermediate Representation Specification (T071)

## Epistemic Constitution & IR Design

- **SSA Form**: OIR (ORIGIN Intermediate Representation) is an SSA-like typed intermediate representation designed for deterministic reasoning composition across the 9 micro-ISA opcodes.
- **100% Provenance Mapping**: Every OIR node/instruction MUST contain a valid `source_orid` mapping back to its spec or derivation origin.
- **Type-Opcode Matrix Invariant**: Invalid combinations of opcodes and result types are strictly rejected by `OirInstruction::validate()`.

---

## Type System & Micro-ISA OpCodes

| OpCode (1..9) | Result Type Identifier | Canonical Type String | Description |
| :--- | :--- | :--- | :--- |
| **Observe (1)** | `OirType::Observation` | `!origin.observation` | External/Internal state ingestion |
| **Propose (2)** | `OirType::Claim` | `!origin.claim` | Unverified hypothesis claim creation |
| **Relate (3)** | `OirType::Relation` | `!origin.relation` | Structural/Causal relationship binding |
| **Refine (4)** | `OirType::Refinement` | `!origin.refinement` | E-graph quotient/simplification refinement |
| **Query (5)** | `OirType::Query` | `!origin.query` | Epistemic active-information query |
| **Intervene (6)** | `OirType::Intervention` | `!origin.intervention` | Counterfactual intervention do(x) operator |
| **Verify (7)** | `OirType::VerifiedClaim` | `!origin.verified_claim` | Formally proven/verified claim |
| **Commit (8)** | `OirType::Commitment` | `!origin.commitment` | Authoritative transaction commit |
| **Compile (9)** | `OirType::CompiledArtifact` | `!origin.compiled_artifact` | AOT/StableHLO compiled target |

---

## Canonical Form Formats

### 1. Textual Representation Syntax
```mlir
module @reasoning_flow {
  %obs0 = observe() [source: orid:001122...] : !origin.observation
  %h1 = propose(%obs0) [source: orid:112233...] : !origin.claim
  %v2 = verify(%h1) [source: orid:223344...] : !origin.verified_claim
}
```

### 2. Binary Layout (`OIR1`)
```text
+-------------------+--------------------+-----------------------+
| Magic "OIR1" (4b) | Name Len + Name    | Inst Count u32        |
+-------------------+--------------------+-----------------------+
| OpCode (1b)       | Type (1b)          | Effect (1b)           |
+-------------------+--------------------+-----------------------+
| Result ID Len+Str | ORID Bytes (32b)   | Operand Count + Items |
+-------------------+--------------------+-----------------------+
```

---

## Audit Lenses

1. **Grace Hopper Lens**: IR must be machine-compilable, target-agnostic, and strictly typed.
2. **Ada Lovelace Lens**: Strict separation between operation (OpCode) and data representation (`OirType`).
3. **Dennis Ritchie Lens**: High-performance binary layout with minimal memory footprint mapping directly to machine representations.
