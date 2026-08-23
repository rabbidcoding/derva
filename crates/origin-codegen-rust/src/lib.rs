#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO — OIR->Rust Codegen Subsystem
// INVARIANT: Safe Rust codegen with 0 unsafe usage, explicit guards, and budget charges.

pub mod lower;

pub use lower::RustCodegen;

pub fn crate_name() -> &'static str {
    "origin-codegen-rust"
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_codegen_rust_crate_boundary() {
        assert_eq!(super::crate_name(), "origin-codegen-rust");
    }
}
