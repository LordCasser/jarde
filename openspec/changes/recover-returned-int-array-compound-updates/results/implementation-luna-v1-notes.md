# Returned int-array update implementation patch, Luna v1

Base source revision: `561de209531c021a9d8adb7c979bae5a57d6c3fb` (provided by the coordinating task).  
Patch: `implementation-luna-v1.patch`  
SHA-256: `13c4aa858adf316be2170154f03bd6e880da6cda3799d58b5eb7fbed0f1bb74f`

## Patch contents

The patch adds a value-producing `ArrayAssign` expression for the one proved `int[] +=` return shape and routes it through the existing `CompoundAssignments::prove` walk. The returned path shares `prove_array_update`'s array/load/add/prefix/RHS proof with ordinary compound updates, then proves the immediate `dup_x2`'s three category-1 inputs and four distinct outputs, the exact unique `iastore`/`ireturn` consumers, and the contiguous `iadd → dup_x2 → iastore → ireturn` interval. Claiming happens at the return; the closed producer prefix and update chain are suppressed until that return emits one Java assignment expression.

The lvalue row must already prove as `int[]`, and the method return must be descriptor `int`. Index and RHS additionally need a known presented Java primitive accepted by the existing `primitive_conversion_source_matches(&Type::Int, …)` path. This accepts existing int-compatible byte/short/char/int expressions while rejecting absent evidence and boolean; verifier category-1/frame `Int` is not used as Java expression type evidence. The implementation adds no conversion rule or general field/local/operator support.

The AST node is handled by emission, precedence, type presentation, origin/name/source-map/field/assertion and expression walkers. The class-initializer field-read walk marks it incomplete as a write; lambda-donor substitution keeps it physical with other writes. Tests upgrade only the returned boundary expectation, retain the merged row-Phi refusal/source origins, exercise the full returned fixture for v8/v23 and exact store/return origins, cover boolean and char descriptor controls from canonical `controls/v8|v23/{variant}.class` fixture paths, and add public budget/cancellation plus ignored real-JDK full-class comparison. CI explicitly invokes that ignored comparison.

Changed paths:

- `.github/workflows/ci.yml`
- `crates/jarde-java/src/asserts.rs`
- `crates/jarde-java/src/ast.rs`
- `crates/jarde-java/src/build.rs`
- `crates/jarde-java/src/emit.rs`
- `crates/jarde-java/src/field.rs`
- `crates/jarde-java/src/report.rs`
- `tests/p3_nested_int_array_compound_updates.rs`

The test source references the fixed `fixtures/returned-int-array-compound-updates/Runner.java`, the two `ReturnedIntArrayUpdates.class` inputs, and eight typed-control class files. These are intentionally not embedded in this patch: the coordinating task stages them byte-for-byte from the frozen, independently verified inputs and records their source/fingerprint evidence.

## Metering and validation

The existing CompoundAssignments instruction scan remains the sole pass and keeps its per-instruction budget charge. On the new returned path, single-use enumeration uses the budgeted helper, and existing expression-prefix/RHS dependency and interval walkers remain budgeted. Ordinary compound proofs keep their prior unmetered `single_use_at` behavior through the shared wrapper. Public budget-stop and pre-cancel controls are added; they do not claim internal checkpoint-pressure acceptance.

Validated only with `git apply --check` against the named base workspace. No Rust compilation, rustfmt, Cargo tests, JDK, decompiler, or Jarde CLI were run; those operations are reserved for the coordinating root task. The patch was prepared on selected source copies in `/private/tmp/jarde-returned-array-luna-v1/`; no product, test, fixture, workflow, spec, task, or handoff files in the main source tree were edited.
