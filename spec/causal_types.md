# ORIGIN-Ω ZERO — Causal Status Type Algebra Specification (T018)

> **INVARIANT:** Zero observational to verified causal promotions without explicit intervention or mechanistic witness.  
> **KPI:** False causal promotion count = 0 in synthetic known-truth test suite; 100% of promotions include provenance + assumptions.

---

## 1. Los 5 Estados Causales Canónicos (`CausalStatus`)

1. **`Observational`**: Correlación pasiva o asociación puramente observacional en los datos.
2. **`AssumedCausal`**: Hipótesis causal condicionada a supuestos explícitos no respaldados aún por intervención.
3. **`Interventional`**: Operación respaldada por experimento o intervención causal directa ($do$-calculus).
4. **`Mechanistic`**: Operador respaldado por derivación física, lógica o de primer principio.
5. **`VerifiedCausal`**: Estado causal final verificado que combina intervención experimental y derivación mecanicista.

---

## 2. Estructura de Testigo Causal (`CausalWitness`)

Para que una promoción causal sea válida, el testigo debe incluir proveniencia y supuestos explícitos:

```rust
pub enum CausalWitnessKind {
    Assumption,
    Intervention,
    MechanisticDerivation,
}

pub struct CausalWitness {
    pub kind: CausalWitnessKind,
    pub witness_orid: ORID,
    pub provenance_roots: Vec<ORID>,
    pub assumptions: Vec<String>,
}
```

---

## 3. Matriz de Promoción Causal (`causal_promote`)

| Estado Inicial | `CausalWitnessKind` | Estado Resultante | Requisitos |
| :--- | :--- | :--- | :--- |
| `Observational` | `Assumption` | `AssumedCausal` | Supuestos no vacíos |
| `Observational` | `Intervention` | `Interventional` | Proveniencia primaria |
| `AssumedCausal` | `MechanisticDerivation` | `Mechanistic` | Demostración formal |
| `Interventional` | `MechanisticDerivation` | `VerifiedCausal` | Proveniencia + Intervención |
| `Mechanistic` | `Intervention` | `VerifiedCausal` | Proveniencia + Intervención |
| `Observational` | Cualquier directo a `VerifiedCausal` | **`Err(IllegalPromotion)`** | **Bloqueo Fail-Closed** |
