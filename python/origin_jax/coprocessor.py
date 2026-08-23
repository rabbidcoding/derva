# INVARIANT: Rust owns authoritative state; JAX is pure numerical coprocessor only.
# KPI: Zero trainable parameters in numerical coprocessor.

import jax
import jax.numpy as jnp

def evaluate_vector_field(matrix: jnp.ndarray, vector: jnp.ndarray) -> jnp.ndarray:
    """
    Pure numerical vector field evaluation without trainable parameters or autograd.
    """
    return jnp.dot(matrix, vector)
