# Safe Horn-Rule Subset Specification (v1)

## Decidability & Safety Invariants

1. **Range Restriction (Safety Property)**: Every variable appearing in the rule head `H` MUST appear in at least one positive body literal $B_i$.
2. **Stratified Negation Safety**: Any variable occurring in a negated literal $\neg B_j$ MUST occur in at least one positive body literal $B_k$.
3. **Finite Term Depth**: Complex un-grounded function terms (e.g. $f(f(X))$) are disallowed in v1 to preserve polynomial time decidability and prevent non-terminating unification loops.
4. **Content-Addressed Identity**: Every `HornRule` derives an immutable content-addressed ORID computed over its canonical binary serialization.

## Syntax & Examples

```text
// Valid Horn Rule
grandparent(X, Z) :- parent(X, Y), parent(Y, Z).

// Valid Rule with Stratified Negation
unassigned_task(T) :- task(T), not assigned(T).

// REJECTED: Range Restriction Violation (X in head is not bound in positive body)
invalid_head(X, Z) :- parent(Y, Z).
```
