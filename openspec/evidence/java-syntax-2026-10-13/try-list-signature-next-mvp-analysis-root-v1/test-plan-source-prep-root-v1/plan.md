# Private acceptance plan: ordinary try + generic List foreach (v2)

Status: preparation only. The candidate Java sources were **not compiled or run**. This plan fixes no tool output, Region support, compile result, or runtime outcome. The target is the full existing CF11 fixture plus appended candidates, never a minimal replacement.

## Scope and syntax candidates

Apply the root-approved exact E proof uniformly to every currently supported exact element type, at least `String` and `Integer`; do not special-case class, method, local, or type spelling. The envelope is the currently certified ordinary single-range Try shape: one prefix sequence followed by one complete loop, one named typed catch, exact row/range and exit bridge, full owner accounting, and no extra top-level tail.

The two positive-shaped methods use the same body logic for String and Integer. Both have a branch that adds the iterated E and a branch that adds `null`, so each arm has distinct observable behavior while retaining the already-supported List null-add form. They each have an empty-input and null-element runtime case. `tryStringRemovePrefix` checks an explicit `remove((Object) key)` prefix plus foreach/add. `tryStringNested` probes nested foreach under the protected envelope; its support is **pending real Region/source proof**. `tryStringUnknownCall` passes output to `observeUnknown` and is actually called by Runner; whether the source gate refuses it must be established from the future report.

The complete existing control class still contains `rawAliasEscape`, `castBoundary`, `indexOverload`, `reassigned`, `wildcardNull`, `rawLists`, and the original `handled` method. The intended controls remain distinct: raw aliasing and casts, `remove(Object)` versus `remove(int)`, formal reassignment, wildcard writes, unknown external use, and raw-signature behavior.

## Future full-class matrix

For each cell, use the complete candidate `UpstreamNestedLoops.java` and `Runner.java`, including all old methods. Never patch generated decompiler Java to get it to compile.

| Axis | Required cells | Evidence |
| --- | --- | --- |
| JDK | 8 and 23 | Executable/version, argv, raw streams, exit status, guards |
| Original class debug tables | `javac -g` and `javac -g:none` | Separate original class-set SHA and original stdout for each JDK/profile |
| Source provider | Original, JADX default, JADX rename-none, Jarde default, Jarde all | Full-class source/report pins, raw records, source-map/method identity checks |
| Compile and run | Complete target class plus complete Runner for each provider × JDK × debug profile | Compile result; only compiled legs run; compile failure is recorded as no-runtime |
| Jarde evidence mode | default and all | Whole-class text, each target method text, report and maps agree; no partial publication |
| Runtime traces | Exact cells below | Same-JDK original versus each compiled provider: values, exception class/message, effects, and event sequence |

Treat `-g`/`-g:none` as javac input metadata; Jarde default/all and JADX default/rename-none are separate evidence/profile axes. Preserve exact effective argv and every raw stream. No row count or result is frozen in this prep.

Runtime trace cells:

- For each E in String and Integer: successful input exercising both branch arms, empty input, and one null element.
- For each E in String and Integer: inject both `IllegalStateException` and `AssertionError` at input `iterator`, `hasNext`, and `next`, then at output `add` (16 cells total). The harness configures failure on the list that receives that operation. RuntimeException is inside the method's catch; Error checks that the catch does not widen.
- String `remove(Object)` prefix: normal removal, injected RuntimeException, and injected Error.
- Nested-loop probe and unknown-call control run as ordinary traces. The nested case remains topology-pending; the unknown-call row proves the fixture actually invokes the control.
- The original 17 output rows are retained in the same order and before all new trace rows. They continue to cover the existing baseline, exact generic controls, casts, alias, reassignment, wildcard, handler, and overload paths.

The `TraceList` has only one constructor (`TraceList(String label)`); values are seeded through a method that bypasses event logging. Failure injection is a separate explicit call after construction. This avoids constructor overload ambiguity and ensures `output.add` failures target output rather than input. Event capture records iterator protocol and List operations; snapshots avoid adding events.

## Source-level acceptance gates (not runtime claims)

1. Exact physical method signature erasure and parsed generic E agree. String and Integer use the same proof path and differ only by E.
2. Accepted methods have a complete same-run Code/operation/SSA/effect census, including handler-owned instructions; each instruction and formal read is accounted exactly once.
3. Exact Try row, catches, exit, and current Region tree own prefix, protected loop, handler, and pure bridge once. Unowned blocks, competing/mismatched rows, fallback, or Stop refuse atomically.
4. Iterable formal reads feed the certified foreach; output formal reads are direct approved List receiver uses. Parameter writes, raw alias escape, unknown call arguments, or hidden consumers prevent generic projection where the existing proof requires closure.
5. `add(E)` requires exact List receiver/call descriptor and argument/wrapper proof. `remove(Object)` and `remove(int)` remain separate physical overloads. Real iterator/checkcast BCIs stay mapped; an explicit cast is not erased as a synthetic wrapper.
6. Default/all agree on class and method text, raw report body, and source maps. Source-map spans are UTF-8 boundaries and BCI provenance binds to the exact raw method; Try block ownership is checked against its certificate.
7. Budget exhaustion and cancellation through census, Region binding, List-call proof, or projection propagate the actual Stop and publish no partial generic declaration/body/map. New traversal is charged and polled using existing budget dimensions.

These are source-proof gates only. Raw Java compiling does not prove generic projection. A decompiler lane is accepted only if its full class emits the exact E declarations/calls, compiles, and reproduces same-JDK original values, exceptions, effects, and traces.

## Control outcomes to check, not assume

| Control | Evidence decision to verify |
| --- | --- |
| `rawAliasEscape` | Preserve raw alias and `add(Object)` with physical casts; generic projection should refuse because a raw alias can pollute `List<String>`. |
| `castBoundary` | Preserve explicit physical checkcasts and BCI origin. |
| `indexOverload` | Preserve `remove(Object)` and `remove(int)` exact bindings. |
| `reassigned` | Refuse generic projection if a parameter slot is written/rebound. |
| `wildcardNull` | Keep unsupported wildcard write outside generic projection; do not infer capture from null legality. |
| `tryStringUnknownCall` | The Runner invokes it; inspect actual source proof and record its first refusal or result. |
| `rawLists` | Preserve existing raw path; it is not a generic projection refusal. |
| nested Try/finally, multiple ranges/handlers, Try after loop | Keep unproven shapes conservative until independently certified and tested. |

## Future collector guardrails

Reuse the approved private full-class baseline collector pattern: 5 GiB preflight, 1 GiB target cap, one-second process-group guard, default stack, capture all argv/raw streams and before/after source/tool pins, and exact finally-clean. Reject output-directory overwrite. Preserve failed/no-runtime legs and never hand-edit generated sources. This plan creates no execution directory and runs no collector.
