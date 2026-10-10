# Integer-array constant-name V3 private patch review

Patch: `private-patch-luna-2.1-2.2-v3.patch`  
SHA-256: `efa4b4584ac4bb26590c511df91ff2ca3aab0806d290d8ca7294adef2afb56a6`  
Status: private review artifact; not applied to product files.

V3 narrows the V2 implementation in four places:

1. The facade no longer adds `integer_array_projection_source_is_current` or a predicate-only test. For an actual array-use candidate it checks whether that physical method already has staged member text, then compares the candidate’s original-body replay with the current physical method text. A new integration test compiles an assertion-bearing array-return method and verifies that the prior assertion projection owns the method, the physical array literal remains unchanged, and no array-name anchors are published for it.
2. The `returns_int_array` mismatch path now refuses the candidate directly. The nested `returns_int` recovery fallback was unreachable for that descriptor and has been removed.
3. `ClassSourceMethod::integer_constant_projection_text` no longer requires whole-method text equality for the existing switch/return projection path. The array-use path retains its separate exact AST replay/body comparison before it can publish array-name spans.
4. Added traversal, mapping, replay and atomic-publication accounting remains scoped to the new array-use path. Existing switch candidate, occupancy and name-map handling keeps its prior accounting and behavior. The compiled shadow/name tests remain in the patch.

The patch contains four files: `crates/jarde-java/src/report.rs`, `crates/jarde-java/src/emit.rs`, `src/facade.rs`, and `src/class_source.rs`. The facade test uses the existing `compiled()` helper, which supplies Java parameter names (`-g -parameters`), and exercises refusal through the real facade adapter rather than calling an extracted boolean predicate.

No Git, Rust, Cargo, rustfmt, JDK, JADX, or CLI command was run for this revision. The patch was generated as a unified diff from a private staging copy and was not applied or compiled; Rust/API validity and test outcomes remain for root review and execution.
