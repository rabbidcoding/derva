# ORIGIN-Ω ZERO — JAX Static PyTree Shapes & Dtypes Specification (T062)

## PyTree Schemas & Shape Invariants

To eliminate accidental XLA JIT recompilations and enforce strict memory layout bounds, all data entering the JAX numerical coprocessor must be packaged into frozen, registered PyTree dataclasses with static shapes and explicit dtypes.

---

## Registered PyTree Schemas

### 1. `CandidateBatch`
- **`values`**: `jax.Array` with shape `[N, D]` and dtype `float32`.
- **`masks`**: `jax.Array` with shape `[N]` and dtype `bool_`.
- **Use Case**: Batch evaluation of state candidates in search and planning space.

### 2. `IntervalBatch`
- **`lower_bounds`**: `jax.Array` with shape `[N, D]` and dtype `float32`.
- **`upper_bounds`**: `jax.Array` with shape `[N, D]` and dtype `float32`.
- **`active_mask`**: `jax.Array` with shape `[N]` and dtype `bool_`.
- **Use Case**: Bounded interval propagation and spatial region verification.

### 3. `OperatorBatch`
- **`weights`**: `jax.Array` with shape `[M, D]` and dtype `float32`.
- **`biases`**: `jax.Array` with shape `[M]` and dtype `float32`.
- **`valid_flags`**: `jax.Array` with shape `[M]` and dtype `int32`.
- **Use Case**: Parallel evaluation of vector transformation operators.

---

## Schema Hash & Compiled Artifact Metadata

Each PyTree schema computes a deterministic SHA256 `schema_hash` from its structural spec:
```text
CandidateBatch:values=[N,D]:float32;masks=[N]:bool
```
The `schema_hash` is recorded in `CompiledArtifactMetadata` to ensure schema identity across compilation caches and execution traces.

---

## Audit Lens Governance

1. **Bjarne Stroustrup Lens**: Zero hidden runtime overhead. Types and shapes are explicit at compile time.
2. **John Carmack Lens**: Fixed memory buffers, zero dynamic allocations, and steady-state recompile rate $< 1\%$.
3. **Guido van Rossum Lens**: Clean Python data contracts using `@jax.tree_util.register_dataclass`.
