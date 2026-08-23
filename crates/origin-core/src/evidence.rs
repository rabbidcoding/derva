// INVARIANT: Primary observations vs derived info strictly separated; no double counting by lineage duplication.
// KPI: 100% of derivations retain original roots of provenance.

use crate::orid::ORID;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SupportKind {
    Primary { raw_orid: ORID },
    Derived { rule_id: String, parents: Vec<ORID> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceRecord {
    pub id: ORID,
    pub support: SupportKind,
    pub source_domain: String,
}

impl EvidenceRecord {
    pub fn new_primary(id: ORID, raw_orid: ORID, source_domain: impl Into<String>) -> Self {
        Self {
            id,
            support: SupportKind::Primary { raw_orid },
            source_domain: source_domain.into(),
        }
    }

    pub fn new_derived(
        id: ORID,
        rule_id: impl Into<String>,
        parents: Vec<ORID>,
        source_domain: impl Into<String>,
    ) -> Self {
        Self {
            id,
            support: SupportKind::Derived {
                rule_id: rule_id.into(),
                parents,
            },
            source_domain: source_domain.into(),
        }
    }

    /// Recursively computes all unique primary root ORIDs in the lineage graph.
    pub fn resolve_primary_roots(&self, graph: &HashMap<ORID, EvidenceRecord>) -> HashSet<ORID> {
        let mut roots = HashSet::new();
        let mut visited = HashSet::new();
        self.collect_roots_recursive(graph, &mut roots, &mut visited);
        roots
    }

    fn collect_roots_recursive(
        &self,
        graph: &HashMap<ORID, EvidenceRecord>,
        roots: &mut HashSet<ORID>,
        visited: &mut HashSet<ORID>,
    ) {
        if !visited.insert(self.id) {
            return; // Prevent cyclic graph infinite loops
        }

        match &self.support {
            SupportKind::Primary { raw_orid } => {
                roots.insert(*raw_orid);
            }
            SupportKind::Derived { parents, .. } => {
                for parent_id in parents {
                    if let Some(parent_record) = graph.get(parent_id) {
                        parent_record.collect_roots_recursive(graph, roots, visited);
                    }
                }
            }
        }
    }

    /// Computes the number of independent primary roots (anti-amplification invariant).
    pub fn independent_root_count(&self, graph: &HashMap<ORID, EvidenceRecord>) -> usize {
        self.resolve_primary_roots(graph).len()
    }
}

/// Validates whether a target evidence record has a complete, valid path to at least one Primary root.
pub fn is_verified_path_valid(graph: &HashMap<ORID, EvidenceRecord>, target_id: &ORID) -> bool {
    if let Some(target) = graph.get(target_id) {
        !target.resolve_primary_roots(graph).is_empty()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orid::ObjectKind;

    #[test]
    fn test_anti_amplification_100_copies_count_as_one_root() {
        let mut graph = HashMap::new();

        let primary_raw = ORID::compute(ObjectKind::Evidence, b"primary_sensor_data");
        let primary_id = ORID::compute(ObjectKind::Evidence, b"primary_record");

        let primary_rec = EvidenceRecord::new_primary(primary_id, primary_raw, "sensor_net");
        graph.insert(primary_id, primary_rec);

        // Derive 100 copies from the single primary record
        let mut last_id = primary_id;
        for i in 0..100 {
            let derived_id = ORID::compute(ObjectKind::Evidence, format!("copy_{}", i).as_bytes());
            let derived_rec = EvidenceRecord::new_derived(
                derived_id,
                "replication_rule",
                vec![last_id],
                "derived_net",
            );
            graph.insert(derived_id, derived_rec);
            last_id = derived_id;
        }

        // Even after 100 chained derivations, the independent root count MUST equal 1
        let final_derived = graph.get(&last_id).unwrap();
        assert_eq!(final_derived.independent_root_count(&graph), 1);
        assert!(is_verified_path_valid(&graph, &last_id));
    }

    #[test]
    fn test_zero_verified_without_valid_primary_path() {
        let mut graph = HashMap::new();

        let rootless_id = ORID::compute(ObjectKind::Evidence, b"rootless_record");
        let missing_parent_id = ORID::compute(ObjectKind::Evidence, b"missing_parent");

        let rootless_rec = EvidenceRecord::new_derived(
            rootless_id,
            "ungrounded_rule",
            vec![missing_parent_id],
            "invalid_net",
        );
        graph.insert(rootless_id, rootless_rec);

        // Path validation MUST fail when no Primary root can be resolved
        assert!(!is_verified_path_valid(&graph, &rootless_id));
    }
}
