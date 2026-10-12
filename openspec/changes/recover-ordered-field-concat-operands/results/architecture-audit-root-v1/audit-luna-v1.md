# Retry v2 `exc()V`: nested Error-message construction audit

Read-only architecture audit, 2026-10-12. No shared source was edited and no Cargo/JDK/JADX/CLI/toolchain was run. This note uses the root's completed comparison output and frozen class artifacts; it does not claim a new test or recovery passed.

## Observed failure

The root's `/private/tmp/jarde-loop-candidate-comparison-root-v2/root-observation-summary.json` records 40 commands, 4 retry-v2 whole-class runtime mismatches, and the reason: `exc()V` omits the nested `AssertionError` message construction, so the uncaught-error case changes behavior. The v2 report for `RetryFixture.exc()V` is in `candidate/retry-v2-debug-javac8/report.json`; its physical class digest is `2f473c5d80f0e81e6a867ef0e1123c7039434d79b9106a4cc9f6282fb16fc533` (1742 bytes). The same report says method `exc()V` was recovered, but its source has comments at BCIs 53 and 56 where the throw expression belongs. It is a recovered artifact, not a compiling Java result (`syntax_status: not_java`, `verification: not_performed`). The root's comparison summary is the actual runtime evidence; the per-method report is the actual candidate refusal evidence.

The frozen javac-8 `RetryFixture.javap.raw` at `/private/tmp/jarde-loop-catch-retry-baseline-root-v2-v1/physical/debug/javac8/RetryFixture.javap.raw` gives this bytecode shape in `exc()V`:

| BCI | Instruction and role |
|---|---|
| 53 | `new AssertionError` |
| 56 | `dup` |
| 57–61 | `new StringBuilder; dup; invokespecial <init>()V` |
| 64–76 | append literal `"uncaught-error:"`, read `this.c` at 70, append int, `toString()` at 76 |
| 79 | `AssertionError.<init>(Object)`; its physical argument is the String result from 76 |
| 82 | `athrow` |

The report's exact `new@1` refusal for head 53 is `jre_new_shape`: the construction at 57 is recursively accepted as a construction, but its Builder instance is not an argument to constructor 79. That is the right reading of the value flow: the Error constructor gets the `String` produced by `toString@76`, not the Builder instance produced by ctor 61.

The report's `concat@1` separately refuses head 57 as `jre_concat_interleaved_effect`: `getfield RetryFixture.c` at BCI 70 lies between the Builder's `append` calls. It gives the analogous refusal at head 14 / field read 27 for `"call:" + this.c`, and at head 108 / read 121 for `"success:" + this.c`. These are independent candidates; the observable uncaught Error message depends specifically on 57–76.

The actual output's current generated text has a conditional at the right location but only attribution comments for allocation/copy and the throw tail. It does not spell the Error object or message. This matches the root's runtime mismatch and must remain distinct from the `test()I` retry-region ownership work. The completed same-run IR diagnostic package analyzed retry `test()I`, not `exc()V`; I therefore do not claim it printed `exc()V` SSA. For `exc()V`, the evidence here is the frozen class disassembly plus the normal per-method SSA-backed analysis report and its source-map BCIs.

## Existing decision path and smallest extension point

`concat::plan` (`crates/jarde-java/src/concat.rs:393–425`) scans one method's SSA blocks and verifies recognized `StringBuilder`/`StringBuffer` chains. `verify`'s operand-order check around 1490–1544 requires each append operand to be produced after the preceding chain instruction and before the append, preserving left-to-right position. Its intervening-instruction match around 1570–1638 accepts only modeled value instructions/calls and a narrow field-copy case; every other `Operation::Field` reaches the effect refusal. The source comment explicitly calls out moving or changing an effect's order/count.

The outer construction path already has the needed composition mechanism:

- `init::verify_metered` scans nested `new` candidates at `init.rs:802–813`, and skips a candidate already owned by a verified concat (`!chains.owns(bci)`).
- It obtains the outer physical constructor operands, traces their argument dependencies, and calls `verify_concat_arguments_metered` at `init.rs:1077–1087.
- That helper (`init.rs:2217+`) accepts a chain only when the exact SSA value written at the chain's `toString` tail is the physical constructor argument, the result has one use at that constructor, the entire chain interval is inside the argument interval, and the chain owns exactly the whole interval and is in the argument dependency set.
- The outer expression's exception-boundary proof at `init.rs:1286+` requires identical handler coverage for its span and sole consumer. Existing nested construction boundary and single-reader rules remain in force.

Consequently the minimum architectural seam is the concat candidate gate for one narrowly proven field-read operand. Once chain 57–76 is owned by concat, `new@1` will no longer misclassify BCI 57 as an independently nested constructor; it will see the verified chain tail at 76 as the actual argument to AssertionError ctor 79. The existing `new` checks then retain authority over argument dependency, one use, evaluation order, and handler coverage. Do not weaken the nested-instance-must-be-a-constructor-argument invariant: it correctly rejects arbitrary nested allocations whose values are only used as intermediate receivers.

The necessary missing fact is field volatility. `concat::plan` currently receives only SSA, decoded operations, and `FieldCopies`; a raw `FieldAccess::Read` does not state whether a member is volatile. A rule that blanket-accepts reads would break Java memory ordering for volatile fields and could alter effects. A small private API extension may pass just the declaring class's field headers to concat (or an already available, exact field-read fact with flags), then allow only an instance read whose resolved owner/name/descriptor matches a declaration proven non-volatile, whose receiver is exactly local 0 (`this`) with no intervening receiver computation, and whose result is the single operand consumed by the immediately following append. Static reads must remain excluded because class initialization can be observable. Reads through other receivers must remain excluded in this first child because null-check timing and receiver-expression effects need separate proof. If class-header lookup is unavailable or ambiguous, preserve the existing refusal.

This is a single bounded concat proof extension, not a general pure-field system or a `new` gate relaxation. It should be specified as its own OpenSpec child because the current parent retry change is already accepted for a different region behavior and root explicitly says retry v3 does not replace retry v2.

## Must-hold proof and accounting conditions

The candidate chain must still satisfy every existing concat condition: exact known builder type and constructor; ordered append overloads; each argument produced after the previous chain operation; exact builder identity for each receiver and `toString`; one owning chain per instruction; all append operands in SSA dependencies; and no extra readers or calls. For the new one field-read shape, additionally require:

1. `getfield` only, not `getstatic`; receiver's SSA producer is precisely `aload_0` for this instance method, with no Phi, cast, intervening call, or alternate receiver.
2. The symbolic owner, field name and descriptor resolve exactly to the same class member header; the header does not carry `ACC_VOLATILE`. Unknown external/superclass declarations and unresolved members refuse.
3. The field-read result is the exact one value passed to exactly one append in this chain; no store, second reader, or branch observes it. The read sits between the previous chain instruction and that append, as the existing operand-order test already requires.
4. Preserve original exception coverage for every chain instruction and its outer Error construction/consumer. If the field read or a concat operation crosses a handler boundary, reject; do not infer catch equivalence from source shape.
5. Preserve call/effect order. `StringBuilder` construction, each append and `toString` remain inside the exact dependency expression. Any other invocation, write, volatile read, class-initializing static access, or state-changing instruction between chain links continues to refuse.
6. Account every new scan of member headers, receiver SSA, uses, chain interval and handler coverage in existing budgets. A budget `Stop` propagates; it never becomes a `false` proof or a fallback “safe” result. Keep the ownership update atomic: if any gate fails/stops, the concat candidate claims no instructions and the existing `jre_concat_interleaved_effect` remains available.
7. Keep source-map attribution over the full interval: the resulting Error expression must account for allocation 53, copy 56, concat interval 57–76, constructor 79, and throw 82 without losing or double-owning BCIs. In particular the Error's exact Object-constructor overload and message argument must be attributed to the physical facts that support them.

## Adversarial regression matrix for the separate child

The positive fixture should be the frozen retry-v2 `exc()V` bytecode in all existing javac/debug/none comparison lanes. Assert the uncaught path still throws `AssertionError`, its exact message is `uncaught-error:<call count>`, the call counter/events show one `exc()` attempt and no success event, and the earlier `call:<n>` event remains once. Compare whole-class observable outputs and mapped recovery; it is not sufficient to assert that the helper accepts a synthetic concat chain.

Keep negative controls beside it:

- A `volatile int` field with the same append shape must retain `jre_concat_interleaved_effect` and must not be emitted as a concat.
- A static field operand must refuse, including an otherwise simple static constant-like field if class initialization is not proven inert.
- An instance field read through a possibly null object or a receiver returned by a method must refuse in this first bounded rule.
- A side-effecting method call between append links, a write between links, a field read used twice, or a chain whose `toString` result has another reader must refuse and keep each call/read count/order unchanged.
- An exception-table split over the field read, append, concat tail, Error ctor or throw consumer must refuse unless the existing exact handler-coverage check proves identical coverage.
- Exhaust each relevant IR-item/analysis-step budget immediately before the added metadata/SSA/coverage work; assert a Stop receipt and no partial ownership. Also assert a completed scan remains byte-for-byte deterministic across debug/none and Java release lanes.
- Check both direct concat recovery and embedding under `new AssertionError(Object)`: concat alone passing is not enough. Assert `new` sees the String tail as the sole constructor argument and that its `jre_new_shape` does not reappear.

## Local JADX reference and limits

The local JADX source at `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/ConstructorVisitor.java:65–105` turns a constructor invoke plus its `NEW_INSTANCE` assignment chain into `ConstructorInsn`, moves the instance to the constructor result, and removes the allocation assignment chain when possible. `InsnGen.java:727–785` then writes `new Type(arguments)` from that instruction's arguments. This is a useful value-composition reference: constructor arguments are rendered from the value graph, and the nested Builder chain can be the Error's argument through its String result. It does not provide a safety proof for this repository's field/effect/exception/budget rules and is not evidence that the proposed Jarde gate is safe.

## Open questions before implementation

- Does the production class-analysis entry point already carry the declaring class's `MemberHeader` list into concat planning, or is the smallest safe metadata handoff at the build caller? Unit-test call sites currently invoke `concat::plan` with only SSA/operations/FieldCopies, so any new fact must remain explicit in tests and production.
- Is the requested artifact a compiling Java source, or an analysis artifact whose comparison runner separately consumes generated snippets? The v2 method report explicitly says syntax was not Java and verification was not performed. The child must retain the existing output contract but still prove its promised observable recovery behavior through the root's whole-class test path.
- Are `RetryFixture.c` reads explicitly nonvolatile in every frozen class lane? The javap field flags shown in javac8 report no volatile flag; confirm each pinned class header in the test before using this as a positive gate.
- What source-map primary/derived mapping is the canonical accepted presentation expected for literal, field operand, append, constructor, and `athrow`? Freeze it in the child test after the source-map contract is checked; do not fabricate expected mapping based solely on source text.

The independent child should therefore be narrowly titled around permitting a resolved non-volatile `this` field read as one ordered concat operand, with the v2 Error fixture and the listed negatives. It must not remove the `StatementFree` effect gate generally, alter the retry-region work, or treat retry v3 as covering this v2 failure.
