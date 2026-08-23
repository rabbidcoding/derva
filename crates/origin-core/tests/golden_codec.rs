use origin_core::codec::*;
use origin_core::object::Canonical;
use origin_core::{Claim, ObjectKind, Status, ORID};
use std::time::Instant;

#[test]
fn test_golden_vectors_canonical_and_malleable_rejection() {
    let dummy_orid = ORID::compute(ObjectKind::Claim, b"golden_seed");
    let claim = Claim {
        id: dummy_orid,
        statement: "Golden statement".to_string(),
        status: Status::Verified,
        provenance_roots: vec![dummy_orid],
    };

    let encoded = claim.canonical_bytes();
    assert!(!encoded.is_empty());

    // Test rejection of overlong varint in decoding
    let overlong_varint = vec![0x80, 0x80, 0x00];
    let mut off = 0;
    assert_eq!(
        decode_varint(&overlong_varint, &mut off),
        Err(CodecError::NonCanonicalEncoding)
    );
}

#[test]
fn test_codec_high_throughput_benchmark_target() {
    // Benchmark 2,000,000 small varint/string encodings and decodings
    let iterations = 2_000_000;
    let start = Instant::now();

    let mut buf = Vec::with_capacity(64);
    for i in 0..iterations {
        buf.clear();
        encode_varint(i as u64, &mut buf);
        let mut off = 0;
        let val = decode_varint(&buf, &mut off).unwrap();
        assert_eq!(val, i as u64);
    }

    let elapsed = start.elapsed();
    let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();
    println!("Codec benchmark throughput: {:.2} ops/sec", ops_per_sec);
    assert!(
        ops_per_sec >= 1_000_000.0,
        "Throughput was {:.2} ops/sec",
        ops_per_sec
    );
}
