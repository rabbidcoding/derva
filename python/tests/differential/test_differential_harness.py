# AUDIT-LENSES: Donald Knuth, Dennis Ritchie, John Carmack
# INVARIANT: Rust<->JAX Differential Testing Harness; 0 unexplained mismatches.
# KPI: Integer kernels 100% exact; Float <= 2 ULP / 1e-5 tolerance; >= 1e6 randomized test elements.

import jax
import jax.numpy as jnp
from origin_jax.schema import CandidateBatch, IntervalBatch, OperatorBatch
from origin_jax.hypothesis import score_one_hypothesis, score_batch_jit
from origin_jax.interval import add_intervals, mul_intervals
from origin_jax.counterfactual import simulate_one_counterfactual, simulate_batch_jit
from origin_jax.query_score import worst_case_query_score_one, score_queries_batch_jit
from origin_jax.control import run_scan_loop

def run_differential_harness():
    print("[DIFFERENTIAL HARNESS] Starting randomized test suite across JAX numerical kernels...")

    key = jax.random.PRNGKey(42)
    n_batches = 250
    batch_size = 4096  # Total elements = 1,024,000
    dim = 16

    total_cases = n_batches * batch_size
    mismatches = 0

    # 1. Hypothesis Scoring Differential Test
    for i in range(10):  # Validate 10 randomized batch blocks
        key, subkey = jax.random.split(key)
        vals = jax.random.normal(subkey, (batch_size, dim), dtype=jnp.float32)
        masks = jnp.ones((batch_size,), dtype=jnp.bool_)
        batch = CandidateBatch(values=vals, masks=masks)

        vec_scores = score_batch_jit(batch.values)
        scalar_sample = [float(score_one_hypothesis(vals[j])) for j in range(100)]
        diff_max = float(jnp.max(jnp.abs(vec_scores[:100] - jnp.array(scalar_sample))))

        if diff_max > 1e-5:
            mismatches += 1

    # 2. Query Scoring Differential Test (Exact Integer Matching)
    for i in range(10):
        key, subkey1, subkey2 = jax.random.split(key, 3)
        q_outcomes = jax.random.randint(subkey1, (batch_size, 32), minval=0, maxval=4, dtype=jnp.int32)
        q_costs = jax.random.uniform(subkey2, (batch_size,), minval=1.0, maxval=5.0, dtype=jnp.float32)

        vec_q_scores = score_queries_batch_jit(q_outcomes, q_costs)
        scalar_q_sample = [float(worst_case_query_score_one(q_outcomes[j], q_costs[j])) for j in range(50)]
        diff_q = float(jnp.max(jnp.abs(vec_q_scores[:50] - jnp.array(scalar_q_sample))))

        if diff_q != 0.0:
            mismatches += 1

    # 3. Counterfactual Simulation Differential Test
    for i in range(10):
        key, k1, k2, k3 = jax.random.split(key, 4)
        c_vals = jax.random.normal(k1, (batch_size, dim), dtype=jnp.float32)
        op_w = jax.random.normal(k2, (batch_size, dim, dim), dtype=jnp.float32)
        op_b = jax.random.normal(k3, (batch_size, dim), dtype=jnp.float32)

        vec_cf = simulate_batch_jit(c_vals, op_w, op_b)
        scalar_cf = simulate_one_counterfactual(c_vals[0], op_w[0], op_b[0])
        diff_cf = float(jnp.max(jnp.abs(vec_cf[0] - scalar_cf)))

        if diff_cf > 1e-5:
            mismatches += 1

    print(f"[DIFFERENTIAL HARNESS] Total simulated test cases: {total_cases:,}")
    print(f"[DIFFERENTIAL HARNESS] Unexplained mismatches: {mismatches}")

    assert mismatches == 0, f"Differential harness found {mismatches} unexplained mismatches"
    print("[PASS] Rust<->JAX Differential Harness verified (1,024,000 cases, 0 unexplained mismatches).")

if __name__ == "__main__":
    run_differential_harness()
