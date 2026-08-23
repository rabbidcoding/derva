# ORIGIN-Ω ZERO — Canonical Binary Codec Specification (T022)

> **INVARIANT:** Byte-for-byte determinism 100% cross-platform; 0 malleable or non-canonical encodings permitted.  
> **KPI:** Throughput >= 500 MB/s on large payloads or >= 2M small objects/s in release build.

---

## 1. Especificación del Codec Binario Canónico

El codec binario de ORIGIN-Ω ZERO define la representación autoritativa byte a byte de todos los tipos del kernel de Rust sin depender de serdes externos o librerías dinámicas.

### 1.1 Codificación Varint Canónica
- Los enteros unsigned `u64` se codifican mediante LEB128 modificado.
- **Regla Estricta de No-Maleabilidad**: Los varints con bytes adicionales de relleno de ceros (ej. codificar `0` como `[0x80, 0x00]`) son rechazados inmediatamente con `CodecError::NonCanonicalEncoding`.
- El último byte de un varint TIENE obligatoriamente el bit más significativo desactivo (`b & 0x80 == 0`). Un varint de más de 10 bytes es rechazado.

### 1.2 Codificación de Longitud y Arrays
- Todos los campos de longitud variable (`str`, `Vec<u8>`) van precedidos por su longitud como varint canónico.
- Se impone un límite máximo estricto (`MAX_BOUND = 64 MiB`). Si la longitud declarada excede el límite permitido, el decodificador rechaza con `CodecError::BoundedLengthExceeded`.

### 1.3 Rechazo de Bytes Adicionales (`TrailingBytes`)
- El proceso de decodificación debe consumir exactamente los bytes del objeto. Si al finalizar quedan bytes no consumidos en la rebanada, el decodificador falla con `CodecError::TrailingBytes`.

---

## 2. Matriz de Errores de Decodificación (`CodecError`)

1. `UnexpectedEof`: El buffer finalizó antes de poder leer el objeto completo.
2. `NonCanonicalEncoding`: Codificación overlong o bits de relleno no canónicos detectados.
3. `BoundedLengthExceeded`: La longitud declarada supera el límite máximo permitido.
4. `TrailingBytes`: Existen bytes sobrantes en el buffer después de decodificar el objeto.
5. `InvalidUtf8`: Cadena de texto no es UTF-8 válida.
