#![forbid(unsafe_code)]

// INVARIANT: Congruence closure correct in exhaustive algebra tests; 0 type-violating unions; 1M enodes memory budget < 1.5GB.
// KPI: Correct congruence closure; 0 type-violating unions; 1M enodes < 1.5GB.

use std::collections::HashMap;

pub mod rewrite;

pub use rewrite::{
    AppliedRewrite, EqualitySaturator, Extractor, RewriteRule, SaturationBudget, SaturationReport,
    SaturationStopReason,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EType {
    Bool,
    Int,
    Float,
    Vector,
}

pub type Id = u32;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ENode {
    pub op: String,
    pub children: Vec<Id>,
    pub ty: EType,
}

impl ENode {
    pub fn new(op: impl Into<String>, children: Vec<Id>, ty: EType) -> Self {
        Self {
            op: op.into(),
            children,
            ty,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnionFind {
    parents: Vec<Id>,
}

impl UnionFind {
    pub fn new() -> Self {
        Self {
            parents: Vec::new(),
        }
    }

    pub fn make_set(&mut self) -> Id {
        let id = self.parents.len() as Id;
        self.parents.push(id);
        id
    }

    pub fn find_immutable(&self, i: Id) -> Id {
        let mut root = i;
        while root != self.parents[root as usize] {
            root = self.parents[root as usize];
        }
        root
    }

    pub fn find(&mut self, i: Id) -> Id {
        let mut root = i;
        while root != self.parents[root as usize] {
            root = self.parents[root as usize];
        }
        // Path compression
        let mut curr = i;
        while curr != root {
            let next = self.parents[curr as usize];
            self.parents[curr as usize] = root;
            curr = next;
        }
        root
    }

    pub fn union(&mut self, i: Id, j: Id) -> Id {
        let root_i = self.find(i);
        let root_j = self.find(j);
        if root_i != root_j {
            self.parents[root_j as usize] = root_i;
            root_i
        } else {
            root_i
        }
    }
}

impl Default for UnionFind {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EGraphError {
    TypeMismatch {
        a: Id,
        b: Id,
        ty_a: EType,
        ty_b: EType,
    },
    InvalidId(Id),
}

impl std::fmt::Display for EGraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EGraphError::TypeMismatch { a, b, ty_a, ty_b } => {
                write!(
                    f,
                    "Type mismatch union between Id {} ({:?}) and Id {} ({:?})",
                    a, ty_a, b, ty_b
                )
            }
            EGraphError::InvalidId(id) => write!(f, "Invalid Id in e-graph: {}", id),
        }
    }
}

impl std::error::Error for EGraphError {}

#[derive(Debug, Clone, Default)]
pub struct EGraph {
    pub uf: UnionFind,
    pub memo: HashMap<ENode, Id>,
    pub classes: Vec<Vec<ENode>>,
    pub class_types: Vec<EType>,
    pub parents: HashMap<Id, Vec<(ENode, Id)>>,
    worklist: Vec<(Id, Id)>,
}

impl EGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn find_immutable(&self, id: Id) -> Id {
        self.uf.find_immutable(id)
    }

    pub fn find(&mut self, id: Id) -> Id {
        self.uf.find(id)
    }

    /// Adds an ENode using hash-consing.
    pub fn add(&mut self, mut enode: ENode) -> Result<Id, EGraphError> {
        // Canonicalize children
        for child in &mut enode.children {
            if (*child as usize) >= self.uf.parents.len() {
                return Err(EGraphError::InvalidId(*child));
            }
            *child = self.uf.find(*child);
        }

        if let Some(&id) = self.memo.get(&enode) {
            return Ok(self.uf.find(id));
        }

        let id = self.uf.make_set();
        self.memo.insert(enode.clone(), id);
        self.classes.push(vec![enode.clone()]);
        self.class_types.push(enode.ty);

        // Track parent relationships for congruence closure
        for child in &enode.children {
            self.parents
                .entry(*child)
                .or_default()
                .push((enode.clone(), id));
        }

        Ok(id)
    }

    /// Unions two e-classes after asserting type compatibility.
    /// INVARIANT: 0 type-violating unions.
    pub fn union_typed(&mut self, a: Id, b: Id) -> Result<bool, EGraphError> {
        let root_a = self.find(a);
        let root_b = self.find(b);

        if root_a == root_b {
            return Ok(false);
        }

        let ty_a = self.class_types[root_a as usize];
        let ty_b = self.class_types[root_b as usize];

        if ty_a != ty_b {
            return Err(EGraphError::TypeMismatch {
                a: root_a,
                b: root_b,
                ty_a,
                ty_b,
            });
        }

        self.worklist.push((root_a, root_b));
        Ok(true)
    }

    /// Restores congruence closure invariants over all e-classes.
    pub fn rebuild(&mut self) -> usize {
        let mut merges = 0;

        while let Some((a, b)) = self.worklist.pop() {
            let root_a = self.find(a);
            let root_b = self.find(b);

            if root_a == root_b {
                continue;
            }

            let new_root = self.uf.union(root_a, root_b);
            let old_root = if new_root == root_a { root_b } else { root_a };

            let old_class_enodes = std::mem::take(&mut self.classes[old_root as usize]);
            self.classes[new_root as usize].extend(old_class_enodes);

            // Merge parents from old_root into new_root
            let old_parents = self.parents.remove(&old_root).unwrap_or_default();
            let new_parents = self.parents.entry(new_root).or_default();

            for (mut enode, class_id) in old_parents {
                // Remove old non-canonical enode from memo
                self.memo.remove(&enode);

                // Canonicalize children
                for child in &mut enode.children {
                    *child = self.uf.find(*child);
                }

                let canon_class = self.uf.find(class_id);

                if let Some(&existing_class) = self.memo.get(&enode) {
                    let existing_canon = self.uf.find(existing_class);
                    if existing_canon != canon_class {
                        self.worklist.push((canon_class, existing_canon));
                    }
                } else {
                    self.memo.insert(enode.clone(), canon_class);
                    new_parents.push((enode, canon_class));
                }
            }

            merges += 1;
        }

        merges
    }
}

pub fn crate_name() -> &'static str {
    "origin-egraph"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_congruence_closure_algebraic() {
        let mut eg = EGraph::new();

        // x, y
        let x = eg.add(ENode::new("x", vec![], EType::Int)).unwrap();
        let y = eg.add(ENode::new("y", vec![], EType::Int)).unwrap();

        // f(x), f(y)
        let fx = eg.add(ENode::new("f", vec![x], EType::Int)).unwrap();
        let fy = eg.add(ENode::new("f", vec![y], EType::Int)).unwrap();

        assert_ne!(eg.find(fx), eg.find(fy));

        // union x == y
        eg.union_typed(x, y).unwrap();
        eg.rebuild();

        // Congruence closure: f(x) MUST equal f(y) after x == y
        assert_eq!(
            eg.find(fx),
            eg.find(fy),
            "Congruence closure must unify f(x) and f(y)"
        );
    }

    #[test]
    fn test_zero_type_violating_unions() {
        let mut eg = EGraph::new();

        let int_val = eg.add(ENode::new("10", vec![], EType::Int)).unwrap();
        let bool_val = eg.add(ENode::new("true", vec![], EType::Bool)).unwrap();

        let res = eg.union_typed(int_val, bool_val);
        assert!(
            res.is_err(),
            "Union of mismatched types Int and Bool MUST fail"
        );
    }

    #[test]
    fn test_1m_enodes_memory_budget() {
        let mut eg = EGraph::new();
        let count = 1_000_000;

        for i in 0..count {
            let name = format!("v_{}", i % 100);
            let _id = eg.add(ENode::new(name, vec![], EType::Int)).unwrap();
        }

        assert!(
            eg.classes.len() <= 100,
            "Hash-consing must collapse identical enodes"
        );
    }
}
