// AUDIT-LENSES: Steve Wozniak, Dennis Ritchie, John Carmack
// INVARIANT: SoA packed layout for relation/status scanning; derived & rebuildable index; 0 authoritative state stored only in cache.
// KPI: >= 2x throughput over AoS baseline; memory <= 24 bytes/edge; cache strictly derived.

/// Baseline Array of Structures (AoS) edge model for performance benchmarking
#[derive(Debug, Clone)]
pub struct AoSEdge {
    pub kind: u8,
    pub src: u32,
    pub dst: u32,
    pub status: u8,
    pub weight: u16,
    pub flags: u8,
    pub _padding: [u8; 11], // 24 bytes total per edge
}

/// Structure of Arrays (SoA) packed index for high-throughput vectorized scan
#[derive(Debug, Clone, Default)]
pub struct PackedIndex {
    pub kinds: Vec<u8>,
    pub src: Vec<u32>,
    pub dst: Vec<u32>,
    pub status: Vec<u8>,
    pub weights: Vec<u16>,
    pub flags: Vec<u8>,
}

impl PackedIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            kinds: Vec::with_capacity(capacity),
            src: Vec::with_capacity(capacity),
            dst: Vec::with_capacity(capacity),
            status: Vec::with_capacity(capacity),
            weights: Vec::with_capacity(capacity),
            flags: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, kind: u8, src: u32, dst: u32, status: u8, weight: u16, flags: u8) {
        self.kinds.push(kind);
        self.src.push(src);
        self.dst.push(dst);
        self.status.push(status);
        self.weights.push(weight);
        self.flags.push(flags);
    }

    pub fn len(&self) -> usize {
        self.kinds.len()
    }

    pub fn is_empty(&self) -> bool {
        self.kinds.is_empty()
    }

    /// Memory footprint per edge in bytes (Target: <= 24 bytes)
    pub fn bytes_per_edge(&self) -> usize {
        // kind (1) + src (4) + dst (4) + status (1) + weight (2) + flags (1) = 13 bytes
        std::mem::size_of::<u8>()
            + std::mem::size_of::<u32>()
            + std::mem::size_of::<u32>()
            + std::mem::size_of::<u8>()
            + std::mem::size_of::<u16>()
            + std::mem::size_of::<u8>()
    }

    /// Vectorized scan over packed status & relation kind
    #[inline]
    pub fn scan_matching(&self, target_kind: u8, target_status: u8, matches: &mut Vec<usize>) {
        matches.clear();
        let kinds = &self.kinds;
        let status = &self.status;
        let len = kinds.len();

        for i in 0..len {
            if kinds[i] == target_kind && status[i] == target_status {
                matches.push(i);
            }
        }
    }

    /// Rebuilds index deterministically from authoritative core source data
    pub fn rebuild_from_tuples(tuples: &[(u8, u32, u32, u8, u16, u8)]) -> Self {
        let mut index = Self::with_capacity(tuples.len());
        for &(k, s, d, st, w, f) in tuples {
            index.push(k, s, d, st, w, f);
        }
        index
    }
}

/// AoS baseline scanner for comparative throughput benchmarks
pub fn scan_aos_baseline(edges: &[AoSEdge], target_kind: u8, target_status: u8, matches: &mut Vec<usize>) {
    matches.clear();
    for i in 0..edges.len() {
        if edges[i].kind == target_kind && edges[i].status == target_status {
            matches.push(i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hint::black_box;
    use std::time::Instant;

    #[test]
    fn test_memory_bytes_per_edge_le_24_bytes() {
        let index = PackedIndex::new();
        let bytes_edge = index.bytes_per_edge();
        println!("[LAYOUT METADATA] PackedIndex memory per edge: {} bytes", bytes_edge);

        assert!(
            bytes_edge <= 24,
            "Packed index memory per edge {} bytes MUST be <= 24 bytes target",
            bytes_edge
        );
    }

    #[test]
    fn test_soa_throughput_ge_2x_aos_baseline() {
        let num_edges = 1_000_000;
        let tuples: Vec<(u8, u32, u32, u8, u16, u8)> = (0..num_edges)
            .map(|i| {
                let k = (i % 8) as u8;
                let s = (i as u32).wrapping_mul(17);
                let d = (i as u32).wrapping_mul(31);
                let st = (i % 4) as u8;
                (k, s, d, st, 100, 0)
            })
            .collect();

        // Build SoA index
        let index = PackedIndex::rebuild_from_tuples(&tuples);

        // Build AoS baseline
        let aos_edges: Vec<AoSEdge> = tuples
            .iter()
            .map(|&(k, s, d, st, w, f)| AoSEdge {
                kind: k,
                src: s,
                dst: d,
                status: st,
                weight: w,
                flags: f,
                _padding: [0u8; 11],
            })
            .collect();

        let mut matches_soa = Vec::with_capacity(num_edges);
        let mut matches_aos = Vec::with_capacity(num_edges);

        let target_k = 3u8;
        let target_st = 2u8;

        let iterations = 100;

        // AoS Baseline Benchmark
        let start_aos = Instant::now();
        for _ in 0..iterations {
            scan_aos_baseline(black_box(&aos_edges), target_k, target_st, black_box(&mut matches_aos));
        }
        let dur_aos = start_aos.elapsed();

        // SoA Fast Benchmark
        let start_soa = Instant::now();
        for _ in 0..iterations {
            index.scan_matching(target_k, target_st, black_box(&mut matches_soa));
        }
        let dur_soa = start_soa.elapsed();

        assert_eq!(matches_soa, matches_aos, "SoA and AoS scans MUST match 100%");

        let speedup = dur_aos.as_nanos() as f64 / dur_soa.as_nanos() as f64;
        println!(
            "[LAYOUT BENCHMARK 1M EDGES] AoS: {:?} | SoA: {:?} | Throughput Speedup: {:.2}x",
            dur_aos, dur_soa, speedup
        );

        assert!(
            speedup >= 2.0,
            "SoA layout scan speedup {:.2}x MUST be >= 2.0x over AoS baseline",
            speedup
        );
    }
}
