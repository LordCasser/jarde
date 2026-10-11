# Compound condition negative fixture draft v2

Private Java source-only fixture. No compiler or runtime was invoked and no class files were generated.

Both methods place the compound condition directly in an ordinary `if/else` with two statement
payload arms (`StringBuilder.append`), then return the resulting text and the local `x`. This avoids
a boolean local/Phi as the condition's primary consumer. The condition contains an inner side effect
in each method: prefix increment (`++x`) and explicit local assignment (`x = x + 1`).

Expected Runner output:

```text
hit:3
hit:1
miss:-1
hit:-1
hit:3
hit:1
miss:-1
hit:-1
```

The cases check that the effect is skipped when `a` is false, runs exactly once when `a` is true,
and that `c && d` supplies the fallback when the incremented/assigned value fails the comparison.
They do not establish the compiled CFG shape or which recovery layer refuses the candidate. Root must
compile the source with Java 8 and inspect reader → recover → Builder before assigning coverage credit.

## Coverage audit against current implementation

- **Hit by current real compound proof tests:** the Switch2 compound proof directly asserts refusal for a frame boundary, a case entry, and a wrong join. It also exercises proof/integration budget, pre-cancellation, and integration rollback on refusal/late Stop. The two-payload class is a positive for disjoint ordinary arms and one shared join.
- **Not hit on the new compound proof path:** external-entry, duplicate-edge, and exceptional-edge mutations in the Switch2 test call the older switch `prove_rows_with` helper. There is no compound-proof negative for external entry, duplicate canonical rows/successors, non-Normal edge, cycle, clone/path mismatch, or irreducible reduction.
- **Not hit in `Builder::prepare_compound_condition`:** no test directly stages a real `CompoundIf` with an inner store/Increment, independent instruction, or unaccounted producer and checks complete refusal/quoted ownership with no partial output. Older shared-false and conditional-value independent-effect tests exercise separate proof paths.
- **Scope:** this fixture targets the missing statement-form, inner-effect/assignment candidate. It remains a source proposal until its actual compiled class reaches the intended compound proof or Builder branch. Do not add CFG edge override APIs to force a shape; keep other proof-negative classes separate.
