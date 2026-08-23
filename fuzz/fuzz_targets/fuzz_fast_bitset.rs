#![no_main]
// AUDIT-LENSES: Ken Thompson, Donald Knuth, Dennis Ritchie
// INVARIANT: Fuzz target comparing AVX2 SIMD/Assembly bitset against pure-Rust scalar reference; 0 mismatch allowed.

use libfuzzer_sys::fuzz_target;
use origin_fast::bitset::PackedBitset;
use origin_fast::cardinality::CardinalityEngine;

fuzz_target!(|data: &[u8]| {
    if data.len() < 16 {
        return;
    }

    let u64_count = data.len() / 8;
    let mut words_a = Vec::with_capacity(u64_count);
    let mut words_b = Vec::with_capacity(u64_count);

    for i in 0..u64_count {
        let chunk: [u8; 8] = data[i * 8..(i + 1) * 8].try_into().unwrap();
        let val = u64::from_le_bytes(chunk);
        if i % 2 == 0 {
            words_a.push(val);
        } else {
            words_b.push(val);
        }
    }

    let min_len = words_a.len().min(words_b.len());
    if min_len == 0 {
        return;
    }

    words_a.truncate(min_len);
    words_b.truncate(min_len);

    let bitset_a = PackedBitset::new(words_a.clone());
    let bitset_b = PackedBitset::new(words_b.clone());

    let mut dst_scalar = vec![0u64; min_len];
    let mut dst_fast = vec![0u64; min_len];

    // Intersect check
    bitset_a.intersect_scalar(&bitset_b, &mut dst_scalar);
    bitset_a.intersect(&bitset_b, &mut dst_fast);
    assert_eq!(dst_scalar, dst_fast, "Intersect SIMD vs Scalar mismatch!");

    // Cardinality check
    let card_naive = CardinalityEngine::count_naive_scalar(&words_a);
    let card_chunked = CardinalityEngine::count_chunked(&words_a);
    let card_fast = CardinalityEngine::count_ones(&words_a);

    assert_eq!(card_naive, card_chunked, "Cardinality chunked vs naive mismatch!");
    assert_eq!(card_naive, card_fast, "Cardinality fast vs naive mismatch!");
});
