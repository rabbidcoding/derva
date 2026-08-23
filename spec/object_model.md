# ORIGIN-Ω ZERO — Canonical Object Model Specification (T013)

> **INVARIANT:** Same semantic object => identical canonical byte encoding cross-platform.  
> **KPI:** 100% canonical encoding round-trip accuracy across >= 1e6 property cases.

---

## 1. Los 7 Objetos Canónicos Fundamentales

Todos los objetos de conocimiento en ORIGIN-Ω ZERO se reducen a 7 clases canónicas inmutables:

1. **`Entity`**: Entidad de dominio identificada por nombre y esquema de atributos.
2. **`Observation`**: Ingesta primaria con identificador de fuente y timestamp estandarizado.
3. **`Claim`**: Proposición epistémica con estado lattice y referencias de proveniencia.
4. **`Evidence`**: Registro de evidencia vinculado a la observación primaria u otras evidencias.
5. **`Operator`**: Transformación o acción causal con esquema y costo cuantitativo.
6. **`Obligation`**: Compromiso explícito de verificación vinculado a un claim.
7. **`Artifact`**: Binario o bytecode compilado (SSA-OIR) con proveniencia declarada.

---

## 2. Invariante de Codificación Canónica

Para garantizar la identidad por direccionamiento de contenido (ORID) y la reproducibilidad entre plataformas (Linux x86_64, macOS ARM64):

1. **Endianness Fijo**: Todos los enteros (`u64`, `u32`, `u16`) se codifican en formato Big-Endian (`to_be_bytes()`).
2. **Prefix de Longitud**: Todas las cadenas de texto UTF-8 y colecciones vectoriales incluyen un prefijo de longitud `u64` en Big-Endian antes de los bytes del contenido.
3. **Exclusión de Campos Efímeros**: Punteros de memoria local, timestamps de ejecución no deterministas o depuraciones efímeras están prohibidos en la codificación canónica.

---

## 3. Especificación del Trait Canónico (`Canonical`)

```rust
pub trait Canonical {
    fn encode_canonical(&self, out: &mut Vec<u8>);

    fn canonical_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.encode_canonical(&mut buf);
        buf
    }
}
```

---

## 4. Estructura de Codificación de Bytes

### 4.1 Entity
`[domain_id_len (u64)] + [domain_id_bytes] + [name_len (u64)] + [name_bytes]`

### 4.2 Observation
`[source_id_len (u64)] + [source_id_bytes] + [timestamp (u64_be)] + [payload_len (u64)] + [payload_bytes]`

### 4.3 Claim
`[id.hash (32b)] + [statement_len (u64)] + [statement_bytes] + [status (u8)] + [parent_count (u64)] + [parent_hashes]`

### 4.4 Evidence
`[id.hash (32b)] + [raw_orid.hash (32b)] + [source_id_len (u64)] + [source_id_bytes] + [timestamp (u64_be)]`

### 4.5 Operator
`[id.hash (32b)] + [name_len (u64)] + [name_bytes] + [schema_len (u64)] + [schema_bytes] + [status (u8)] + [cost (u64_be)]`

### 4.6 Obligation
`[id.hash (32b)] + [claim_id.hash (32b)] + [kind_len (u64)] + [kind_bytes] + [resolved (u8)]`

### 4.7 Artifact
`[id.hash (32b)] + [name_len (u64)] + [name_bytes] + [data_len (u64)] + [data_bytes]`
