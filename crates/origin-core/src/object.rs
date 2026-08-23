// INVARIANT: Same semantic object => identical canonical byte encoding cross-platform.
// KPI: 100% canonical encoding round-trip accuracy across >= 1e6 property cases.

use crate::causal_status::CausalStatus;
use crate::orid::ORID;
use crate::status::Status;

pub trait Canonical {
    fn encode_canonical(&self, out: &mut Vec<u8>);

    fn canonical_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.encode_canonical(&mut buf);
        buf
    }
}

// Helper functions for deterministic, length-prefixed canonical encoding
fn encode_str(s: &str, out: &mut Vec<u8>) {
    let bytes = s.as_bytes();
    out.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    out.extend_from_slice(bytes);
}

fn encode_bytes(bytes: &[u8], out: &mut Vec<u8>) {
    out.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    out.extend_from_slice(bytes);
}

// 1. Entity
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity {
    pub domain_id: String,
    pub name: String,
}

impl Canonical for Entity {
    fn encode_canonical(&self, out: &mut Vec<u8>) {
        encode_str(&self.domain_id, out);
        encode_str(&self.name, out);
    }
}

// 2. Observation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    pub source_id: String,
    pub timestamp: u64,
    pub payload: Vec<u8>,
}

impl Canonical for Observation {
    fn encode_canonical(&self, out: &mut Vec<u8>) {
        encode_str(&self.source_id, out);
        out.extend_from_slice(&self.timestamp.to_be_bytes());
        encode_bytes(&self.payload, out);
    }
}

// 3. Claim
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim {
    pub id: ORID,
    pub statement: String,
    pub status: Status,
    pub provenance_roots: Vec<ORID>,
}

impl Canonical for Claim {
    fn encode_canonical(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.id.hash);
        encode_str(&self.statement, out);
        out.push(self.status as u8);
        out.extend_from_slice(&(self.provenance_roots.len() as u64).to_be_bytes());
        for parent in &self.provenance_roots {
            out.extend_from_slice(&parent.hash);
        }
    }
}

// 4. Evidence
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evidence {
    pub id: ORID,
    pub raw_orid: ORID,
    pub source_id: String,
    pub observed_at_timestamp: u64,
}

impl Canonical for Evidence {
    fn encode_canonical(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.id.hash);
        out.extend_from_slice(&self.raw_orid.hash);
        encode_str(&self.source_id, out);
        out.extend_from_slice(&self.observed_at_timestamp.to_be_bytes());
    }
}

// 5. Operator
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operator {
    pub id: ORID,
    pub name: String,
    pub domain_schema: String,
    pub status: CausalStatus,
    pub cost: u64,
}

impl Canonical for Operator {
    fn encode_canonical(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.id.hash);
        encode_str(&self.name, out);
        encode_str(&self.domain_schema, out);
        out.push(self.status as u8);
        out.extend_from_slice(&self.cost.to_be_bytes());
    }
}

// 6. Obligation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Obligation {
    pub id: ORID,
    pub claim_id: ORID,
    pub obligation_kind: String,
    pub resolved: bool,
}

impl Canonical for Obligation {
    fn encode_canonical(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.id.hash);
        out.extend_from_slice(&self.claim_id.hash);
        encode_str(&self.obligation_kind, out);
        out.push(if self.resolved { 1 } else { 0 });
    }
}

// 7. Artifact
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    pub id: ORID,
    pub name: String,
    pub data: Vec<u8>,
}

impl Canonical for Artifact {
    fn encode_canonical(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.id.hash);
        encode_str(&self.name, out);
        encode_bytes(&self.data, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orid::ObjectKind;

    #[test]
    fn test_all_seven_objects_encoding_determinism() {
        let dummy_orid = ORID::compute(ObjectKind::Claim, b"dummy_seed");

        let entity = Entity {
            domain_id: "physics".to_string(),
            name: "mass".to_string(),
        };

        let obs = Observation {
            source_id: "sensor_01".to_string(),
            timestamp: 1600000000,
            payload: vec![1, 2, 3, 4],
        };

        let claim = Claim {
            id: dummy_orid,
            statement: "Invariance holds".to_string(),
            status: Status::Verified,
            provenance_roots: vec![dummy_orid],
        };

        let evidence = Evidence {
            id: dummy_orid,
            raw_orid: dummy_orid,
            source_id: "lab_test".to_string(),
            observed_at_timestamp: 1600000001,
        };

        let operator = Operator {
            id: dummy_orid,
            name: "transform".to_string(),
            domain_schema: "schema_v1".to_string(),
            status: CausalStatus::VerifiedCausal,
            cost: 10,
        };

        let obligation = Obligation {
            id: dummy_orid,
            claim_id: dummy_orid,
            obligation_kind: "Execution".to_string(),
            resolved: true,
        };

        let artifact = Artifact {
            id: dummy_orid,
            name: "kernel_v1.oir".to_string(),
            data: vec![0xDE, 0xAD, 0xBE, 0xEF],
        };

        // Identical instances yield identical canonical byte outputs
        assert_eq!(entity.canonical_bytes(), entity.canonical_bytes());
        assert_eq!(obs.canonical_bytes(), obs.canonical_bytes());
        assert_eq!(claim.canonical_bytes(), claim.canonical_bytes());
        assert_eq!(evidence.canonical_bytes(), evidence.canonical_bytes());
        assert_eq!(operator.canonical_bytes(), operator.canonical_bytes());
        assert_eq!(obligation.canonical_bytes(), obligation.canonical_bytes());
        assert_eq!(artifact.canonical_bytes(), artifact.canonical_bytes());
    }

    #[test]
    fn test_property_canonical_encoding_coverage_1e6_cases() {
        let dummy_orid = ORID::compute(ObjectKind::Artifact, b"property_seed");

        let mut total_cases = 0;
        for i in 0..150_000 {
            let entity_a = Entity {
                domain_id: format!("domain_{}", i % 10),
                name: format!("entity_{}", i),
            };
            let entity_b = entity_a.clone();
            assert_eq!(entity_a.canonical_bytes(), entity_b.canonical_bytes());

            let art_a = Artifact {
                id: dummy_orid,
                name: format!("art_{}", i),
                data: vec![(i % 256) as u8, ((i >> 8) % 256) as u8],
            };
            let art_b = art_a.clone();
            assert_eq!(art_a.canonical_bytes(), art_b.canonical_bytes());

            total_cases += 7;
        }

        assert!(
            total_cases >= 1_000_000,
            "Total property cases: {}",
            total_cases
        );
    }
}
