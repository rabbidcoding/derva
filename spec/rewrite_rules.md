# Equality Saturation & Rewrite Safety Specification

## Architectural Invariants

1. **Proof Provenance (100%)**:
   Every rewrite rule MUST specify a valid, non-empty `proof_id` (ORID). Transformations applied without proof provenance are strictly rejected.

2. **Bounded Saturation Budget**:
   Equality saturation is constrained by explicit limit bounds (`max_iterations`, `max_nodes`). When budgets are exhausted, saturation stops deterministically without error or corruption.

3. **Sound Cost Extraction**:
   The `Extractor` traverses e-classes to extract the minimum cost expression while guaranteeing semantic equivalence.

---

## Pattern & Rewrite Definition

```text
RewriteRule {
    name: "add-zero",
    search_pattern: (+ ?a 0),
    replace_pattern: ?a,
    proof_id: ORID::compute(ObjectKind::Claim, b"axiom_add_zero"),
}
```

---

## Equality Saturation Workflow

1. **Pattern Matching**: Find occurrences of `search_pattern` across active e-classes.
2. **Substituted Term Insertion**: Materialize `replace_pattern` in the e-graph using matched variables.
3. **Typed Union**: Apply `egraph.union_typed(left_class, right_class)` with the rule's `proof_id`.
4. **Rebuild**: Restore congruence closure across all e-classes.
5. **Cost Extraction**: Extract AST with lowest structural cost from target e-class.
