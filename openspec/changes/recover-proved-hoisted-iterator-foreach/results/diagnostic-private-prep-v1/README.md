# Hoisted iterator same-IR diagnostic candidate v1

Private, unapplied static candidate. It reuses the accepted TestNestedLoops v4 harness/test and adds a narrowly filtered observation for the frozen `cf11/UpstreamNestedLoops.test(Ljava/util/List;Ljava/util/List;)V` loops at test BCI 13 and 42.

For inner BCI 42 only, the candidate may continue past the old same-list declaration lookup when the existing `reuse::Plan` maps init/test/next to one `LocalVariable`, `Declarations::placements` names a visible complete owner, the declaration was emitted into the active lexical scope, and an Elevated placement is present in `at_region`. It logs the concrete `RegionPath`, placement, owner rows, bounded instructions/SSA values/phi inputs and uses/effect rows, then follows the existing proof checks in order.

If the existing proof returns a candidate, the diagnostic path returns `Ok(None)` before name synthesis, body rewriting, or caller-side statement removal. It never fabricates a declaration index and never emits a foreach. BCI 13 remains an entry-gate negative control. All logger output is gated by the explicit environment opt-in, declaring class, physical method name/descriptor, test BCI, and exact region path.

No shared file was edited. No Cargo/Rust compiler, JDK, JADX, CLI, or Git command was run. The source has lexical delimiter and logger-call-arity checks only; it has not been Rust-parsed, compiled, or executed.
