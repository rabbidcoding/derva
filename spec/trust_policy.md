# Trust Policy Engine Specification

## Invariants & Principles

1. **Trust $\neq$ Truth**: High historical reliability (`trust = 1.0`) grants execution priority for evaluation, but CANNOT upgrade an evidence item to `Status::Verified` without explicit formal derivation and proof lineage.
2. **Orthogonal Dimensions**:
   - **Source Reliability**: Historical domain track record.
   - **Transport Integrity**: Cryptographic transit verification.
   - **Logical Correctness**: Formal verification status anchored in the immutable commit DAG.
3. **Immutable Policy Versioning**: Every policy change is content-addressed by ORID (`policy_root`). Evaluating evidence under different policy versions produces identical raw provenance graphs.
