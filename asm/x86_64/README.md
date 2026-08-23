# ORIGIN-Ω ZERO — Isolated x86_64 Assembly Kernel Directory (T003)

> **INVARIANT:** Assembly routines are isolated behind Rust FFI/intrinsics references with pure portable Rust fallbacks.  
> **DEPENDENCY RULE:** `asm/` MUST NOT depend on Python, JAX, or external runtimes. Pure x86_64 machine instructions only.
