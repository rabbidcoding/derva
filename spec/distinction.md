# ORIGIN-Ω ZERO — Distinction Semantics Specification (T014)

> **INVARIANT:** Distinction is an explicit domain-relative predicate with cost; 0 global implicit distinctions.  
> **KPI:** Decision-relative equivalence demonstrated across formal suite.

---

## 1. Concepto de Distinción Formal

Una **Distinción** $D$ es un observable o predicado tipado que separa dos o más hipótesis o estados únicamente en relación con un dominio y una decisión explícitos:

$$D = (\text{DomainID}, \text{PredicateID}, \text{Cost})$$

### 1.1 Invariantes Fundamentales
1. **Separación Relativa al Dominio**: No existen distinciones globales ni implícitas en ORIGIN-Ω ZERO. Toda distinción pertenece a un `DomainID` explícito.
2. **Costo de Evaluación**: Cada evaluación de una distinción consume una cantidad cuantitativa declarada de recursos (`Cost`).
3. **Equivalencia Relativa a la Decisión**: Dos estados $S_1$ y $S_2$ son equivalentes bajo un conjunto relevante de distinciones $\mathcal{D}_R$ si y solo si:

$$\forall D \in \mathcal{D}_R, \quad D(S_1) = D(S_2)$$

---

## 2. Definición del Trait y Estructura en Rust

```rust
pub struct DomainId(pub String);
pub struct PredicateId(pub String);
pub struct Cost(pub u64);

pub struct Distinction {
    pub domain: DomainId,
    pub predicate: PredicateId,
    pub cost: Cost,
}
```

---

## 3. Matriz de Falsabilidad

- **Falla Rígida**: Instanciar una distinción sin `DomainId` o sin `Cost` explícito cancela la compilación o ejecución transaccional.
- **Transparencia Presupuestaria**: La ejecución de $N$ evaluaciones de distinciones deduce automáticamente $N \times \text{Cost}$ del presupuesto $B$.
