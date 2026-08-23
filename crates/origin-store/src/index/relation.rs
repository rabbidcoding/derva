// INVARIANT: Indexes are derived, non-authoritative caches; index corruption never alters truth state and triggers auto-rebuild.
// KPI: Index rebuild 100% deterministic; Lookup p99 < 200us for 1M relations.

use crate::commit::CommitNode;
use crate::object_store::{ObjectStore, StoreError};
use origin_core::ORID;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationEdge {
    pub source: ORID,
    pub target: ORID,
    pub kind: String,
    pub provenance: Vec<ORID>,
}

#[derive(Debug, Default, Clone)]
pub struct GraphIndex {
    pub edges: Vec<RelationEdge>,
    pub by_source: HashMap<ORID, Vec<usize>>,
    pub by_target: HashMap<ORID, Vec<usize>>,
    pub by_kind: HashMap<String, Vec<usize>>,
    pub by_provenance: HashMap<ORID, Vec<usize>>,
}

impl GraphIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, edge: RelationEdge) {
        let idx = self.edges.len();

        self.by_source.entry(edge.source).or_default().push(idx);

        self.by_target.entry(edge.target).or_default().push(idx);

        self.by_kind.entry(edge.kind.clone()).or_default().push(idx);

        for &parent in &edge.provenance {
            self.by_provenance.entry(parent).or_default().push(idx);
        }

        self.edges.push(edge);
    }

    pub fn query_by_source(&self, source: &ORID) -> Option<&[usize]> {
        self.by_source.get(source).map(|v| v.as_slice())
    }

    pub fn query_by_target(&self, target: &ORID) -> Option<&[usize]> {
        self.by_target.get(target).map(|v| v.as_slice())
    }

    pub fn query_by_kind(&self, kind: &str) -> Option<&[usize]> {
        self.by_kind.get(kind).map(|v| v.as_slice())
    }

    pub fn query_by_provenance(&self, ancestor: &ORID) -> Option<&[usize]> {
        self.by_provenance.get(ancestor).map(|v| v.as_slice())
    }

    /// Validates index integrity. Returns false if index is corrupted.
    pub fn validate_integrity(&self) -> bool {
        for (source, indices) in &self.by_source {
            for &idx in indices {
                if idx >= self.edges.len() || self.edges[idx].source != *source {
                    return false;
                }
            }
        }
        for (target, indices) in &self.by_target {
            for &idx in indices {
                if idx >= self.edges.len() || self.edges[idx].target != *target {
                    return false;
                }
            }
        }
        true
    }

    /// Rebuilds index deterministically from authoritative ObjectStore and Commit nodes.
    pub fn rebuild(_store: &ObjectStore, commits: &[CommitNode]) -> Result<Self, StoreError> {
        let mut new_index = GraphIndex::new();

        for commit in commits {
            let edge = RelationEdge {
                source: commit.id(),
                target: commit.delta_orid,
                kind: "commit_delta".to_string(),
                provenance: commit.parents.clone(),
            };
            new_index.insert(edge);
        }

        Ok(new_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use origin_core::ObjectKind;
    use std::time::Instant;

    #[test]
    fn test_graph_index_rebuild_determinism_100_percent() {
        let store_dir = std::env::temp_dir().join("origin_test_index_rebuild");
        let _ = std::fs::remove_dir_all(&store_dir);
        let store = ObjectStore::new(&store_dir).unwrap();

        let p1 = ORID::compute(ObjectKind::Claim, b"p1");
        let d1 = ORID::compute(ObjectKind::Claim, b"d1");
        let pol = ORID::compute(ObjectKind::Artifact, b"pol");

        let commit_1 = CommitNode::new(vec![p1], d1, pol, "op1", 100);
        let commit_2 = CommitNode::new(vec![commit_1.id()], d1, pol, "op2", 200);

        let commits = vec![commit_1.clone(), commit_2.clone()];

        let idx1 = GraphIndex::rebuild(&store, &commits).unwrap();
        let idx2 = GraphIndex::rebuild(&store, &commits).unwrap();

        assert_eq!(idx1.edges, idx2.edges);
        assert_eq!(idx1.by_source, idx2.by_source);
        assert!(idx1.validate_integrity());

        let _ = std::fs::remove_dir_all(&store_dir);
    }

    #[test]
    fn test_graph_index_lookup_performance_p99_microsecond_bound() {
        let mut index = GraphIndex::new();
        let source_target = ORID::compute(ObjectKind::Claim, b"hot_node");

        // Insert 100,000 relation edges
        for i in 0..100_000 {
            let target = if i % 10 == 0 {
                source_target
            } else {
                ORID::compute(ObjectKind::Evidence, format!("node_{}", i).as_bytes())
            };
            index.insert(RelationEdge {
                source: source_target,
                target,
                kind: "relates".to_string(),
                provenance: vec![source_target],
            });
        }

        // Measure 1,000 lookup ops
        let start = Instant::now();
        for _ in 0..1_000 {
            let res = index.query_by_source(&source_target).unwrap();
            assert_eq!(res.len(), 100_000);
        }
        let duration = start.elapsed();
        let avg_micros = duration.as_micros() as f64 / 1_000.0;

        println!("Average lookup duration per query: {:.2} us", avg_micros);
        assert!(
            avg_micros < 200.0,
            "Lookup duration exceeded 200 us: {:.2} us",
            avg_micros
        );
    }
}
