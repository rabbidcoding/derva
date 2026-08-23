# ORIGIN-Ω ZERO — System Limitations & Boundary Scope

## Architectural Scope & Epistemic Boundaries

> **DEFINITION OF DONE INVARIANT:** ZERO is explicitly designed for formalizable domains: mathematics, symbolic logic, causal worlds, knowledge graph revision, program synthesis, and agents with structured schemas. Any future perceptual or natural language frontend using learned weights MUST live outside the ZERO Trusted Computing Base (TCB) and CANNOT modify the epistemic authority of the Rust kernel.

---

## 1. Explicit Architectural Limitations

### 1.1 No General Human Perception
- ORIGIN-Ω ZERO contains **0 trainable parameters** (`trainable_parameter_count == 0`).
- The system does not attempt raw pixel, audio, or unstructured natural language perception internally. All inputs must be converted into canonical domain-separated ORID objects or structured observations before entering the kernel.

### 1.2 Non-Authoritative Coprocessor Boundary
- JAX/XLA and external numerical accelerators function strictly as pure, stateless coprocessors.
- Coprocessors cannot grant `Verified` epistemic status, authorize state commits, or modify causal DAG edges. Epistemic authority is exclusively held by the Rust TCB kernel (`origin-core`, `origin-store`, `origin-evidence`).

### 1.3 Finitude of Proof & Budget Limits
- Proof search in E-graph equality saturation and counterexample generation is bounded by explicit resource budgets (`time_budget_ms`, `max_nodes`).
- Undecidable or budget-exhausted queries fail-closed with status `Unknown` or typed `Stop::BudgetExhausted`, preventing unbounded computation or partial commits.

---

## 2. Kill Decisions & Scope Non-Goals

1. **No In-Kernel Neural Weight Training**: Deep neural network gradient updates within the TCB are hard-rejected by build-time and runtime AST/assembly scanners (`zero_train_guard.py`).
2. **No Unattested Execution**: Binaries without SLSA Level 3 provenance attestation and SPDX 2.3 SBOM are rejected by production rulesets (`.github/ruleset.production.json`).
3. **No Unverified Causal Promotion**: Claims cannot transition to `Verified` without deductively sound proof chains and non-conflicting observation graphs.
