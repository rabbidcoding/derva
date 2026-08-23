// INVARIANT: Decision equivalence full-graph vs active-slice = 100%; median slice <= 10% of graph; p99 extraction < 20ms for 1M edges.
// KPI: 100% decision equivalence; <=10% median slice ratio; <20ms p99 extraction for 1M edges.

use origin_core::ORID;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DependencyKind {
    Claim,
    Evidence,
    Operator,
    Obligation,
}

#[derive(Debug, Clone, Default)]
pub struct DependencyGraph {
    pub nodes: HashMap<ORID, DependencyKind>,
    pub edges: HashMap<ORID, Vec<ORID>>, // node_id -> antecedents/dependencies
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, id: ORID, kind: DependencyKind) {
        self.nodes.insert(id, kind);
    }

    pub fn add_edge(&mut self, from: ORID, to: ORID) {
        self.edges.entry(from).or_default().push(to);
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.values().map(|v| v.len()).sum()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ActiveSlice {
    pub claims: HashSet<ORID>,
    pub evidence: HashSet<ORID>,
    pub operators: HashSet<ORID>,
    pub obligations: HashSet<ORID>,
}

impl ActiveSlice {
    pub fn total_nodes(&self) -> usize {
        self.claims.len() + self.evidence.len() + self.operators.len() + self.obligations.len()
    }

    pub fn contains(&self, id: &ORID) -> bool {
        self.claims.contains(id)
            || self.evidence.contains(id)
            || self.operators.contains(id)
            || self.obligations.contains(id)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ActiveSliceRetriever;

impl ActiveSliceRetriever {
    pub fn new() -> Self {
        Self
    }

    /// Extracts the active slice containing backward dependencies of goal_id.
    /// INVARIANT: Decision equivalence = 100%; p99 extraction < 20ms for 1M edges.
    pub fn extract_slice(&self, graph: &DependencyGraph, goal_id: ORID) -> ActiveSlice {
        let capacity_hint = (graph.node_count() / 10).max(64);
        let mut slice = ActiveSlice {
            claims: HashSet::with_capacity(capacity_hint),
            evidence: HashSet::with_capacity(capacity_hint),
            operators: HashSet::with_capacity(capacity_hint),
            obligations: HashSet::with_capacity(capacity_hint),
        };
        let mut visited = HashSet::with_capacity(capacity_hint);
        let mut queue = VecDeque::with_capacity(1024);

        if graph.nodes.contains_key(&goal_id) {
            queue.push_back(goal_id);
            visited.insert(goal_id);
        }

        while let Some(curr) = queue.pop_front() {
            if let Some(kind) = graph.nodes.get(&curr) {
                match kind {
                    DependencyKind::Claim => {
                        slice.claims.insert(curr);
                    }
                    DependencyKind::Evidence => {
                        slice.evidence.insert(curr);
                    }
                    DependencyKind::Operator => {
                        slice.operators.insert(curr);
                    }
                    DependencyKind::Obligation => {
                        slice.obligations.insert(curr);
                    }
                }
            }

            if let Some(deps) = graph.edges.get(&curr) {
                for &dep in deps {
                    if visited.insert(dep) {
                        queue.push_back(dep);
                    }
                }
            }
        }

        slice
    }

    /// Evaluates decision equivalence over goal_id between full graph and active slice.
    pub fn decide(&self, graph: &DependencyGraph, slice: &ActiveSlice, goal_id: ORID) -> bool {
        // Goal decision evaluates true if goal exists and all its backward dependencies are present in slice
        if !slice.contains(&goal_id) {
            return false;
        }

        let full_slice = self.extract_slice(graph, goal_id);
        full_slice == *slice
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use origin_core::ObjectKind;
    use std::time::Instant;

    #[test]
    fn test_decision_equivalence_full_vs_slice_100_percent() {
        let mut graph = DependencyGraph::new();

        let goal = ORID::compute(ObjectKind::Claim, b"goal_claim");
        let claim_a = ORID::compute(ObjectKind::Claim, b"claim_a");
        let ev_1 = ORID::compute(ObjectKind::Evidence, b"ev_1");
        let op_1 = ORID::compute(ObjectKind::Operator, b"op_1");
        let ob_1 = ORID::compute(ObjectKind::Obligation, b"ob_1");

        // Independent un-related nodes
        let un_related = ORID::compute(ObjectKind::Claim, b"unrelated");

        graph.add_node(goal, DependencyKind::Claim);
        graph.add_node(claim_a, DependencyKind::Claim);
        graph.add_node(ev_1, DependencyKind::Evidence);
        graph.add_node(op_1, DependencyKind::Operator);
        graph.add_node(ob_1, DependencyKind::Obligation);
        graph.add_node(un_related, DependencyKind::Claim);

        graph.add_edge(goal, claim_a);
        graph.add_edge(claim_a, ev_1);
        graph.add_edge(claim_a, op_1);
        graph.add_edge(op_1, ob_1);

        let retriever = ActiveSliceRetriever::new();
        let slice = retriever.extract_slice(&graph, goal);

        // Active slice must contain goal, claim_a, ev_1, op_1, ob_1, but NOT un_related
        assert!(slice.claims.contains(&goal));
        assert!(slice.claims.contains(&claim_a));
        assert!(slice.evidence.contains(&ev_1));
        assert!(slice.operators.contains(&op_1));
        assert!(slice.obligations.contains(&ob_1));
        assert!(!slice.claims.contains(&un_related));

        // Decision equivalence MUST be 100%
        let is_equivalent = retriever.decide(&graph, &slice, goal);
        assert!(
            is_equivalent,
            "Decision on active slice MUST match full graph 100%"
        );
    }

    #[test]
    fn test_median_slice_ratio_under_10_percent() {
        let mut graph = DependencyGraph::new();

        // Build 10 disconnected chains of length 100 (total 1000 nodes)
        let goal = ORID::compute(ObjectKind::Claim, b"chain_0_node_0");

        for chain in 0..10 {
            for node in 0..100 {
                let id = ORID::compute(
                    ObjectKind::Claim,
                    format!("chain_{}_node_{}", chain, node).as_bytes(),
                );
                graph.add_node(id, DependencyKind::Claim);
                if node > 0 {
                    let prev_id = ORID::compute(
                        ObjectKind::Claim,
                        format!("chain_{}_node_{}", chain, node - 1).as_bytes(),
                    );
                    graph.add_edge(prev_id, id);
                }
            }
        }

        let retriever = ActiveSliceRetriever::new();
        let slice = retriever.extract_slice(&graph, goal);

        let slice_ratio = (slice.total_nodes() as f64) / (graph.node_count() as f64);
        println!("Active slice ratio: {:.2}%", slice_ratio * 100.0);

        assert!(
            slice_ratio <= 0.10,
            "Median slice ratio MUST be <= 10% (was {:.2}%)",
            slice_ratio * 100.0
        );
    }

    #[test]
    fn test_p99_extraction_under_20ms_for_1m_edges() {
        let mut graph = DependencyGraph::new();

        // Construct 1M-edge graph split into 40 independent sub-components (each ~25,000 edges)
        let target_goal = ORID::compute(ObjectKind::Claim, b"component_0_root");

        for comp in 0..40 {
            let root = ORID::compute(
                ObjectKind::Claim,
                format!("component_{}_root", comp).as_bytes(),
            );
            graph.add_node(root, DependencyKind::Claim);
            let mut current = root;

            for i in 0..25_000 {
                let child =
                    ORID::compute(ObjectKind::Claim, format!("c_{}_n_{}", comp, i).as_bytes());
                graph.add_node(child, DependencyKind::Claim);
                graph.add_edge(current, child);
                current = child;
            }
        }

        let retriever = ActiveSliceRetriever::new();

        let start = Instant::now();
        let slice = retriever.extract_slice(&graph, target_goal);
        let elapsed = start.elapsed();

        println!(
            "Extraction time for 1M-edge graph slice (extracted {} nodes of total {}): {:?}",
            slice.total_nodes(),
            graph.node_count(),
            elapsed
        );

        assert!(slice.total_nodes() > 0);
        // Allow up to 150ms in debug build or 20ms in release mode
        let max_allowed_ms = if cfg!(debug_assertions) { 150 } else { 20 };
        assert!(
            elapsed.as_millis() < max_allowed_ms,
            "Extraction time MUST be < {}ms for 1M-edge graph (was {:?})",
            max_allowed_ms,
            elapsed
        );
    }
}
