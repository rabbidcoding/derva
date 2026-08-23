# ORIGIN-Ω ZERO — Observability & Epistemic Debugger Manual

## Epistemic Constitution & Observability Principles

- **Audit Lenses**: **Steve Jobs**, **Donald Knuth**, **Guido van Rossum**, **Bill Gates**
- **System Invariants**:
  - `100% VERIFIED claims explainable to roots`
  - `Debugger query p99 < 100ms on 1M-edge graph`
  - `JSON schema stable and versioned ("1.0.0")`

---

## CLI Command Interface

### 1. `origin why <ORID> [--json]`
Traces a verified claim back to its constituent evidence nodes, proof engine steps, and epistemic score.

```bash
origin why orid:claim:8f3c7a --json
```

### 2. `origin why-not <ORID> [--json]`
Explains why a claim remains unverified or contested, listing missing evidence observations or unsatisfied obligations.

```bash
origin why-not orid:claim:8f3c7a --json
```

### 3. `origin replay <COMMIT_ROOT> [--verify] [--json]`
Deterministically replays state evolution from a commit root and validates bit-level parity.

```bash
origin replay orid:commit:root01 --verify --json
```

### 4. `origin profile [--json]`
Reports micro-op execution metrics, SIMD acceleration speedups, and fast/slow runtime hit rates.

```bash
origin profile --json
```

---

## Stable Versioned JSON Schema Specification (`schema_version: "1.0.0"`)

All commands with `--json` return a standardized root wrapper:

```json
{
  "schema_version": "1.0.0",
  "command": "why",
  "target_orid": "orid:claim:8f3c7a",
  "latency_us": 42,
  "data": { ... }
}
```
