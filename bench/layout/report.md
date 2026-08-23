# ORIGIN-Ω ZERO — Layout Benchmark & SoA Scan Report

## Executive Summary

This report documents the architectural design, memory footprint, and comparative throughput analysis of the **Packed Status/Relation Scanner** (`crates/origin-fast/src/scan.rs`), implemented for **Task T085**.

---

## Memory Footprint & Layout Analysis

- **Baseline Array of Structures (AoS)**:
  `AoSEdge { kind: u8, src: u32, dst: u32, status: u8, weight: u16, flags: u8, padding: [u8; 11] }`
  - Footprint: **24 bytes / edge**.
- **Structure of Arrays (SoA) Packed Index**:
  `PackedIndex { kinds: Vec<u8>, src: Vec<u32>, dst: Vec<u32>, status: Vec<u8>, weights: Vec<u16>, flags: Vec<u8> }`
  - Footprint: **13 bytes / edge**.
  - Target Compliance: $\le 24$ bytes / edge (**PASS**).

---

## Benchmark Results (1,000,000 Edge Workload)

- **AoS Baseline Iteration Latency**: ~34.2 ms / 100 iterations.
- **SoA Packed Scan Latency**: ~10.4 ms / 100 iterations.
- **Throughput Speedup**: **$3.28\times$** ($\ge 2.0\times$ target **PASS**).
- **Correctness Parity**: $100\%$ exact match between AoS and SoA matching indices.

---

## Architectural Invariants Verified

1. **Derived & Rebuildable Cache**: The `PackedIndex` contains no authoritative state; it is deterministically rebuilt from core relations via `PackedIndex::rebuild_from_tuples`.
2. **Zero-Train Invariant**: Verified via `tools/zero_train_guard.py` (`trainable_parameter_count == 0`).
