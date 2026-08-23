# AUDIT-LENSES: John Carmack, Donald Knuth, Bjarne Stroustrup
# INVARIANT: Vectorized hypothesis evaluation via jax.vmap(score_one); deterministic tie-breaking; zero trainable parameters.
# KPI: float <= 2 ULP / rtol=1e-5 tolerance; >= 20x throughput vs Python scalar at N >= 4096; 100% ranking stability.

import time
import jax
import jax.numpy as jnp
from typing import Tuple
try:
    from .schema import CandidateBatch
except ImportError:
    from schema import CandidateBatch

def score_one_hypothesis(single_vector: jax.Array) -> jax.Array:
    """
    Pure numerical scoring function for a single hypothesis feature vector.
    Computes weighted linear dot product and penalty term without trainable parameters.
    """
    dim = single_vector.shape[-1]
    weights = jnp.linspace(0.1, 1.0, dim, dtype=single_vector.dtype)
    base_score = jnp.dot(single_vector, weights)
    penalty = jnp.sum(jnp.square(jnp.maximum(0.0, -single_vector)))
    return base_score - 0.5 * penalty

# JIT-compiled vectorized hypothesis scoring kernel
score_batch_jit = jax.jit(jax.vmap(score_one_hypothesis))

def rank_hypotheses_deterministic(batch: CandidateBatch) -> Tuple[jax.Array, jax.Array]:
    """
    Evaluates CandidateBatch hypotheses in parallel and performs deterministic tie-breaking.
    Returns: (sorted_scores, sorted_indices)
    """
    raw_scores = score_batch_jit(batch.values)
    # Mask out invalid candidates with negative infinity
    valid_scores = jnp.where(batch.masks, raw_scores, -jnp.inf)

    # Deterministic tie-breaking: primary key = -score (descending), secondary key = candidate index (ascending)
    indices = jnp.arange(valid_scores.shape[0])
    sorted_indices = jnp.lexsort((indices, -valid_scores))
    sorted_scores = valid_scores[sorted_indices]

    return sorted_scores, sorted_indices

def test_vectorized_hypothesis_scoring():
    n_candidates = 4096
    dim = 64

    key = jax.random.PRNGKey(42)
    values = jax.random.normal(key, (n_candidates, dim), dtype=jnp.float32)
    masks = jnp.ones((n_candidates,), dtype=jnp.bool_)

    batch = CandidateBatch(values=values, masks=masks)

    # 1. Differential Accuracy: JAX Vectorized vs Python Scalar Loop
    # Warmup JIT kernel
    _ = score_batch_jit(batch.values[:10])

    vectorized_scores = score_batch_jit(batch.values)

    # Scalar reference evaluation for comparison
    scalar_scores_list = []
    for i in range(100):  # Validate first 100 candidates
        scalar_scores_list.append(float(score_one_hypothesis(batch.values[i])))
    scalar_scores = jnp.array(scalar_scores_list, dtype=jnp.float32)

    # Check differential float tolerance <= 1e-5 (<= 2 ULP)
    diff_max = jnp.max(jnp.abs(vectorized_scores[:100] - scalar_scores))
    assert diff_max <= 1e-5, f"Vectorized vs scalar discrepancy {diff_max} exceeds tolerance spec"

    # 2. Performance Benchmark: >= 20x Throughput at N=4096
    t0_scalar = time.perf_counter()
    scalar_all = [float(score_one_hypothesis(batch.values[i])) for i in range(n_candidates)]
    t1_scalar = time.perf_counter()
    scalar_duration = t1_scalar - t0_scalar

    # Warmup vectorized
    _ = score_batch_jit(batch.values)

    t0_vec = time.perf_counter()
    for _ in range(50):
        vec_out = score_batch_jit(batch.values)
        vec_out.block_until_ready()
    t1_vec = time.perf_counter()
    vec_duration = (t1_vec - t0_vec) / 50.0

    speedup = scalar_duration / vec_duration
    print(f"[BENCHMARK] N={n_candidates} Scalar: {scalar_duration*1000:.2f}ms | JAX Vectorized: {vec_duration*1000:.3f}ms | Speedup: {speedup:.1f}x")
    assert speedup >= 20.0, f"Vectorized throughput speedup {speedup:.1f}x MUST be >= 20x"

    # 3. Ranking Stability across repeated runs
    s1, idx1 = rank_hypotheses_deterministic(batch)
    for _ in range(5):
        s_n, idx_n = rank_hypotheses_deterministic(batch)
        assert jnp.array_equal(idx1, idx_n), "Deterministic tie-breaking MUST produce 100% identical ranking"

    print("[PASS] Vectorized Hypothesis Scoring verified (accuracy, >=20x speedup, 100% ranking stability).")

if __name__ == "__main__":
    test_vectorized_hypothesis_scoring()
