# INVARIANT: Pure x86_64 SIMD Assembly routines for bitset operations with 0 external dependencies.
# SAFETY: Pure leaf function; rdi=dst, rsi=a, rdx=b, rcx=num_u64s; non-overlapping pointers.

.global bitset_and_avx2_asm
bitset_and_avx2_asm:
    # rdi = dst, rsi = a, rdx = b, rcx = num_u64s
    # Process 4 x u64 (256-bit YMM register) per iteration
    shr $2, %rcx
    jz .L_done

.L_loop:
    vmovdqu (%rsi), %ymm0
    vmovdqu (%rdx), %ymm1
    vandps %ymm1, %ymm0, %ymm2
    vmovdqu %ymm2, (%rdi)
    add $32, %rsi
    add $32, %rdx
    add $32, %rdi
    dec %rcx
    jnz .L_loop
    vzeroupper

.L_done:
    ret
