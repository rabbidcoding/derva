// INVARIANT: Content addressing by ORID (Origin Resource Identifier) with domain separation.
// KPI: 0 collisions in synthetic high-volume tests; 100% type-domain separation; 100% string parse/format round-trip.

use sha2::{Digest, Sha256};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectKind {
    Entity,
    Observation,
    Claim,
    Evidence,
    Operator,
    Obligation,
    Commit,
    Artifact,
}

impl ObjectKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ObjectKind::Entity => "Entity",
            ObjectKind::Observation => "Observation",
            ObjectKind::Claim => "Claim",
            ObjectKind::Evidence => "Evidence",
            ObjectKind::Operator => "Operator",
            ObjectKind::Obligation => "Obligation",
            ObjectKind::Commit => "Commit",
            ObjectKind::Artifact => "Artifact",
        }
    }

    pub fn domain_prefix(&self) -> &'static [u8] {
        match self {
            ObjectKind::Entity => b"origin:entity:v1\0",
            ObjectKind::Observation => b"origin:observation:v1\0",
            ObjectKind::Claim => b"origin:claim:v1\0",
            ObjectKind::Evidence => b"origin:evidence:v1\0",
            ObjectKind::Operator => b"origin:operator:v1\0",
            ObjectKind::Obligation => b"origin:obligation:v1\0",
            ObjectKind::Commit => b"origin:commit:v1\0",
            ObjectKind::Artifact => b"origin:artifact:v1\0",
        }
    }
}

impl FromStr for ObjectKind {
    type Err = OridParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Entity" => Ok(ObjectKind::Entity),
            "Observation" => Ok(ObjectKind::Observation),
            "Claim" => Ok(ObjectKind::Claim),
            "Evidence" => Ok(ObjectKind::Evidence),
            "Operator" => Ok(ObjectKind::Operator),
            "Obligation" => Ok(ObjectKind::Obligation),
            "Commit" => Ok(ObjectKind::Commit),
            "Artifact" => Ok(ObjectKind::Artifact),
            _ => Err(OridParseError::InvalidKind),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ORID {
    pub kind: ObjectKind,
    pub hash: [u8; 32],
}

impl ORID {
    pub fn compute(kind: ObjectKind, canonical_bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(kind.domain_prefix());
        hasher.update(canonical_bytes);
        let hash: [u8; 32] = hasher.finalize().into();
        ORID { kind, hash }
    }
}

impl fmt::Display for ORID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "orid:{}:", self.kind.as_str())?;
        for byte in &self.hash {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OridParseError {
    InvalidPrefix,
    InvalidKind,
    InvalidHashHex,
}

impl fmt::Display for OridParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OridParseError::InvalidPrefix => write!(f, "ORID must start with 'orid:'"),
            OridParseError::InvalidKind => write!(f, "Invalid ORID ObjectKind"),
            OridParseError::InvalidHashHex => write!(f, "Invalid 64-character SHA256 hex string"),
        }
    }
}

impl std::error::Error for OridParseError {}

impl FromStr for ORID {
    type Err = OridParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 3 || parts[0] != "orid" {
            return Err(OridParseError::InvalidPrefix);
        }

        let kind = ObjectKind::from_str(parts[1])?;
        let hex_str = parts[2];
        if hex_str.len() != 64 {
            return Err(OridParseError::InvalidHashHex);
        }

        let mut hash = [0u8; 32];
        for i in 0..32 {
            hash[i] = u8::from_str_radix(&hex_str[i * 2..i * 2 + 2], 16)
                .map_err(|_| OridParseError::InvalidHashHex)?;
        }

        Ok(ORID { kind, hash })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_orid_type_domain_separation_100_percent() {
        let payload = b"identical_payload_data";

        let orid_claim = ORID::compute(ObjectKind::Claim, payload);
        let orid_evidence = ORID::compute(ObjectKind::Evidence, payload);
        let orid_artifact = ORID::compute(ObjectKind::Artifact, payload);

        // Hashes MUST be distinct even though canonical_bytes are identical!
        assert_ne!(orid_claim.hash, orid_evidence.hash);
        assert_ne!(orid_claim.hash, orid_artifact.hash);
        assert_ne!(orid_evidence.hash, orid_artifact.hash);
    }

    #[test]
    fn test_orid_string_format_and_parse_roundtrip() {
        let original = ORID::compute(ObjectKind::Claim, b"canonical_bytes_test");
        let formatted = original.to_string();

        assert!(formatted.starts_with("orid:Claim:"));
        let parsed: ORID = formatted.parse().unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn test_orid_synthetic_collision_free() {
        let mut set = HashSet::new();
        for i in 0..10_000 {
            let data = format!("synthetic_object_{}", i);
            let orid = ORID::compute(ObjectKind::Claim, data.as_bytes());
            assert!(set.insert(orid), "Collision detected at index {}", i);
        }
    }
}
