# ORIGIN-Ω ZERO — Authoritative State Algebra (T011)

> **INVARIANT:** State $S = (G, C, E, U, O, B, Z)$ is the single authoritative ground truth.  
> **KPI:** 0 authoritative state fields duplicated in numerical or runtime components.

---

## 1. Definición del Estado Autoritativo

El estado global $S$ de ORIGIN-Ω ZERO está formalizado como una 7-tupla inmutable y versionada por transacciones:

$$S = (G, C, E, U, O, B, Z)$$

Donde los 7 dominios canónicos son:

- **$G$ (`GraphRoot`)**: Grafo de claims e hipótesis epistémicas indexado por `ORID`.
- **$C$ (`ConstraintRoot`)**: Colección de restricciones lógicas y de dominio.
- **$E$ (`EvidenceRoot`)**: Hipergrafo de proveniencia e invarianza de evidencias primarias y derivadas.
- **$U$ (`OperatorRoot`)**: Catálogo de operadores causales, transformaciones y reglas de reescritura.
- **$O$ (`ObligationRoot`)**: Conjunto de obligaciones de verificación pendientes y resueltas.
- **$B$ (`Budget`)**: Presupuesto acotado de recursos (pasos CPU, tiempo wall-clock, memoria máxima).
- **$Z$ (`ArtifactRoot`)**: Artefactos compilados inmutables (SSA-OIR) protegidos por hashes de proveniencia.

---

## 2. Matriz de Permisos y Control de Mutabilidad

Queda prohibida la duplicación o mutación directa de cualquier componente del estado fuera del kernel autoritativo de Rust.

| Componente del Estado $S$ | Dueño Autorizado (Escritura) | Subsistemas de Solo Lectura |
| :--- | :--- | :--- |
| **$G$ (GraphRoot)** | `origin-kernel` (vía `StateTxn`) | `origin-reason`, `origin-search`, JAX Coprocessor |
| **$C$ (ConstraintRoot)** | `origin-constraints` (vía `StateTxn`) | `origin-logic`, `origin-verify` |
| **$E$ (EvidenceRoot)** | `origin-verify` (vía `StateTxn`) | `origin-causal`, `origin-store` |
| **$U$ (OperatorRoot)** | `origin-causal` (vía `StateTxn`) | `origin-fast`, JAX Coprocessor |
| **$O$ (ObligationRoot)** | `origin-verify` (vía `StateTxn`) | `origin-plan`, `origin-logic` |
| **$B$ (Budget)** | `origin-runtime` (vía `StateTxn`) | Todos los subsistemas |
| **$Z$ (ArtifactRoot)** | `origin-compiler` (vía `StateTxn`) | `origin-fast`, `asm/x86_64` |

---

## 3. Semántica Transaccional (`StateTxn`)

Toda mutación sobre $S$ exige instanciar una transacción aislada `StateTxn`:

1. **Schema Versioning**: Todo estado incluye `schema_version = 0`.
2. **Aislamiento e Inmutabilidad**: Las modificaciones se acumulan en deltas dentro de `StateTxn`.
3. **Commit Atómico**: El método `.commit()` valida los invariantes epistémicos y la versión del schema antes de promover el estado base.
4. **Rollback Determinista**: Si la validación falla o se agota el presupuesto $B$, la transacción se aborta sin alterar el estado autoritativo previo.
