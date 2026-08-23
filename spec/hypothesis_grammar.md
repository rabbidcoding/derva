# Typed Hypothesis Grammar Specification (v1)

## Formal Invariants & Type Safety

1. **Strict Type Safety**: Every generated AST expression $E$ satisfies $E.\text{type\_check}() \equiv \text{true}$.
2. **Canonical Versioning**: The grammar structure derives a unique content-addressed ORID hash (`canonical_hash`). Any change in non-terminals, operators, or types mutates the hash.
3. **Bounded Expansion**: Non-terminal productions report explicit branching factors and enforce strict depth bounds to prevent infinite un-bounded recursion.
4. **Cost Model**: Every operator and production rule declares an integer cost used by proof search heuristics.

## Example Grammar Definition

```text
// Non-terminals and types
Expr: Int
  -> Apply(Add, [Expr: Int, Expr: Int]) (cost: 1)
  -> Apply(Mul, [Expr: Int, Expr: Int]) (cost: 2)
  -> Var("x", Int)                      (cost: 0)
  -> Const("1", Int)                    (cost: 0)
```
