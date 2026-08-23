# INVARIANT: Pure x86_64 Assembly routine for chunked popcnt cardinality with 0 external dependencies.
# SAFETY: Pure leaf function; rdi=data_ptr, rsi=num_u64s. Returns count in rax.
# AUDIT-LENSES: Steve Wozniak, John Carmack, Bjarne Stroustrup
# NOTE: xor-zero before every popcnt to break false output-dependency on Sandy/Ivy/Haswell.

.text

.global chunked_popcnt_u64_asm
.type chunked_popcnt_u64_asm, @function
chunked_popcnt_u64_asm:
    # rdi = data_ptr, rsi = num_u64s
    # Returns total count of set bits in rax
    xor %eax, %eax            # accumulator = 0
    test %rsi, %rsi
    jz .Lpop_done

    mov %rsi, %rcx
    shr $2, %rcx              # rcx = num_u64s / 4 (full unrolled chunks)
    jz .Lpop_residual

    .p2align 4                # Align loop entry to 16-byte boundary for fetch efficiency
.Lpop_unroll:
    # Break false output-dependency on %r8-%r11 (Sandy/Ivy/Haswell errata)
    xor %r8d, %r8d
    xor %r9d, %r9d
    xor %r10d, %r10d
    xor %r11d, %r11d
    popcnt (%rdi), %r8
    popcnt 8(%rdi), %r9
    popcnt 16(%rdi), %r10
    popcnt 24(%rdi), %r11
    add %r8, %rax
    add %r9, %rax
    add %r10, %rax
    add %r11, %rax
    add $32, %rdi
    dec %rcx
    jnz .Lpop_unroll

.Lpop_residual:
    and $3, %esi              # remaining 0-3 u64s (use 32-bit and to zero-extend)
    jz .Lpop_done

.Lpop_scalar:
    xor %r8d, %r8d            # Break false dependency
    popcnt (%rdi), %r8
    add %r8, %rax
    add $8, %rdi
    dec %esi
    jnz .Lpop_scalar

.Lpop_done:
    ret
.size chunked_popcnt_u64_asm, .-chunked_popcnt_u64_asm
