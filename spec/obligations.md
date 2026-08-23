# ORIGIN-Ω ZERO — Verification Obligation Algebra Specification (T017)

> **INVARIANT:** Verification obligations are explicit typed contracts; 0 self-satisfaction by protected claim.  
> **KPI:** 100% of critical promotions resolve explicit obligation sets; expired obligations invalidate freshness 100%.

---

## 1. Tipos de Obligación de Verificación (`ObligationKind`)

ORIGIN-Ω ZERO reemplaza la deuda de verificación por 7 tipos de obligaciones explícitas:

```rust
pub enum ObligationKind {
    SourceRequired,
    IndependentSource,
    Execution,
    Proof,
    Intervention,
    Freshness,
    HumanApproval,
}
```

1. **`SourceRequired`**: Exige al menos una fuente primaria autenticada.
2. **`IndependentSource`**: Exige verificación cruzada por una fuente no relacionada por proveniencia.
3. **`Execution`**: Exige una traza de ejecución determinista y reproducible.
4. **`Proof`**: Exige un certificado de verificación formal o prueba matemática.
5. **`Intervention`**: Exige evidencia de intervención causal directa.
6. **`Freshness`**: Exige un sello temporal dentro del presupuesto de vigencia ($t \le t_{\text{expire}}$).
7. **`HumanApproval`**: Exige firma o autorización explícita para operaciones críticas.

---

## 2. Reglas de Invariancia

1. **Prohibición de Auto-Satisfacción**: Ninguna obligación asignada al claim $C$ puede ser resuelta usando el propio claim $C$ como testigo (`witness_orid != target_claim`). El testigo debe ser un objeto de evidencia independiente.
2. **Expiración de Vigencia (`Freshness`)**: Si el tiempo actual $t > t_{\text{expire}}$, la obligación pasa a estar invalidada y bloquea cualquier promoción epistémica que dependa de ella.
3. **Detección de Ciclos (`detect_obligation_cycles`)**: El arnés de auditoría bloquea cualquier grafo de obligaciones con ciclos de dependencia transitiva.
