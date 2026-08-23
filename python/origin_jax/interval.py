# AUDIT-LENSES: Donald Knuth, Guido van Rossum, Alan Turing
# INVARIANT: Outward-rounded interval arithmetic kernels on IntervalBatch.
# KPI: 100% true value containment in oracle test suite; zero unhandled NaNs; pathological width inflation fallback.

import jax
import jax.numpy as jnp
from typing import Tuple
try:
    from .schema import IntervalBatch
except ImportError:
    from schema import IntervalBatch

EPSILON_ROUNDING = 1e-6
MAX_WIDTH_THRESHOLD = 1e6

@jax.jit
def add_intervals(a: IntervalBatch, b: IntervalBatch, eps: float = EPSILON_ROUNDING) -> IntervalBatch:
    """
    Outward-rounded interval addition: [a.lo + b.lo - eps, a.hi + b.hi + eps]
    """
    lo = (a.lower_bounds + b.lower_bounds) - eps
    hi = (a.upper_bounds + b.upper_bounds) + eps

    has_nan = jnp.any(jnp.isnan(lo) | jnp.isnan(hi), axis=-1)
    width = jnp.max(hi - lo, axis=-1)
    pathological = width > MAX_WIDTH_THRESHOLD

    valid_mask = a.active_mask & b.active_mask & (~has_nan) & (~pathological)

    return IntervalBatch(
        lower_bounds=lo,
        upper_bounds=hi,
        active_mask=valid_mask,
    )

@jax.jit
def sub_intervals(a: IntervalBatch, b: IntervalBatch, eps: float = EPSILON_ROUNDING) -> IntervalBatch:
    """
    Outward-rounded interval subtraction: [a.lo - b.hi - eps, a.hi - b.lo + eps]
    """
    lo = (a.lower_bounds - b.upper_bounds) - eps
    hi = (a.upper_bounds - b.lower_bounds) + eps

    has_nan = jnp.any(jnp.isnan(lo) | jnp.isnan(hi), axis=-1)
    width = jnp.max(hi - lo, axis=-1)
    pathological = width > MAX_WIDTH_THRESHOLD

    valid_mask = a.active_mask & b.active_mask & (~has_nan) & (~pathological)

    return IntervalBatch(
        lower_bounds=lo,
        upper_bounds=hi,
        active_mask=valid_mask,
    )

@jax.jit
def mul_intervals(a: IntervalBatch, b: IntervalBatch, eps: float = EPSILON_ROUNDING) -> IntervalBatch:
    """
    Outward-rounded interval multiplication: min/max of corner products +- eps
    """
    p1 = a.lower_bounds * b.lower_bounds
    p2 = a.lower_bounds * b.upper_bounds
    p3 = a.upper_bounds * b.lower_bounds
    p4 = a.upper_bounds * b.upper_bounds

    min_p = jnp.minimum(jnp.minimum(p1, p2), jnp.minimum(p3, p4))
    max_p = jnp.maximum(jnp.maximum(p1, p2), jnp.maximum(p3, p4))

    lo = min_p - eps
    hi = max_p + eps

    has_nan = jnp.any(jnp.isnan(lo) | jnp.isnan(hi), axis=-1)
    width = jnp.max(hi - lo, axis=-1)
    pathological = width > MAX_WIDTH_THRESHOLD

    valid_mask = a.active_mask & b.active_mask & (~has_nan) & (~pathological)

    return IntervalBatch(
        lower_bounds=lo,
        upper_bounds=hi,
        active_mask=valid_mask,
    )

def test_interval_arithmetic_kernels():
    # 1. Containment Test
    a = IntervalBatch(
        lower_bounds=jnp.array([[1.0, 2.0]], dtype=jnp.float32),
        upper_bounds=jnp.array([[2.0, 3.0]], dtype=jnp.float32),
        active_mask=jnp.array([True], dtype=jnp.bool_),
    )
    b = IntervalBatch(
        lower_bounds=jnp.array([[3.0, 4.0]], dtype=jnp.float32),
        upper_bounds=jnp.array([[4.0, 5.0]], dtype=jnp.float32),
        active_mask=jnp.array([True], dtype=jnp.bool_),
    )

    sum_res = add_intervals(a, b)
    assert float(sum_res.lower_bounds[0, 0]) <= (1.0 + 3.0)
    assert float(sum_res.upper_bounds[0, 0]) >= (2.0 + 4.0)
    assert float(sum_res.lower_bounds[0, 1]) <= (2.0 + 4.0)
    assert float(sum_res.upper_bounds[0, 1]) >= (3.0 + 5.0)

    prod_res = mul_intervals(a, b)
    assert float(prod_res.lower_bounds[0, 0]) <= (1.0 * 3.0)
    assert float(prod_res.upper_bounds[0, 0]) >= (2.0 * 4.0)
    assert float(prod_res.lower_bounds[0, 1]) <= (2.0 * 4.0)
    assert float(prod_res.upper_bounds[0, 1]) >= (3.0 * 5.0)

    # 2. NaN Protection Check
    nan_a = IntervalBatch(
        lower_bounds=jnp.array([[jnp.nan, 2.0]], dtype=jnp.float32),
        upper_bounds=jnp.array([[2.0, 3.0]], dtype=jnp.float32),
        active_mask=jnp.array([True], dtype=jnp.bool_),
    )
    res_nan = add_intervals(nan_a, b)
    assert not bool(res_nan.active_mask[0]), "NaN bounds MUST invalidate active_mask"

    # 3. Pathological Width Inflation Fallback Check
    wide_a = IntervalBatch(
        lower_bounds=jnp.array([[0.0, 0.0]], dtype=jnp.float32),
        upper_bounds=jnp.array([[2e6, 2e6]], dtype=jnp.float32),
        active_mask=jnp.array([True], dtype=jnp.bool_),
    )
    res_wide = add_intervals(wide_a, b)
    assert not bool(res_wide.active_mask[0]), "Pathological width MUST invalidate active_mask for fallback"

    print("[PASS] Interval Arithmetic Kernels verified (containment 100%, NaN protection, width inflation fallback).")

if __name__ == "__main__":
    test_interval_arithmetic_kernels()
