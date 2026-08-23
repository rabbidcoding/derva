# ORIGIN-Ω ZERO — Security Architecture & Threat Model

## Epistemic Constitution & Governance Summary

- **Security Model**: Strict Capability-Based Access & Immutable Content-Addressed Provenance
- **System Invariant**: `trainable_parameter_count == 0`
- **Audit Lenses**: **Ken Thompson**, **Guido van Rossum**, **Tim Berners-Lee**

---

## Security Boundaries & Isolation Zones

```
+-----------------------------------------------------------------------------------+
|                            ORIGIN-Ω SECURITY ARCHITECTURE                         |
+-----------------------------------------------------------------------------------+
|                                                                                   |
|  [ TRUSTED COMPUTING BASE (TCB) ]                                                 |
|  - Rust Authoritative Kernel (origin-core, origin-store, origin-kernel)          |
|  - Full ownership of State Algebra S = (G, C, E, U, O, B, Z)                      |
|  - Enforces #![forbid(unsafe_code)] at crate level                               |
|                                                                                   |
+------------------------------------------+----------------------------------------+
                                           |
                    Failsafe Barrier       | Strict Sandbox Boundaries
                                           v
+------------------------------------------+----------------------------------------+
|  [ UNTRUSTED COMPONENT ZONES ]                                                    |
|  - JAX Numerical Coprocessor (origin_jax) — Pure math, zero privilege             |
|  - Compiled Fast Artifacts (origin-fast)  — Boundary guarded by DomainGuard       |
|  - External Data & Observations          — Treated strictly as inert values       |
+-----------------------------------------------------------------------------------+
```

---

## Threat Matrix & Adversarial Vectors

| Threat ID | Adversarial Vector | Attack Mechanism | Defense & Mitigation Protocol | Status |
| :--- | :--- | :--- | :--- | :--- |
| **THREAT-01** | **Forged ORIDs** | Attempting to forge ORID hashes to impersonate valid claims or commits. | Domain-separated SHA256 hashing; exact payload re-verification on lookup. | **MITIGATED** |
| **THREAT-02** | **Provenance Laundering** | Injecting orphan claims without valid parent commit ancestry. | Iterative topological DAG ancestor validation (`replay_ancestor_sequence`). | **MITIGATED** |
| **THREAT-03** | **Capability Escalation** | Executing `Intervene` or `Commit` under `Pure` effect context. | Lattice effect checking (`Pure` < `Observe` < `Read` < `Intervene` < `Commit`). | **MITIGATED** |
| **THREAT-04** | **Data-Instruction Confusion** | Injecting executable OIR code into raw data payload observations. | Strict separation of data values (`Value::String`) from OIR opcodes (`Instruction`). | **MITIGATED** |
| **THREAT-05** | **Stale Artifact Execution** | Re-executing compiled fast artifacts after schema or dependency root mutation. | Pre-execution `validate_and_acquire` enforcing dependency root ORID match. | **MITIGATED** |
| **THREAT-06** | **Malicious OIR Modules** | Injecting invalid SSA graph cycles or effect barrier evasions into compiler IR. | Two-phase OIR verifier checking SSA dominance, single-assignment, and barriers. | **MITIGATED** |

---

## Red-Team Verification Matrix

All 6 threat vectors are continuously fuzz-tested and validated by automated red-team regression suites in `security/redteam/`.
