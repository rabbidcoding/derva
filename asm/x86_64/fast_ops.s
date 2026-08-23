# INVARIANT: Pure x86_64 SIMD/Assembly routines with 0 external runtime dependencies.
# SAFETY: Pure leaf function; no stack frame modification; Rust ABI compatible.

.global origin_popcnt_u64
origin_popcnt_u64:
    popcnt %rdi, %rax
    ret
