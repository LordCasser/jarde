# Catch after a field store: minimal Java 8 probe

This frozen class isolates the prefix instruction immediately before a plain `try/catch` protected range. `fieldStore` writes `calls = 7`, compiled as `putstatic`; `localStore` initializes `value = 7`, compiled as `istore`. Both then call the same potentially throwing helper inside a `try`, return the stored value normally, and return stored value plus 11 in the `IllegalArgumentException` handler. The methods therefore have the same source-level control flow and observable outputs; only the kind and destination of the prefix store differ.

## Frozen input and runtime

- Compiler: local `javac --release 8 -g:none` (JDK 23; obsolete source/target warnings only).
- `CatchAfterFieldStore.class` SHA-256: `81b927f95b3651b69b9694eb6cdd49f7521c636fa278ce8daa3c97ba63429ee3`
- `CatchAfterFieldStore.java` SHA-256: `b965cfe869f436c2ea4fce4e2e374be2bb869d4cf4541186511b9a4f4c2da320`
- `CatchAfterFieldStoreRunner.java` SHA-256: `5b4a4152e35f3f0d35f2e48c0d08f50182a5b884773917699d6c423c3238515f`
- `javap.txt` shows `fieldStore`'s protected range `[5,12)` after `putstatic` at BCI 2, while `localStore`'s range `[3,8)` follows `istore_1` at BCI 2.
- Frozen original under `java -Xverify:all`: `field=7,local=7` for normal flow and `field=18,local=18` for the throwing flow (`original-run.txt`).

## Three-way result

JADX 1.5.6 outputs both plain catches. In particular, its `fieldStore` contains `calls = 7; try { maybeFail(z); return calls; } catch (...) { return calls + 11; }`. The default-package-adjusted complete JADX source recompiles with `javac --release 8 -g:none` and matches the original runner under `java -Xverify:all`; `jadx-diff.txt` is empty. Original JADX output is preserved under `jadx2/sources/defpackage/CatchAfterFieldStore.java`.

Jarde CLI `/tmp/jarde-for-add-store-replay/jarde-cli`, SHA-256 `63cf72aef794786c3f34e3c232607eb6ebaa168c9ca08a517a473d95323008f2`, class-source output is preserved in `jarde-source.java.txt`, with the full report in `jarde-report.txt`. `fieldStore(Z)I` is refused with `jre_guard_resource_init` at BCI 2: “the resource's own initialisation is not one statement of this block whose value lands in a slot”. Its normal class-source consequently does not preserve either return path. `localStore(Z)I` is **not** classified as a resource: it has `jre_region_exception_edge`, followed by a local definition-use fallback. This is a classification distinction, not evidence that Jarde fully recovers the local method. The frozen class-source itself is intentionally not compiled or run because it contains explanatory fallback comments instead of complete method bodies.

## Classification boundary and smallest repair

In `crates/jarde-java/src/guard.rs`, `resources()` finds the last instruction before each protected row. It skips rows with no preceding instruction, then tries TWR proof. `initialisation()` ultimately requires a local `Operation::Store { slot }`, and `initialises_resource()` regards a readable store as a resource candidate only when the statement ending at that store contains `Allocate`, `Invoke`, or `InvokeDynamic`. A static field write is not a local slot store, but `resources()` currently reaches the TWR refusal path anyway; that refusal then owns the ordinary catch row and suppresses catch recovery.

The earlier suggestion to skip every row whose preceding instruction is not a local `Operation::Store` was too broad and is withdrawn. `tests/p3_guard.rs::a_handler_range_that_swallows_the_initialisation_is_refused` mutates a valid TWR row from `[6,9)` to `[5,9)`: the instruction immediately before the widened range is `invokestatic` (not a store), yet the row now covers part of the resource initialization and must remain `jre_guard_resource_init`.

A narrower candidate is to recognize only a complete field assignment immediately before the protected range as a non-resource prefix, using the existing statement proof (`single_statement`) plus a proof that the assignment leaves no live operand-stack output. The candidate must not fire when the range starts inside the assignment, when statement boundaries are ambiguous, or when earlier initialization instructions are now covered. Those cases continue through the present conservative TWR refusal. This rule can then hand only the fully proved ordinary `putstatic` prefix case to `catches()`, while local resource stores and all close/suppression/row-contour proofs remain unchanged. No implementation was made in this evidence task.

## JADX try-region ordering

The checked local JADX 1.5.6 source in `/Users/lordcasser/workspace/testzone/jadx` shows the region pipeline in `jadx-core/src/main/java/jadx/core/dex/visitors/regions/RegionMakerVisitor.java`: build the method region, attach exception-handler regions with `ExcHandlersRegionMaker`, then call `ProcessTryCatchRegions.process`, followed by post-processing and cleanup. `ProcessTryCatchRegions.java` collects exception table rows, moves parent rows first, and wraps blocks reachable from each row's top splitter while excluding blocks reachable from handler paths. `RegionGen.makeTryCatch` emits the already-built try region and its catch regions. This ordering preserves a field assignment outside the protected interval as a preceding statement; the catch reconstruction does not infer a resource header from it.


## Assignment-only control probe

`CatchAfterFieldAssignment` isolates the field-store prefix from the earlier probe's return-inside-try and local-definition complications. `fieldAssignment` is exactly `field = 7; try { maybeFail(flag); } catch (...) { field = 18; } return field;`; `localAssignment` has the same control flow with a local initialized to 7 and assigned 18 in the catch. The frozen `javap` shows the field method row `[5,9)` immediately after `putstatic` BCI 2, and the local method row `[3,7)` immediately after `istore_1` BCI 2.

The original runs normally and with `-Xverify:all`: `field=7,local=7` and `field=18,local=18`. JADX 1.5.6 recovers both catches, and the adjusted complete class recompiles and produces identical output (`assignment-jadx-diff.txt` empty). Jarde still refuses `fieldAssignment(Z)I` as `jre_guard_resource_init` at BCI 2. It fully recovers `localAssignment(Z)I` as a try/catch with `local1 = 7`, catch assignment `local1 = 18`, and `return local1`; unlike the first control probe, this establishes the local path has no separate recovery obstacle. The raw source, class, Runner, hashes, `javap`, JADX and Jarde outputs are all retained with the `assignment-` prefix in this directory.

For the TWR counter-boundary, the existing regression at `tests/p3_guard.rs::a_handler_range_that_swallows_the_initialisation_is_refused` documents the widened `[5,9)` row and expects `jre_guard_resource_init`. The field-assignment exception here starts exactly after the completed `putstatic` at BCI 2. The evidence therefore supports evaluating a narrowly proved completed-field-assignment case; it does not support a blanket “non-local-store means catch” rule.
