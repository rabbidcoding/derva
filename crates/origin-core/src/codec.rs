// INVARIANT: Byte-for-byte determinism 100% cross-platform; 0 malleable or non-canonical encodings permitted.
// KPI: Decoder rejects 100% malleable fixtures; throughput >= 500 MB/s on large payloads or >= 2M small objects/s.

use std::fmt;

pub const DEFAULT_MAX_BOUND: usize = 64 * 1024 * 1024; // 64 MiB limit

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecError {
    UnexpectedEof,
    NonCanonicalEncoding,
    BoundedLengthExceeded,
    TrailingBytes,
    InvalidUtf8,
}

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodecError::UnexpectedEof => write!(f, "Unexpected end of file during decoding"),
            CodecError::NonCanonicalEncoding => {
                write!(f, "Non-canonical overlong varint encoding rejected")
            }
            CodecError::BoundedLengthExceeded => {
                write!(f, "Declared length exceeds maximum allowed bound")
            }
            CodecError::TrailingBytes => {
                write!(f, "Unconsumed trailing bytes found after decoding")
            }
            CodecError::InvalidUtf8 => write!(f, "String is not valid UTF-8"),
        }
    }
}

impl std::error::Error for CodecError {}

/// Encodes a u64 into a canonical LEB128 varint byte stream.
pub fn encode_varint(mut value: u64, out: &mut Vec<u8>) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
            out.push(byte);
        } else {
            out.push(byte);
            break;
        }
    }
}

/// Decodes a canonical LEB128 varint byte stream.
/// STRICTLY REJECTS overlong encodings (e.g. 0 encoded as 0x80 0x00).
pub fn decode_varint(slice: &[u8], offset: &mut usize) -> Result<u64, CodecError> {
    let start_offset = *offset;
    let mut result: u64 = 0;
    let mut shift: u32 = 0;
    let mut byte_count = 0;

    loop {
        if *offset >= slice.len() {
            return Err(CodecError::UnexpectedEof);
        }

        let byte = slice[*offset];
        *offset += 1;
        byte_count += 1;

        if byte_count > 10 {
            return Err(CodecError::NonCanonicalEncoding);
        }

        let val = (byte & 0x7F) as u64;
        result |= val << shift;

        if (byte & 0x80) == 0 {
            // Check non-canonical overlong encoding rule:
            // If more than 1 byte was used, the top 7 bits of result must not be 0 in the last byte,
            // otherwise it could have been encoded in fewer bytes.
            if byte_count > 1 && val == 0 {
                return Err(CodecError::NonCanonicalEncoding);
            }
            break;
        }

        shift += 7;
        if shift >= 64 {
            return Err(CodecError::NonCanonicalEncoding);
        }
    }

    // Verify minimal representation constraint
    let mut reencoded = Vec::with_capacity(byte_count);
    encode_varint(result, &mut reencoded);
    if reencoded.len() != byte_count || reencoded.as_slice() != &slice[start_offset..*offset] {
        return Err(CodecError::NonCanonicalEncoding);
    }

    Ok(result)
}

pub fn encode_bytes_bounded(data: &[u8], out: &mut Vec<u8>) {
    encode_varint(data.len() as u64, out);
    out.extend_from_slice(data);
}

pub fn decode_bytes_bounded(
    slice: &[u8],
    offset: &mut usize,
    max_len: usize,
) -> Result<Vec<u8>, CodecError> {
    let len = decode_varint(slice, offset)? as usize;
    if len > max_len {
        return Err(CodecError::BoundedLengthExceeded);
    }
    if *offset + len > slice.len() {
        return Err(CodecError::UnexpectedEof);
    }

    let bytes = slice[*offset..*offset + len].to_vec();
    *offset += len;
    Ok(bytes)
}

pub fn encode_str_bounded(s: &str, out: &mut Vec<u8>) {
    encode_bytes_bounded(s.as_bytes(), out);
}

pub fn decode_str_bounded(
    slice: &[u8],
    offset: &mut usize,
    max_len: usize,
) -> Result<String, CodecError> {
    let bytes = decode_bytes_bounded(slice, offset, max_len)?;
    String::from_utf8(bytes).map_err(|_| CodecError::InvalidUtf8)
}

/// Strict exact buffer decoder guard checking zero trailing bytes.
pub fn decode_exact<T, F>(slice: &[u8], decode_fn: F) -> Result<T, CodecError>
where
    F: FnOnce(&[u8], &mut usize) -> Result<T, CodecError>,
{
    let mut offset = 0;
    let res = decode_fn(slice, &mut offset)?;
    if offset != slice.len() {
        return Err(CodecError::TrailingBytes);
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_canonical_roundtrip() {
        let values = [0, 1, 127, 128, 255, 16384, u64::MAX];
        for &val in &values {
            let mut buf = Vec::new();
            encode_varint(val, &mut buf);
            let mut offset = 0;
            let decoded = decode_varint(&buf, &mut offset).unwrap();
            assert_eq!(decoded, val);
            assert_eq!(offset, buf.len());
        }
    }

    #[test]
    fn test_rejection_of_non_canonical_overlong_varints() {
        // Overlong encoding of 0 as [0x80, 0x00]
        let overlong_zero = vec![0x80, 0x00];
        let mut offset = 0;
        let res = decode_varint(&overlong_zero, &mut offset);
        assert_eq!(res, Err(CodecError::NonCanonicalEncoding));
    }

    #[test]
    fn test_rejection_of_trailing_bytes() {
        let mut buf = Vec::new();
        encode_str_bounded("valid_str", &mut buf);
        buf.push(0xFF); // Trailing garbage byte

        let res = decode_exact(&buf, |s, off| decode_str_bounded(s, off, DEFAULT_MAX_BOUND));
        assert_eq!(res, Err(CodecError::TrailingBytes));
    }

    #[test]
    fn test_rejection_exceeding_max_bound() {
        let mut buf = Vec::new();
        encode_varint(1000, &mut buf);
        buf.resize(1005, 0);

        let mut offset = 0;
        let res = decode_bytes_bounded(&buf, &mut offset, 500); // max_bound = 500 < 1000
        assert_eq!(res, Err(CodecError::BoundedLengthExceeded));
    }
}
