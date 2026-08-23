#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO — OIR->JAX Codegen Subsystem
// INVARIANT: Pure JAX codegen with strict effect containment and zero trainable parameters.

pub mod lower;

pub use lower::{JaxCodegen, JaxLoweringError};

pub fn crate_name() -> &'static str {
    "origin-codegen-jax"
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_codegen_jax_crate_boundary() {
        assert_eq!(super::crate_name(), "origin-codegen-jax");
    }
}
