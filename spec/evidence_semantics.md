# ORIGIN-Ω ZERO — Evidence vs Derived Information Semantics Specification (T016)

> **INVARIANT:** Primary observations, derived information, and trust are strictly separated; 0 double-counting by lineage duplication.  
> **KPI:** 100% of derivations retain original roots of provenance; 100 copies of 1 root count as 1 independent domain.

---

## 1. Clasificación Tipada de Evidencia

Toda evidencia en ORIGIN-Ω ZERO se clasifica formalmente como observación primaria o información derivada:

```rust
pub enum SupportKind {
    Primary { raw_orid: ORID },
    Derived { rule_id: String, parents: Vec<ORID> },
}
```

1. **`Primary`**: Observación primaria capturada desde un canal de ingestión autenticado. Posee proveniencia atómica `raw_orid`.
2. **`Derived`**: Registro deductivo u operador causal derivado mediante una regla lógicamente identificada (`rule_id`) a partir de padres explícitos (`parents`).

---

## 2. Invariante de No Doble Conteo por Lineage

El conteo de proveniencia independiente calcula el conjunto único (deduplicado) de raíces primarias subyacentes:

$$\text{IndependentCount}(E) = | \{ \text{root} \in \text{Roots}(E) \} |$$

### Regla Anti-Amplificación
- Si una observación primaria $O_1$ es duplicada, citada o derivada $N$ veces (ej. $N=100$), el peso de proveniencia independiente permanece estrictamente en **$1$**.
- La duplicación de linajes jamás promociona un estado epistémico.

---

## 3. Requisito de Verificación (`Verified` Path)

Cualquier reclamo o estado promovido a `Verified` requiere obligatoriamente un camino continuo y acíclico hasta al menos una raíz `Primary` válida. Queda prohibida la promoción basada en derivaciones vacías o circulares.
