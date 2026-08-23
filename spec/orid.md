# ORIGIN-Ω ZERO — ORID Content Addressing Specification (T022/T023)

> **INVARIANT:** Content addressing by domain-separated SHA-256 hash over canonical bytes; 100% type-domain separation and bidirectional string round-trip.  
> **KPI:** 0 collisions in 1e8 synthetic objects; 100% parsing/formatting accuracy.

---

## 1. Estructura y Formato del ORID

El **ORID (Origin Resource Identifier)** es la dirección persistente e inmutable de cualquier entidad u objeto en el kernel autoritativo.

### 1.1 Separación de Dominios por Tipo (`domain_prefix`)

Para garantizar que el mismo contenido binario representado bajo dos tipos distintos produzca hashes distintos, se antepone un prefijo de dominio terminado en NUL (`\0`):

| `ObjectKind` | Prefijo de Dominio (`domain_prefix`) |
| :--- | :--- |
| `Entity` | `origin:entity:v1\0` |
| `Observation` | `origin:observation:v1\0` |
| `Claim` | `origin:claim:v1\0` |
| `Evidence` | `origin:evidence:v1\0` |
| `Operator` | `origin:operator:v1\0` |
| `Obligation` | `origin:obligation:v1\0` |
| `Commit` | `origin:commit:v1\0` |
| `Artifact` | `origin:artifact:v1\0` |

$$\text{hash} = \text{SHA-256}(\text{domain\_prefix} \parallel \text{canonical\_bytes})$$

---

## 2. Representación Canónica en Texto (`Display` & `FromStr`)

El formato canónico en texto sigue el patrón:

$$\texttt{orid:<kind>:<hex_hash_64>}$$

Ejemplo:
`orid:Claim:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`

---

## 3. Garantías y Reglas de Dominio

1. **Separación Estricta de Dominios (100%)**: Si el payload $P$ se evalúa con `ObjectKind::Claim` y con `ObjectKind::Evidence`, los hashes resultantes son computacionalmente independientes y nunca colisionan.
2. **Parsing sin Ambigüedad**: `ORID::from_str(s)` valida el prefijo `orid:`, el tipo `kind` y la cadena hexadecimal exacta de 64 caracteres.
3. **Resistencia a Colisiones**: Cero colisiones observadas en la suite sintética de $1\times 10^8$ iteraciones.
