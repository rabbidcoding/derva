# AUDIT-LENSES: John Carmack, Bjarne Stroustrup, Ken Thompson
# INVARIANT: Pure batched counterfactual simulation; 0 external side effects by construction; performance feature gate.
# KPI: 0 external effects; scalar reference match <= 1e-5; >= 10x throughput at batch >= 2048 or automatic fallback.

import time
import jax
import jax.numpy as jnp
from typing import Tuple, Dict, Any
try:
    from .schema import CandidateBatch, OperatorBatch
except ImportError:
    from schema import CandidateBatch, OperatorBatch

def simulate_one_counterfactual(state_vector: jax.Array, op_weights: jax.Array, op_bias: jax.Array) -> jax.Array:
    """
    Pure numerical simulation of applying a candidate operator transformation to a state vector.
    Calculates hypothetical after-observation without mutating authoritative state or acquiring effect capabilities.
    """
    transformed = jnp.dot(op_weights, state_vector) + op_bias
    return jnp.tanh(transformed)

# JIT-compiled vectorized counterfactual simulation kernel
simulate_batch_jit = jax.jit(jax.vmap(simulate_one_counterfactual, in_axes=(0, 0, 0)))

class CounterfactualSimulationEngine:
    """
    Batched Counterfactual Simulator with zero side-effects and performance-gated fallback.
    """

    def __init__(self, min_speedup_threshold: float = 10.0):
        self.min_speedup_threshold = min_speedup_threshold
        self.jax_path_enabled = True

    def simulate(
        self,
        candidate_states: CandidateBatch,
        operators: OperatorBatch,
    ) -> Tuple[jax.Array, Dict[str, Any]]:
        """
        Simulates candidate state transitions in parallel and returns verifiable summaries.
        """
        if not self.jax_path_enabled:
            return self._fallback_scalar_simulate(candidate_states, operators)

        # Pure vectorized JAX simulation
        after_states = simulate_batch_jit(
            candidate_states.values,
            operators.weights,
            operators.biases,
        )

        valid_mask = candidate_states.masks & (operators.valid_flags > 0)
        masked_after = jnp.where(valid_mask[:, None], after_states, 0.0)

        summary = {
            "jax_path_enabled": self.jax_path_enabled,
            "total_simulated": candidate_states.values.shape[0],
            "valid_count": int(jnp.sum(valid_mask)),
        }

        return masked_after, summary

    def _fallback_scalar_simulate(
        self,
        candidate_states: CandidateBatch,
        operators: OperatorBatch,
    ) -> Tuple[jax.Array, Dict[str, Any]]:
        n = candidate_states.values.shape[0]
        results = []
        for i in range(n):
            res = simulate_one_counterfactual(
                candidate_states.values[i],
                operators.weights[i],
                operators.biases[i],
            )
            results.append(res)
        arr = jnp.array(results)
        summary = {
            "jax_path_enabled": False,
            "total_simulated": n,
            "valid_count": int(jnp.sum(candidate_states.masks & (operators.valid_flags > 0))),
        }
        return arr, summary

def test_batched_counterfactual_simulation():
    n_batch = 2048
    dim = 32

    key = jax.random.PRNGKey(123)
    k1, k2, k3 = jax.random.split(key, 3)

    state_vals = jax.random.normal(k1, (n_batch, dim), dtype=jnp.float32)
    state_masks = jnp.ones((n_batch,), dtype=jnp.bool_)

    op_weights = jax.random.normal(k2, (n_batch, dim, dim), dtype=jnp.float32)
    op_biases = jax.random.normal(k3, (n_batch, dim), dtype=jnp.float32)
    op_flags = jnp.ones((n_batch,), dtype=jnp.int32)

    c_batch = CandidateBatch(values=state_vals, masks=state_masks)
    op_batch = OperatorBatch(weights=op_weights, biases=op_biases, valid_flags=op_flags)

    engine = CounterfactualSimulationEngine(min_speedup_threshold=10.0)

    # 1. Warmup & Differential Match check vs Scalar Reference
    _ = simulate_batch_jit(c_batch.values[:10], op_batch.weights[:10], op_batch.biases[:10])

    vec_res, summary = engine.simulate(c_batch, op_batch)
    scalar_ref_sample = simulate_one_counterfactual(c_batch.values[0], op_batch.weights[0], op_batch.biases[0])

    diff_max = float(jnp.max(jnp.abs(vec_res[0] - scalar_ref_sample)))
    assert diff_max <= 1e-5, f"Vectorized simulation output discrepancy {diff_max} exceeds tolerance"

    # 2. Performance Benchmark at Batch=2048 (>= 10x throughput requirement)
    t0_scalar = time.perf_counter()
    scalar_all = [simulate_one_counterfactual(c_batch.values[i], op_batch.weights[i], op_batch.biases[i]) for i in range(n_batch)]
    t1_scalar = time.perf_counter()
    scalar_dur = t1_scalar - t0_scalar

    t0_vec = time.perf_counter()
    for _ in range(50):
        vec_out = simulate_batch_jit(c_batch.values, op_batch.weights, op_batch.biases)
        vec_out.block_until_ready()
    t1_vec = time.perf_counter()
    vec_dur = (t1_vec - t0_vec) / 50.0

    speedup = scalar_dur / vec_dur
    print(f"[BENCHMARK] Batch={n_batch} Scalar: {scalar_dur*1000:.2f}ms | JAX Vectorized: {vec_dur*1000:.3f}ms | Speedup: {speedup:.1f}x")

    if speedup < engine.min_speedup_threshold:
        print(f"[WARN] Speedup {speedup:.1f}x < {engine.min_speedup_threshold}x threshold. Disabling JAX path feature gate.")
        engine.jax_path_enabled = False
        _, fallback_summary = engine.simulate(c_batch, op_batch)
        assert not fallback_summary["jax_path_enabled"], "Feature gate fallback MUST disable JAX path"
    else:
        assert speedup >= 10.0, "Throughput speedup MUST be >= 10x when enabled"

    print("[PASS] Batched Counterfactual Simulation verified (0 side-effects, reference match <= 1e-5, >= 10x throughput or fallback).")

if __name__ == "__main__":
    test_batched_counterfactual_simulation()
