// INVARIANT: Content addressing by ORID (Origin Resource Identifier) with domain separation.

use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectKind {
    Claim,
    Evidence,
    Operator,
    Obligation,
    Commit,
    Artifact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ORID {
    pub kind: ObjectKind,
    pub hash: [u8; 32],
}

impl ORID {
    pub fn compute(kind: ObjectKind, canonical_bytes: &[u8]) -> Self {
        let domain_prefix: &[u8] = match kind {
            ObjectKind::Claim => b"origin:claim:v1\0",
            ObjectKind::Evidence => b"origin:evidence:v1\0",
            ObjectKind::Operator => b"origin:operator:v1\0",
            ObjectKind::Obligation => b"origin:obligation:v1\0",
            ObjectKind::Commit => b"origin:commit:v1\0",
            ObjectKind::Artifact => b"origin:artifact:v1\0",
        };
        let mut hasher = Sha256::new();
        hasher.update(domain_prefix);
        hasher.update(canonical_bytes);
        let hash: [u8; 32] = hasher.finalize().into();
        ORID { kind, hash }
    }
}

impl fmt::Display for ORID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "orid:{:?}:", self.kind)?;
        for byte in &self.hash[..8] {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}
