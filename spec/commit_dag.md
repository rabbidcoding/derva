# ORIGIN-Ω ZERO — Merkle Commit DAG Specification (T025)

> **INVARIANT:** Merkle DAG of immutable commits; 100% replay accuracy; any delta or policy change alters the commit root ORID.  
> **KPI:** Replay commit root exact 100%; zero history rewriting during branch or merge operations.

---

## 1. Estructura Formal del Nodo de Commit (`CommitNode`)

Cada nodo de commit en el DAG de Merkle representa una mutación o transición autoritativa del estado $S$:

```rust
pub struct CommitNode {
    pub parents: Vec<ORID>,       // 0 para root commit, 1 para commit lineal, 2 para merge
    pub delta_orid: ORID,          // Apunta al delta canónico de la transacción StateTxn
    pub policy_root: ORID,         // Apunta al contrato/reglas de gobernanza vigentes
    pub author: String,            // Identificador autenticado del agente u operador
    pub timestamp: u64,            // Timestamp determinista en nanosegundos
}
```

### 1.1 Codificación Canónica y Identificador (`ORID`)
El `ORID` de un commit se calcula mediante la función de direccionamiento por contenido `ObjectKind::Commit`:

$$\text{ORID}_{\text{commit}} = \text{ORID::compute}(\text{ObjectKind::Commit}, \text{canonical\_bytes}(\text{CommitNode}))$$

---

## 2. Invariantes de Seguridad y Recreación de Estado

1. **Sensibilidad Total al Hash**: Alterar cualquier byte en `parents`, `delta_orid`, `policy_root`, `author` o `timestamp` modifica de forma inmediata e irreversible el hash resultante del commit.
2. **Replay Exacto del Estado (100%)**: Reconstruir el estado aplicando la secuencia determinista de commits desde la raíz reproduce exactamente el mismo $S = (G, C, E, U, O, B, Z)$.
3. **No Reescritura de Historia**: Las operaciones de bifurcación (`branch`) y fusión (`merge`) crean nuevos nodos de commit cuyo array `parents` hace referencia a los nodos existentes. Los commits pasados son estrictamente inmutables.
