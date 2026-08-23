# ORIGIN-Ω ZERO — Numerical Artifact & StableHLO Export Specification (T068)

## Epistemic Constitution & Export Governance

- **Attested Compilation**: Compiling mature JAX numerical kernels into AOT / StableHLO bytecode (`.hlo`) decouples evaluation from dynamic Python execution while preserving absolute numerical reproducibility.
- **Mandatory Manifest**: 100% of exported numerical artifacts must be accompanied by an immutable JSON manifest containing:
  - `kernel_name`: Human-readable identifier.
  - `schema_hash`: Deterministic SHA256 of input/output shape and dtype PyTree contract.
  - `source_orid`: Content-addressed ORID of the originating logic/kernel entity.
  - `source_hash`: SHA256 hash of the compiled HLO representation.
  - `jax_version`: Exact JAX release version used for AOT compilation.
  - `backend`: Target XLA execution backend (`cpu`, `gpu`, `tpu`).
- **Zero-Stale-Schema Rejection**: Runtime loaders must inspect `schema_hash` prior to artifact deserialization. Any mismatch or stale version instantly aborts load with a hard `ValueError`.

---

## Technical Manifest Contract (`python/origin_jax/export.py`)

```json
{
  "kernel_name": "hypothesis_scoring_v1",
  "schema_hash": "sha256:7f8a9b0c...",
  "source_orid": "orid:00112233445566778899aabbccddeeff",
  "source_hash": "sha256:1a2b3c4d...",
  "jax_version": "0.4.30",
  "backend": "cpu"
}
```

---

## Audit Lens Governance

1. **Grace Hopper Lens**: Compile stable work into certified, repeatable binaries. Never recompile what is already known and proven.
2. **Ken Thompson Lens**: Trust through attestation. Verify exact source hashes and ORID provenance before loading code.
3. **Tim Berners-Lee Lens**: Unique global identity. Every kernel and schema manifest possesses explicit URI/ORID identification.
