// INVARIANT: Evidence without raw ORID cannot support VERIFIED status; malformed timestamp/method causes immediate rejection.
// KPI: 100% mandatory fields present; 0 evidence objects without raw ORID promoted to VERIFIED.

use origin_core::{Canonical, ObjectKind, ORID};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceError {
    Malformed(String),
    MissingField(&'static str),
    UntrustedDomain(String),
}

impl std::fmt::Display for EvidenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvidenceError::Malformed(msg) => write!(f, "Malformed evidence payload: {}", msg),
            EvidenceError::MissingField(field) => {
                write!(f, "Missing mandatory evidence field: {}", field)
            }
            EvidenceError::UntrustedDomain(domain) => write!(f, "Untrusted domain: {}", domain),
        }
    }
}

impl std::error::Error for EvidenceError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceRecord {
    pub raw_object_orid: Option<ORID>,
    pub source_identity: String,
    pub acquisition_method: String,
    pub timestamp: u64,
    pub correlation_domain: String,
    pub trust_domain: String,
}

impl EvidenceRecord {
    pub fn new(
        raw_object_orid: Option<ORID>,
        source_identity: impl Into<String>,
        acquisition_method: impl Into<String>,
        timestamp: u64,
        correlation_domain: impl Into<String>,
        trust_domain: impl Into<String>,
    ) -> Result<Self, EvidenceError> {
        let record = Self {
            raw_object_orid,
            source_identity: source_identity.into(),
            acquisition_method: acquisition_method.into(),
            timestamp,
            correlation_domain: correlation_domain.into(),
            trust_domain: trust_domain.into(),
        };

        record.validate()?;
        Ok(record)
    }

    pub fn validate(&self) -> Result<(), EvidenceError> {
        if self.source_identity.trim().is_empty() {
            return Err(EvidenceError::MissingField("source_identity"));
        }
        if self.acquisition_method.trim().is_empty() {
            return Err(EvidenceError::MissingField("acquisition_method"));
        }
        if self.correlation_domain.trim().is_empty() {
            return Err(EvidenceError::MissingField("correlation_domain"));
        }
        if self.trust_domain.trim().is_empty() {
            return Err(EvidenceError::MissingField("trust_domain"));
        }
        if self.timestamp == 0 {
            return Err(EvidenceError::Malformed(
                "Timestamp cannot be zero".to_string(),
            ));
        }
        if !self.acquisition_method.contains("::") && !self.acquisition_method.contains('/') {
            return Err(EvidenceError::Malformed(format!(
                "Acquisition method '{}' lacks valid namespace qualification",
                self.acquisition_method
            )));
        }

        Ok(())
    }

    /// Evaluates if this evidence object is eligible to support `Status::Verified`.
    /// INVARIANT: Evidence without raw_object_orid CANNOT support VERIFIED status.
    pub fn can_support_verified(&self) -> bool {
        if self.raw_object_orid.is_none() {
            return false;
        }
        self.validate().is_ok()
    }

    pub fn id(&self) -> ORID {
        let mut buf = Vec::new();
        self.encode_canonical(&mut buf);
        ORID::compute(ObjectKind::Evidence, &buf)
    }
}

impl Canonical for EvidenceRecord {
    fn encode_canonical(&self, out: &mut Vec<u8>) {
        match &self.raw_object_orid {
            Some(orid) => {
                out.push(1);
                out.extend_from_slice(&orid.hash);
            }
            None => out.push(0),
        }

        let s_bytes = self.source_identity.as_bytes();
        out.extend_from_slice(&(s_bytes.len() as u64).to_be_bytes());
        out.extend_from_slice(s_bytes);

        let m_bytes = self.acquisition_method.as_bytes();
        out.extend_from_slice(&(m_bytes.len() as u64).to_be_bytes());
        out.extend_from_slice(m_bytes);

        out.extend_from_slice(&self.timestamp.to_be_bytes());

        let c_bytes = self.correlation_domain.as_bytes();
        out.extend_from_slice(&(c_bytes.len() as u64).to_be_bytes());
        out.extend_from_slice(c_bytes);

        let t_bytes = self.trust_domain.as_bytes();
        out.extend_from_slice(&(t_bytes.len() as u64).to_be_bytes());
        out.extend_from_slice(t_bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_without_raw_orid_cannot_support_verified() {
        let ev = EvidenceRecord::new(
            None,
            "sensor_node_1",
            "ingest::direct",
            1700000000,
            "domain_a",
            "trust_high",
        )
        .unwrap();

        assert!(!ev.can_support_verified());
    }

    #[test]
    fn test_evidence_with_raw_orid_can_support_verified() {
        let raw_orid = ORID::compute(ObjectKind::Claim, b"raw_observation_data");
        let ev = EvidenceRecord::new(
            Some(raw_orid),
            "sensor_node_1",
            "ingest::direct",
            1700000000,
            "domain_a",
            "trust_high",
        )
        .unwrap();

        assert!(ev.can_support_verified());
    }

    #[test]
    fn test_malformed_timestamp_or_method_rejected() {
        // Zero timestamp rejected
        assert!(
            EvidenceRecord::new(None, "sensor", "ingest::direct", 0, "domain", "trust").is_err()
        );

        // Unqualified method rejected
        assert!(
            EvidenceRecord::new(None, "sensor", "unqualifiedmethod", 100, "domain", "trust")
                .is_err()
        );
    }
}
