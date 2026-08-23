# ORIGIN-Ω ZERO — Kill Criteria Contract (T002)

> **INVARIANT:** Any violation of kill criteria triggers automatic architecture rejection.

---

## 1. Definición de Kill Criteria

Un **Kill Criterion** es una regla de fallo inmediato que obliga a abandonar una rama de optimización, módulo o claim sin posibilidad de bypass ni atenuantes.

## 2. Matriz de Criterios de Abandono (Kill Matrix)

| Claims / Subsystem | Métricas Críticas | Umbral de Abandono (Kill Threshold) | Acción Automatizada |
| :--- | :--- | :--- | :--- |
| **Zero-Training Guard** | `trainable_parameters` | `> 0` | Fallo inmediato de CI, veto de PR, revert automático. |
| **Epistemic Engine** | `illegal_verified_promotions` | `> 0` | Democión inmediata a `RESEARCH_ONLY`. |
| **Causal Graph** | `unwitnessed_causal_promotion` | `> 0` | Bloqueo de publicación en `origin-causal`. |
| **JAX Numerical Coprocessor** | `unexplained_differential_mismatches` | `> 0` | Desactivación del JAX Fast-Path, fallback a Rust pure scalar. |
| **Assembly/SIMD Acceleration** | `intrinsics_speedup_ratio` | `< 1.15x` | Eliminación de kernels ASM manuales, retención de intrinsics o Rust puro. |
| **Compilation Artifacts** | `stale_artifact_executions` | `> 0` | Invalidación forzada de todo artifact precompilado. |
