# Changelog — ORIGIN-Ω ZERO

All notable changes to the **ORIGIN-Ω ZERO** post-frontier deterministic architecture will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.0.0-rc1] - 2026-08-23

### Added
- **P00 (Truth, Toolchain & GitHub Constitution)**: Formalized `zero_train_guard.py` enforcing `trainable_parameter_count == 0`, Rust toolchain setup, workspace layout, and CODEOWNERS.
- **P01 (Formal Semantics & Type System)**: Core domain-separated ORID hashing, state algebra, distinction spaces, and epistemic status lattice (`Unknown`, `Hypothesis`, `Supported`, `Verified`, `Contested`, `Refuted`).
- **P02 (Rust Authoritative Kernel & Immutable Store)**: Multi-version MVCC state store, append-only WAL, non-blocking snapshot isolation, and atomic commit protocol.
- **P03 (Evidence, Logic & Constraint Proof Engine)**: Deductive proof engine, constraint resolution, and evidence weighting.
- **P04 (E-Graph, Proof Search & Counterexample Engine)**: Equality saturation E-graph rewrite engine and counterexample generator.
- **P05 (Causal Operators, Counterfactuals & Planning)**: Causal DAG, Pearl do-calculus interventions, counterfactual reasoning, and causal planner.
- **P06 (JAX Numerical Coprocessor)**: Zero-training JAX/XLA FFI numerical coprocessor bindings.
- **P07 (OIR Intermediate Representation Engine)**: Origin Intermediate Representation (OIR) parser, SSA verifier, and effect checker.
- **P08 (Assembly, Micro-Op Acceleration & Kernel Engine)**: Portable fallback & AVX2 vectorization bitset kernels, packed SoA index scanner, fast artifact runtime executor, and slow deliberative runtime scheduler.
- **P09 (Post-Frontier Production, Security & Release)**:
  - Crash/power-loss recovery campaign (`origin-chaos`).
  - Fuzzing & Miri/Sanitizer integration (`fuzz/`, `tools/miri.sh`, `tools/sanitizers.sh`).
  - Adversarial red-team matrix (`security/redteam/`).
  - SPDX 2.3 SBOM, SHA256 checksums, and SLSA Level 3 build attestation pipeline (`tools/release_verify.sh`, `.github/workflows/release.yml`).
  - GitHub merge queue and production ruleset lock (`.github/ruleset.production.json`).
  - Observability & epistemic debugger CLI (`origin why/why-not/replay/profile` with versioned `schema_version: "1.0.0"` JSON).
  - 10M object / 50M relation long-horizon scale campaign (`bench/scale/`).
  - 4 baseline & 4 ablation matrix benchmark suite (`bench/baselines/`).
  - Release Candidate acceptance gate (`tools/gate_rc.py`, `reports/gates/RC.md`).

### Invariants Maintained
- `trainable_parameter_count == 0` (Exactly zero trainable parameters).
- $100\%$ fail-closed security rejection on forged ORIDs, orphaned provenance, or capability escalation.
- Bit-identical reproducibility across independent builds.
