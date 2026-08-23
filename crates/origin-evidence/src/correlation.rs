// INVARIANT: Multiplicity of correlated or derived evidence items never inflates independent trust count.
// KPI: 100 copies / 1 root => independent_root_count = 1; False merge < 1%; Status amplification via multiplicity prohibited.

use crate::evidence::EvidenceRecord;
use crate::provenance::ProvenanceHypergraph;
use origin_core::ORID;
use std::collections::HashSet;

#[derive(Debug, Default, Clone)]
pub struct CorrelationDeduplicator;

impl CorrelationDeduplicator {
    pub fn new() -> Self {
        Self
    }

    /// Deduplicates evidence records by inspecting lineage roots via ProvenanceHypergraph and correlation domains.
    /// Returns the distinct set of independent root ORIDs.
    pub fn independent_roots(
        &self,
        provenance: &ProvenanceHypergraph,
        evidence_set: &[EvidenceRecord],
    ) -> Vec<ORID> {
        let mut independent_set = HashSet::new();

        for ev in evidence_set {
            let roots = match provenance.why(&ev.id()) {
                Ok(proof) => proof.roots,
                Err(_) => {
                    // Fallback to raw ORID or evidence record ID
                    vec![ev.raw_object_orid.unwrap_or_else(|| ev.id())]
                }
            };

            for root in roots {
                // Incorporate domain grouping key
                independent_set.insert(root);
            }
        }

        let mut result: Vec<ORID> = independent_set.into_iter().collect();
        result.sort_by_key(|a| a.hash);
        result
    }

    /// Calculates independent support count. 100 copies deriving from 1 root return count == 1.
    pub fn independent_support_count(
        &self,
        provenance: &ProvenanceHypergraph,
        evidence_set: &[EvidenceRecord],
    ) -> usize {
        let roots = self.independent_roots(provenance, evidence_set);

        // Group by correlation domain if present
        let mut domain_set = HashSet::new();
        for ev in evidence_set {
            domain_set.insert(&ev.correlation_domain);
        }

        // The effective independent count cannot exceed the distinct root count
        std::cmp::min(roots.len(), domain_set.len().max(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::EvidenceRecord;
    use crate::provenance::{Derivation, ProvenanceHypergraph};
    use origin_core::ObjectKind;

    #[test]
    fn test_100_copies_one_root_yields_independent_count_one() {
        let mut provenance = ProvenanceHypergraph::new();
        let deduplicator = CorrelationDeduplicator::new();

        let root_orid = ORID::compute(ObjectKind::Evidence, b"single_truth_root");

        let mut evidence_set = Vec::new();
        let mut last_orid = root_orid;

        // Generate 100 derived/copied evidence records from 1 single root
        for i in 1..=100 {
            let record = EvidenceRecord::new(
                Some(root_orid),
                format!("source_copy_{}", i),
                "ingest::mirror",
                1700000000 + i as u64,
                "shared_correlation_domain_a",
                "trust_medium",
            )
            .unwrap();

            let current_orid = record.id();
            provenance
                .insert_derivation(Derivation {
                    rule_id: "copy_rule".to_string(),
                    parents: vec![last_orid],
                    child: current_orid,
                    transformation_id: "copy".to_string(),
                })
                .unwrap();

            last_orid = current_orid;
            evidence_set.push(record);
        }

        let roots = deduplicator.independent_roots(&provenance, &evidence_set);
        let support_count = deduplicator.independent_support_count(&provenance, &evidence_set);

        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0], root_orid);
        assert_eq!(
            support_count, 1,
            "100 copies must yield exactly independent_support_count == 1"
        );
    }

    #[test]
    fn test_false_merge_rate_below_one_percent() {
        let provenance = ProvenanceHypergraph::new();
        let deduplicator = CorrelationDeduplicator::new();

        let mut evidence_set = Vec::new();

        // 1,000 genuinely distinct independent evidence records
        for i in 1..=1000 {
            let root_orid = ORID::compute(
                ObjectKind::Evidence,
                format!("distinct_root_{}", i).as_bytes(),
            );
            let record = EvidenceRecord::new(
                Some(root_orid),
                format!("distinct_source_{}", i),
                "ingest::direct",
                1700000000 + i as u64,
                format!("domain_{}", i),
                "trust_high",
            )
            .unwrap();
            evidence_set.push(record);
        }

        let count = deduplicator.independent_support_count(&provenance, &evidence_set);
        assert_eq!(count, 1000); // 0 false merges across 1000 distinct items
    }
}
