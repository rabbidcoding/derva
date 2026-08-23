# INVARIANT: Pure x86_64 SIMD Assembly routines for bitset operations with 0 external dependencies.
# SAFETY: Pure leaf function; rdi=dst, rsi=a, rdx=b, rcx=num_u64s; non-overlapping pointers.
# AUDIT-LENSES: Carmack, Stroustrup, Ritchie

.text

.global bitset_and_avx2_asm
.type bitset_and_avx2_asm, @function
bitset_and_avx2_asm:
    # rdi = dst, rsi = a, rdx = b, rcx = num_u64s
    # Process 4 x u64 (256-bit YMM register) per iteration
    mov %rcx, %r8          # r8 = num_u64s (preserve original count)
    shr $2, %rcx           # rcx = num_u64s / 4 (full 256-bit chunks)
    jz .Land_residual      # if zero full chunks, skip to residual

.Land_loop:
    vmovdqu (%rsi), %ymm0
    vmovdqu (%rdx), %ymm1
    vpand %ymm1, %ymm0, %ymm2   # Integer AND (NOT vandps which is float domain)
    vmovdqu %ymm2, (%rdi)
    add $32, %rsi
    add $32, %rdx
    add $32, %rdi
    dec %rcx
    jnz .Land_loop
    vzeroupper

.Land_residual:
    # Process remaining 0-3 u64s with scalar instructions
    and $3, %r8            # r8 = num_u64s % 4
    jz .Land_done

.Land_scalar:
    mov (%rsi), %rax
    and (%rdx), %rax
    mov %rax, (%rdi)
    add $8, %rsi
    add $8, %rdx
    add $8, %rdi
    dec %r8
    jnz .Land_scalar

.Land_done:
    ret
.size bitset_and_avx2_asm, .-bitset_and_avx2_asm


.global bitset_or_avx2_asm
.type bitset_or_avx2_asm, @function
bitset_or_avx2_asm:
    # rdi = dst, rsi = a, rdx = b, rcx = num_u64s
    mov %rcx, %r8
    shr $2, %rcx
    jz .Lor_residual

.Lor_loop:
    vmovdqu (%rsi), %ymm0
    vmovdqu (%rdx), %ymm1
    vpor %ymm1, %ymm0, %ymm2    # Integer OR
    vmovdqu %ymm2, (%rdi)
    add $32, %rsi
    add $32, %rdx
    add $32, %rdi
    dec %rcx
    jnz .Lor_loop
    vzeroupper

.Lor_residual:
    and $3, %r8
    jz .Lor_done

.Lor_scalar:
    mov (%rsi), %rax
    or (%rdx), %rax
    mov %rax, (%rdi)
    add $8, %rsi
    add $8, %rdx
    add $8, %rdi
    dec %r8
    jnz .Lor_scalar

.Lor_done:
    ret
.size bitset_or_avx2_asm, .-bitset_or_avx2_asm


.global bitset_andnot_avx2_asm
.type bitset_andnot_avx2_asm, @function
bitset_andnot_avx2_asm:
    # rdi = dst, rsi = a, rdx = b, rcx = num_u64s
    # Computes dst[i] = a[i] & ~b[i]  (set difference A \ B)
    mov %rcx, %r8
    shr $2, %rcx
    jz .Ldiff_residual

.Ldiff_loop:
    vmovdqu (%rsi), %ymm0       # ymm0 = a
    vmovdqu (%rdx), %ymm1       # ymm1 = b
    vpandn %ymm0, %ymm1, %ymm2  # ymm2 = ~ymm1 & ymm0 = a & ~b
    vmovdqu %ymm2, (%rdi)
    add $32, %rsi
    add $32, %rdx
    add $32, %rdi
    dec %rcx
    jnz .Ldiff_loop
    vzeroupper

.Ldiff_residual:
    and $3, %r8
    jz .Ldiff_done

.Ldiff_scalar:
    mov (%rsi), %rax
    mov (%rdx), %rcx
    not %rcx
    and %rcx, %rax
    mov %rax, (%rdi)
    add $8, %rsi
    add $8, %rdx
    add $8, %rdi
    dec %r8
    jnz .Ldiff_scalar

.Ldiff_done:
    ret
.size bitset_andnot_avx2_asm, .-bitset_andnot_avx2_asm
