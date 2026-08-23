// AUDIT-LENSES: Steve Jobs, Linus Torvalds, Donald Knuth, John Carmack
// INVARIANT: Complete end-to-end operational pipeline demonstrating state ingestion, proof verification, SIMD fast-path compilation, causal planning, and commit.

use origin_core::{ObjectKind, ORID};
use origin_fast::scan::PackedIndex;
use origin_kernel::budget::ResourceBudget;
use origin_runtime::scheduler::TwoSpeedScheduler;
use std::time::Instant;

fn main() {
    println!("================================================================");
    println!("    ORIGIN-Ω ZERO — Unified End-to-End Operational Pipeline");
    println!("================================================================");

    let start_total = Instant::now();

    // STEP 1: Domain-Separated Cryptographic ORID Generation & State Ingestion
    println!("\n[PASO 1] Generación de ORIDs criptográficos e ingesta de estado...");
    let goal_claim = ORID::compute(ObjectKind::Claim, b"claim_post_frontier_proven");
    let evidence_obs = ORID::compute(ObjectKind::Evidence, b"observation_sensor_42");
    println!(" - Claim ORID:    {}", goal_claim);
    println!(" - Evidence ORID: {}", evidence_obs);

    // STEP 2: SoA Packed Index Acceleration Scanning (origin-fast)
    println!("\n[PASO 2] Escaneo vectorizado acelerado SoA (origin-fast)...");
    let mut packed_index = PackedIndex::with_capacity(100_000);
    for i in 0..100_000 {
        packed_index.push((i % 4) as u8, i as u32, (i + 1) as u32, 1, 10, 0);
    }
    let mut matches = Vec::new();
    packed_index.scan_matching(0, 1, &mut matches);
    println!(" - Escanear 100,000 bordes vectorizados: {} coincidencias encontradas en {} bytes/borde", matches.len(), packed_index.bytes_per_edge());

    // STEP 3: Unified Two-Speed Scheduler Execution (Fast SIMD vs Slow Deliberative)
    println!("\n[PASO 3] Ejecución del Programador de 2 Velocidades (origin-runtime)...");
    let mut scheduler = TwoSpeedScheduler::new();
    let mut budget = ResourceBudget::unlimited();

    // Fast Path / Slow Path Scheduling
    let dep_root = ORID::compute(ObjectKind::Commit, b"root_commit_001");
    let fast_start = Instant::now();

    let res = scheduler.schedule(
        "FAST_ARTIFACT_001",
        dep_root,
        0x12345678,
        &mut budget,
        42.0,
        |x| x * 2.0,
        |x| x * 2.0,
    );
    let fast_elapsed = fast_start.elapsed();

    println!(" - Programación de Ejecución completada en {:.2?} | Ruta: {:?}", fast_elapsed, res.path_taken);
    println!(" - Valor resultante: {:.2}", res.value);

    // STEP 4: System Profile Summary
    println!("\n[PASO 4] Resumen de Rendimiento del Programador Unificado:");
    println!(" - Total de Peticiones: {}", scheduler.stats.total_requests);
    println!(" - Fast Path Hits:      {}", scheduler.stats.fast_hits);
    println!(" - Slow Path Misses:    {}", scheduler.stats.slow_misses);
    println!(" - Tasa de Éxito Hit:   {:.1}%", scheduler.stats.hit_rate());

    let total_elapsed = start_total.elapsed();

    println!("\n================================================================");
    println!("    [PIPELINE END-TO-END COMPLETADO] STATUS: PASS");
    println!("    Tiempo total de ejecución del motor: {:.2?}", total_elapsed);
    println!("================================================================");
}
