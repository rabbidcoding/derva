# ORIGIN-Ω ZERO — Epistemic Status Lattice Specification (T012)

> **INVARIANT:** Status lattice is a partial order; `CONTESTED` never collapses silently to a boolean.  
> **KPI:** 100% of illegal status transition attempts rejected in property tests (>= 1e6 cases).

---

## 1. Estructura del Lattice Epistémico

El espacio de estados de conocimiento en ORIGIN-Ω ZERO se formaliza como un reticulado de orden parcial (Partial Order Lattice):

$$\text{Unknown} \sqsubset \text{Hypothesis} \sqsubset \text{Supported} \sqsubset \text{Verified}$$
$$\text{Supported} \sqcap \text{ContradictionWitness} \to \text{Contested}$$
$$\text{Verified} \sqcap \text{ContradictionWitness} \to \text{Contested}$$
$$\text{Any} \sqcap \text{RefutationWitness} \to \text{Refuted}$$

Diagrama del Lattice:

```text
               Verified (3)
                  |
              Supported (2)
             /           \
       Contested (4)     Hypothesis (1)
             \           /
               Refuted (5)
                  |
               Unknown (0)
```

---

## 2. Definición de Estados y Propiedades

1. **`Unknown` (0)**: Estado por defecto sin observación previa.
2. **`Hypothesis` (1)**: Formulado mediante observación primaria inicial (`ProofKind::Observation`).
3. **`Supported` (2)**: Respaldado por derivación deductiva o soporte empírico (`ProofKind::Derivation`).
4. **`Verified` (3)**: Confirmado formalmente mediante demostración lógica o verificación exhaustiva (`ProofKind::FormalVerification`).
5. **`Contested` (4)**: Existencia de testigo de contradicción activo (`ProofKind::ContradictionWitness`). No colapsa a booleano; requiere resolución de conflicto.
6. **`Refuted` (5)**: Estado terminal irrecuperable ante testigo de refutación formal (`ProofKind::RefutationWitness`).

---

## 3. Matriz de Promoción Transaccional (`promote`)

Queda prohibido cualquier salto arbitrario o uso de `unwrap()` en transiciones epistémicas:

| Estado Actual | `ProofKind` Aceptado | Resultado Promocionado |
| :--- | :--- | :--- |
| `Unknown` | `Observation` | `Ok(Status::Hypothesis)` |
| `Hypothesis` | `Derivation` | `Ok(Status::Supported)` |
| `Supported` | `FormalVerification` | `Ok(Status::Verified)` |
| `Supported` | `ContradictionWitness` | `Ok(Status::Contested)` |
| `Verified` | `ContradictionWitness` | `Ok(Status::Contested)` |
| `Contested` | `Derivation` | `Ok(Status::Supported)` |
| `Contested` | `FormalVerification` | `Ok(Status::Verified)` |
| Cualquier Estado | `RefutationWitness` | `Ok(Status::Refuted)` |
| `Refuted` | Cualquier `ProofKind` | `Err(EpistemicError::IllegalPromotion)` |
| Cualquier combinación invalida | Cualesquiera no definidos | `Err(EpistemicError::IllegalPromotion)` |

---

## 4. Invariantes de No-Colapso y Ausencia de Panic

1. **No-Colapso Booleano**: `CONTESTED` nunca evalúa implícitamente a `true` o `false`. Métodos como `is_verified()` retornan `true` únicamente para `Status::Verified`.
2. **Cero Panics**: La función `promote(s, proof)` retorna `Result<Status, EpistemicError>` sin utilizar `unwrap()`, `expect()` o pánicos en tiempo de ejecución.
