# ORIGIN-Ω ZERO — Lazy Relevance Quotient Specification (T015)

> **INVARIANT:** Never materializes the universe $\Omega$; state equivalence is calculated locally w.r.t relevant set $R$.  
> **KPI:** Active quotient reduces >= 10x states in >= 70% of benchmark scenarios designed for redundancy.

---

## 1. Cociente de Relevancia Lazily Evaluado

En sistemas complejos, enumerar o proyectar el espacio completo de estados posibles $\Omega$ resulta intratable. ORIGIN-Ω ZERO formaliza la equivalencia de estados respecto a un conjunto de distinciones relevante $R$:

$$s_1 \sim_R s_2 \iff \forall d \in R, \quad d(s_1) = d(s_2)$$

El cociente activo $S / \sim_R$ agrupa estados que son indistinguibles para las decisiones del dominio actual.

---

## 2. Garantías Falsables

1. **Cero Materialización de $\Omega$**: El motor de cociente evalúa únicamente los estados reales observados en el pipeline activo mediante un iterator lazy.
2. **Preservación Total de Decisiones (100%)**: Toda decisión observable basada en el conjunto relevante $R$ produce exactamente el mismo resultado al sustituir cualquier representante de la clase de equivalencia.
3. **Factor de Reducción $\ge 10\times$**: En escenarios con alta redundancia o estados efímeros no distinguibles por $R$, la partición en clases de equivalencia reduce el espacio de búsqueda en al menos un orden de magnitud.

---

## 3. Algoritmo de Partición en Rust

```rust
pub struct RelevantSet {
    pub distinctions: Vec<Distinction>,
}

pub fn equivalent(a: &WorldSig, b: &WorldSig, r: &RelevantSet) -> bool {
    r.distinctions.iter().all(|d| d.evaluate(&a.bytes) == d.evaluate(&b.bytes))
}

pub fn partition_active_quotient(states: &[WorldSig], r: &RelevantSet) -> Vec<Vec<WorldSig>> {
    let mut classes: Vec<Vec<WorldSig>> = Vec::new();
    for s in states {
        let mut found = false;
        for cls in &mut classes {
            if equivalent(&cls[0], s, r) {
                cls.push(s.clone());
                found = true;
                break;
            }
        }
        if !found {
            classes.push(vec![s.clone()]);
        }
    }
    classes
}
```
