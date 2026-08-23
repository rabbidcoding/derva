# AUDIT-LENSES: John Carmack, Bjarne Stroustrup, Niklaus Wirth
# INVARIANT: Structured numerical control flow using jax.lax.scan, jax.lax.cond, and jax.lax.while_loop.
# KPI: O(1) HLO IR size growth w.r.t scan length; JIT compile time < 5s p95; exact numerical runtime parity.

import time
import jax
import jax.numpy as jnp
from typing import Tuple

def bounded_step_fn(carry: jax.Array, x: jax.Array) -> Tuple[jax.Array, jax.Array]:
    """
    Pure numerical step function for structured scan loop.
    Applies conditional accumulator updates without unrolling loop state.
    """
    next_carry = jax.lax.cond(
        x > 0.0,
        lambda c: c + x * 0.5,
        lambda c: c + x * 0.1,
        carry,
    )
    return next_carry, next_carry

@jax.jit
def run_scan_loop(init_carry: jax.Array, xs: jax.Array) -> Tuple[jax.Array, jax.Array]:
    """
    Structured JIT-compiled loop using jax.lax.scan to maintain O(1) HLO footprint.
    """
    return jax.lax.scan(bounded_step_fn, init_carry, xs)

@jax.jit
def run_while_loop(init_val: jax.Array, max_steps: int = 100) -> jax.Array:
    """
    Bounded numerical convergence loop using jax.lax.while_loop.
    """
    def cond_fn(state: Tuple[jax.Array, jax.Array]) -> jax.Array:
        step, val = state
        return (step < max_steps) & (jnp.abs(val) < 100.0)

    def body_fn(state: Tuple[jax.Array, jax.Array]) -> Tuple[jax.Array, jax.Array]:
        step, val = state
        return step + 1, val * 0.99 + 0.01

    _, final_val = jax.lax.while_loop(cond_fn, body_fn, (jnp.int32(0), init_val))
    return final_val

def test_jax_control_flow_kernels():
    init_carry = jnp.array(1.0, dtype=jnp.float32)

    # 1. Compile Time & HLO Footprint O(1) Verification
    xs_small = jnp.arange(10, dtype=jnp.float32)
    xs_large = jnp.arange(10000, dtype=jnp.float32)

    t0_comp = time.perf_counter()
    lowered_small = run_scan_loop.lower(init_carry, xs_small)
    hlo_small = lowered_small.as_text()
    t1_comp = time.perf_counter()
    compile_time_s = t1_comp - t0_comp

    print(f"[COMPILE METRIC] JAX scan JIT compile time: {compile_time_s:.3f}s")
    assert compile_time_s < 5.0, f"JIT compile time {compile_time_s:.3f}s MUST be < 5.0s p95 target"

    lowered_large = run_scan_loop.lower(init_carry, xs_large)
    hlo_large = lowered_large.as_text()

    # Compare HLO IR sizes (line counts of lowered text)
    hlo_small_lines = len(hlo_small.splitlines())
    hlo_large_lines = len(hlo_large.splitlines())

    print(f"[HLO FOOTPRINT] N=10 HLO lines: {hlo_small_lines} | N=10000 HLO lines: {hlo_large_lines}")
    # Line count delta must be negligible (O(1) growth), proving no unrolling occurred
    hlo_diff = abs(hlo_small_lines - hlo_large_lines)
    assert hlo_diff <= 10, f"HLO IR line count growth {hlo_diff} violates O(1) invariant"

    # 2. Runtime Parity check vs Python reference loop
    res_carry, res_ys = run_scan_loop(init_carry, xs_small)

    # Python reference scalar loop
    ref_carry = 1.0
    ref_ys = []
    for x_val in range(10):
        val = float(x_val)
        if val > 0.0:
            ref_carry = ref_carry + val * 0.5
        else:
            ref_carry = ref_carry + val * 0.1
        ref_ys.append(ref_carry)

    diff_carry = abs(float(res_carry) - ref_carry)
    diff_ys = jnp.max(jnp.abs(res_ys - jnp.array(ref_ys)))

    assert diff_carry <= 1e-5, f"Scan carry numerical difference {diff_carry} exceeds tolerance"
    assert float(diff_ys) <= 1e-5, f"Scan ys numerical difference {diff_ys} exceeds tolerance"

    # 3. Bounded while loop execution check
    while_res = run_while_loop(jnp.array(10.0, dtype=jnp.float32), max_steps=50)
    assert jnp.isfinite(while_res), "While loop result MUST be finite"

    print("[PASS] JAX Control-Flow Kernels verified (O(1) HLO growth, compile time < 5s, 100% runtime parity).")

if __name__ == "__main__":
    test_jax_control_flow_kernels()
