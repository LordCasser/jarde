# Typed assertion audit (read-only)

Scope: searched the existing Rust assertions under `tests/` and `crates/jarde-java/tests/` for fixed `int local…` and `Object local… = null` spellings that could be changed by the current full-write char or null-leading exact-reference rules. No test, CLI, JDK, Cargo, Git, or formatter was run. This is a static audit against the checked-in test fixtures and the rule predicates; it does not claim the next CI result.

## Confirmed rule match; likely next assertion failure

`tests/p3_required_conversions.rs::a_write_states_the_written_types_own_type` pins the bodies of `RequiredConversions.declared(C)I` and `assigned(C)I` to `int local1`. The committed class's source is [RequiredConversions.java](/Users/lordcasser/workspace/projects/jarde/tests/fixtures/p3-required-conversions/RequiredConversions.java): `declared` stores its `char` parameter into a non-parameter local; `assigned` stores literal zero and then that `char` parameter into the same local. The class is compiled with `-g:none` as documented in [README.md](/Users/lordcasser/workspace/projects/jarde/tests/fixtures/p3-required-conversions/README.md), so those local declarations are inferred rather than taken from LVT metadata. Both write sets fit the implemented char proof: every write is a direct char producer or an in-range integer literal, and each local is outside the parameter slots. The exact-body assertions at `tests/p3_required_conversions.rs:357-372` therefore appear stale under the new rule and are the strongest additional CI failure candidate.

The neighboring `written(C)I` assertion is a field write, not a local declaration, and should not be changed by local-source inference. The other assertions in this test that concern `arg0` are parameter positions; the new local candidate explicitly excludes parameter slots. The fixture README's old examples and explanation also still spell the two locals as `int`, so its prose may need to follow the final behavior if the test is updated.

`tests/p3_meeting.rs` had the same shape in `Meet.viaStore(C)I`. The current checked-in assertion already expects `char local1 = arg0;` (around lines 155-162); root has corrected this known conflict. Its `trunc` test's `int local1 = (byte) arg0` is an actual `i2b` conversion case and is unrelated.

## High-priority null-leading candidate; confirm from next run

`tests/p3_reference_slot_lifetimes.rs::unsupported_reference_boundaries_keep_the_cli9_source_verbatim` compares the complete output for the frozen `NullThenBuilder.class` against `tests/fixtures/p3-reference-slot-lifetimes/negative/unknown-null/NullThenBuilder.baseline.java`. In that checked-in presentation, `run(Z)Ljava/lang/Object;` initializes `local2 = null`, reads it in `if (local2 == null)`, then writes `new java.lang.StringBuilder("second")`; the later read is `local2.length()`. Thus the visible writes are a leading null followed by one direct, exact `StringBuilder` value, which appears to fit the new null-leading rule. If so, a declaration changing from `Object local2` to `StringBuilder local2` changes the verbatim negative assertion even though the earlier null comparison remains equivalent.

This is not confirmed: the directory contains only the frozen class and the prior recovered `.baseline.java`, not the original Java source or a current source-map/IR proof. Keep this as the first null-related CI item to inspect, rather than changing the assertion preemptively. The test name “unknown-null” is not evidence that it remains outside the new producer proof.

## No likely change under the stated predicates

- `tests/p3_twr_saved_return_typing.rs` explicitly expects `Object local1 = null` for `Wtyping.nullValue`. That method's saved value is only null; it has no non-null exact-reference write, so it does not satisfy the null-leading rule's requirement for at least one concrete non-null write.
- The hand-built `STRAIGHT_LINE` null-local cases in `crates/jarde-java/tests/p3_java_recovery.rs` assert `Object local1 = null`, but the body only has the null write and a method call on a separate receiver. They are not null-plus-exact-reference write cases.
- The existing nullable-finally tests (`crates/jarde-java/tests/p3_segmented_null_lead_finally.rs`, `p3_local_null_conditional_finally.rs`, and `p3_null_lead_straight_finally.rs`) assert concrete resource types already recovered from their resource producer/cleanup structure and separately assert the `= null` statement. The typed feature should preserve those spellings and statements; they do not pin a stale `Object` type.
- `p3-reference-slot-lifetimes`'s remaining negative controls cover all-null, mixed exact reference types, copies/phis, handlers, slot reuse, or loop/back-edge shapes. They fail at least one stated null-leading proof condition, unlike `NullThenBuilder`'s visible baseline shape.
- The many other `int local…` spellings found in tests are controls for arithmetic, int parameters, boolean slot recovery, byte/i2b conversion, switch values, or unrelated local-scope/resource behavior. A spelling search alone is not grounds to change them. In particular the full-write proof does not upgrade every `int` local, copied/merged values, arithmetic producers, or parameter slots.
- The new `recover-proved-local-source-types` boundary fixture already has explicit positive/negative controls: `charFieldSeed`, `charI2cSeed`, and `charEntryParameterSeed` spell char locals; `intWithOutOfRangeWrites`, `intWithArithmeticWrite`, and `intWithUnknownCopyMerge` remain `int`; `exactStringWritesAfterNull` spells a `String`; and mixed/all-null/unknown-copy reference controls stay conservative. These are generated evidence artifacts, not old assertions in `tests/` or `crates/jarde-java/tests/`.

## Suggested CI triage order

1. If the next CI fails in `p3_required_conversions`, compare only the `declared` and `assigned` declaration spellings first; preserve the field and consumer-position checks.
2. If it fails in `p3_reference_slot_lifetimes`, inspect the actual `NullThenBuilder.run` method report and physical writes. Decide from that evidence whether this old fixture is now a positive exact-reference case; do not relax the complete-source comparison generically.
3. Treat any other `int`/`Object` assertion as a separate finding unless its fixture proves the exact producer/write shape above.
