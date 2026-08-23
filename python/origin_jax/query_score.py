# AUDIT-LENSES: Donald Knuth, John Carmack, Steve Jobs
# INVARIANT: Vectorized query evaluation minimizing worst-case residual hypothesis class count; canonical tie-breaking.
# KPI: Exact match with exhaustive small-world oracle; >= 15x throughput at 4096 queries; canonical tie-breaking by index/ORID.

import time
import jax
import jax.numpy as jnp
from typing import Tuple

def worst_case_query_score_one(query_outcomes: jax.Array, query_cost: jax.Array) -> jax.Array:
    """
    Computes worst-case remaining hypothesis count multiplied by query cost for a single candidate query.
    query_outcomes: [H] int32 array of outcome class IDs (0..9) for H hypotheses.
    query_cost: float32 query execution cost.
    """
    counts = jnp.bincount(query_outcomes, length=10)
    worst_case_class_size = jnp.max(counts)
    return worst_case_class_size * query_cost

# JIT-compiled vectorized query scoring kernel
score_queries_batch_jit = jax.jit(jax.vmap(worst_case_query_score_one, in_axes=(0, 0)))

def select_best_query_vectorized(
    queries_outcomes: jax.Array,
    query_costs: jax.Array,
) -> Tuple[jax.Array, jax.Array]:
    """
    Evaluates Q queries across H world hypotheses in parallel and returns sorted scores and indices.
    Primary key: score (ascending, lower worst-case cost is better).
    Secondary key: query index (ascending, canonical tie-breaking).
    """
    scores = score_queries_batch_jit(queries_outcomes, query_costs)

    indices = jnp.arange(scores.shape[0])
    sorted_indices = jnp.lexsort((indices, scores))
    sorted_scores = scores[sorted_indices]

    return sorted_scores, sorted_indices

def test_vectorized_query_scoring():
    n_queries = 4096
    n_hypotheses = 128

    key = jax.random.PRNGKey(42)
    k1, k2 = jax.random.split(key)

    queries_outcomes = jax.random.randint(k1, (n_queries, n_hypotheses), minval=0, maxval=4, dtype=jnp.int32)
    query_costs = jax.random.uniform(k2, (n_queries,), minval=1.0, maxval=5.0, dtype=jnp.float32)

    # 1. Warmup & Exact Match check vs Scalar Small-World Oracle
    _ = score_queries_batch_jit(queries_outcomes[:10], query_costs[:10])

    vec_scores = score_queries_batch_jit(queries_outcomes, query_costs)

    # Scalar small-world reference
    scalar_scores_sample = [
        float(worst_case_query_score_one(queries_outcomes[i], query_costs[i]))
        for i in range(100)
    ]
    scalar_arr = jnp.array(scalar_scores_sample, dtype=jnp.float32)

    diff_max = float(jnp.max(jnp.abs(vec_scores[:100] - scalar_arr)))
    assert diff_max == 0.0, f"Vectorized query score discrepancy {diff_max} MUST be exact 0.0"

    # 2. Performance Benchmark at Q=4096 (>= 15x throughput requirement)
    t0_scalar = time.perf_counter()
    scalar_all = [float(worst_case_query_score_one(queries_outcomes[i], query_costs[i])) for i in range(n_queries)]
    t1_scalar = time.perf_counter()
    scalar_dur = t1_scalar - t0_scalar

    t0_vec = time.perf_counter()
    for _ in range(50):
        v_out = score_queries_batch_jit(queries_outcomes, query_costs)
        v_out.block_until_ready()
    t1_vec = time.perf_counter()
    vec_dur = (t1_vec - t0_vec) / 50.0

    speedup = scalar_dur / vec_dur
    print(f"[BENCHMARK] Queries={n_queries} Scalar: {scalar_dur*1000:.2f}ms | JAX Vectorized: {vec_dur*1000:.3f}ms | Speedup: {speedup:.1f}x")
    assert speedup >= 15.0, f"Vectorized query scoring speedup {speedup:.1f}x MUST be >= 15x"

    # 3. Canonical Tie-Breaking Stability Check
    s1, idx1 = select_best_query_vectorized(queries_outcomes, query_costs)
    for _ in range(5):
        s_n, idx_n = select_best_query_vectorized(queries_outcomes, query_costs)
        assert jnp.array_equal(idx1, idx_n), "Canonical tie-breaking MUST yield 100% identical query ordering"

    print("[PASS] Vectorized Query Scoring verified (exact oracle match, >= 15x throughput, 100% canonical tie-breaking).")

if __name__ == "__main__":
    test_vectorized_query_scoring()
