# ORIGIN-Ω ZERO — Benchmark Constitution (T009)

> **INVARIANT:** No performance claims without fixed seeds, frozen baselines, and CV <= 0.03 (3%).  
> **KPI:** 100% of benchmarks enforce maximum coefficient of variation threshold.

---

## 1. Hardware & Execution Classes

1. **Class-A (Dev Host)**: Standard x86_64 CPU (AVX2), 8-core CPU, 16GiB RAM.
2. **Class-B (Accelerated Numerical Coprocessor)**: CPU + JAX numerical coprocessor (AVX2/AVX-512).

## 2. Reglas de Medición y Políticas de Seed

- **Política de Seed Fija**: Todas las generaciones de datos sintéticos deben usar semillas PRNG deterministas fijadas (`seed = 42`).
- **Warmup Obligatorio**: Mínimo 50 ejecuciones de calentamiento antes de iniciar la captura de muestras.
- **Muestreo y Coeficiente de Variación (CV)**:
  - Mínimo 500 muestras por microbenchmark.
  - $CV = \frac{\sigma}{\mu} \le 0.03$. Si $CV > 0.03$, la muestra se clasifica automáticamente como `is_noisy = true` y la prueba se invalida.
- **Inmutabilidad Post-Ejecución**: Queda prohibido modificar la especificación o baseline de un benchmark tras observar sus resultados sin incrementar su versión.
