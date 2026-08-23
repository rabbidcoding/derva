# ORIGIN-Ω ZERO — Nine-Instruction Micro-ISA Specification (T019)

> **INVARIANT:** Exactly 9 primitive opcodes v1. Adding opcodes requires architecture ADR + benchmark justification.  
> **KPI:** 100% of reference reasoning operations expressible by micro-ISA composition.

---

## 1. El Conjunto Canónico de 9 Instrucciones (`OpCode`)

El micro-ISA de ORIGIN-Ω ZERO consta de **exactamente 9 primitivas** orthogonales e inmutables:

| Opcode | Byte Value | Nombre | Semántica Formal |
| :--- | :--- | :--- | :--- |
| `0x01` | `OBSERVE` | Ingesta de observación primaria | Captura datos desde un canal autenticado generando un `EvidenceRecord` primario. |
| `0x02` | `PROPOSE` | Instanciación de hipótesis | Crea una nueva afirmación proposicional (`Claim`) en estado `Hypothesis`. |
| `0x03` | `RELATE` | Asociación formal | Establece una relación lógica, causal o probabilística entre dos `ORID`s. |
| `0x04` | `REFINE` | Acotación de hipótesis | Restringe el dominio de una hipótesis o ajusta restricciones activas $C$. |
| `0x05` | `QUERY` | Evaluación de estado | Ejecuta una consulta o inspección no destructiva sobre el grafo de estado $S$. |
| `0x06` | `INTERVENE` | Aplicación causal | Aplica un operador causal $do(X=x)$ con presupuesto $B$ y prueba de intervención. |
| `0x07` | `VERIFY` | Promoción epistémica | Procesa un certificado formal o testigo para promover un estado epistémico. |
| `0x08` | `COMMIT` | Transacción atómica | Aplica atómicamente un `StateTxn` sobre el estado global autoritativo $S$. |
| `0x09` | `COMPILE` | Generación de OIR | Transforma una región pura de conocimiento en un artefacto SSA-OIR compilado. |

---

## 2. Invariante de Conteo Estricto

1. **`COUNT == 9`**: Queda estrictamente prohibido añadir el opcode 10 o duplicar primitivas sin un proceso formal de ADR.
2. **Expresividad por Composición**: El 100% de los flujos de inferencia, razonamiento simbiótico y optimización se expresan como secuencias binarias de estas 9 instrucciones.
