# INVARIANT: Pure x86_64 Assembly routine for chunked popcnt cardinality with 0 external dependencies.
# SAFETY: Pure leaf function; rdi=data_ptr, rsi=num_u64s.
# AUDIT-LENSES: Steve Wozniak, John Carmack, Bjarne Stroustrup

.text

.global chunked_popcnt_u64_asm
.type chunked_popcnt_u64_asm, @function
chunked_popcnt_u64_asm:
    # rdi = data_ptr, rsi = num_u64s
    # Returns total count of set bits in rax
    xor %rax, %rax            # accumulator rax = 0
    test %rsi, %rsi
    jz .Lpop_done

    mov %rsi, %rcx
    shr $2, %rcx              # Process 4 x u64 unrolled per loop iteration
    jz .Lpop_residual

.Lpop_unroll:
    popcnt (%rdi), %r8
    popcnt 8(%rdi), %r9
    add %r8, %rax
    popcnt 16(%rdi), %r10
    add %r9, %rax
    popcnt 24(%rdi), %r11
    add %r10, %rax
    add %r11, %rax
    add $32, %rdi
    dec %rcx
    jnz .Lpop_unroll

.Lpop_residual:
    and $3, %rsi              # remaining 0-3 u64s
    jz .Lpop_done

.Lpop_scalar:
    popcnt (%rdi), %r8
    add %r8, %rax
    add $8, %rdi
    dec %rsi
    jnz .Lpop_scalar

.Lpop_done:
    ret
.size chunked_popcnt_u64_asm, .-chunked_popcnt_u64_asm
