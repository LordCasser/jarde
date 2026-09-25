# Mixed short-circuit value assigned to an instance field

This is a frozen Java 8 source/class/Runner reproduction for the instance `putfield Z` consumer. The source and class files are under [`tests/fixtures/p3-conditional-values/mixed-short-circuit-instance-field`](../../../../tests/fixtures/p3-conditional-values/mixed-short-circuit-instance-field/README.md); the exact SHA-256 values, tool versions, and the `javap` output are retained beside this file.

## Sample and bytecode

`MixedShortCircuitField.one(ZZ)V` executes:

```java
target(nullReceiver).result = (a && b()) || c();
```

`target` increments `receiverCalls` and returns the shared `Box` or null. The frozen method bytes have this instruction sequence:

| BCI | Instruction | Meaning |
| ---: | --- | --- |
| 0 | `iload_1` | `nullReceiver` |
| 1 | `invokestatic target:(Z)LMixedShortCircuitField$Box;` | evaluate receiver once |
| 4 | `iload_0` | `a` |
| 5 | `ifeq 14` | short-circuit `&&` |
| 8 | `invokestatic b:()Z` | first RHS |
| 11 | `ifne 20` | short-circuit `||` |
| 14 | `invokestatic c:()Z` | second RHS |
| 17 | `ifeq 24` | choose false producer |
| 20 | `iconst_1` | true producer |
| 21 | `goto 25` | converge |
| 24 | `iconst_0` | false producer |
| 25 | `putfield MixedShortCircuitField$Box.result:Z` | consume receiver and boolean value |
| 28 | `return` | method end |

At BCI 25 the earlier receiver remains at operand-stack depth 0 and the short-circuit value Phi is at depth 1. The Phi has one direct value use: the value operand of this `putfield`. The field instruction also reads the receiver operand, so its complete SSA read set necessarily includes both values. `javap.txt` records the exact instruction bytes/descriptor; `sha256.txt` binds them to the frozen fixture.

## Runtime contract: all 16 paths

The Runner iterates `a/b/c` masks 0 through 7, once with a non-null receiver and once with a null receiver. Output columns are `nullBit:mask:outcome:stored:bCalls:cCalls:receiverCalls:receiverState`; the complete source output is `original-run.txt`.

For all eight non-null cases, `receiverCalls` is 1, `bCalls` is 1 exactly when `a` is true, and `cCalls` is 1 exactly when `a` is false or `b` is false. The field equals `(a && b) || c`. For all eight null cases, the call count columns are identical to their non-null counterparts, the assignment throws `NullPointerException`, and the field retains its initial false value. Thus receiver evaluation happens once before the RHS, while the null dereference from the eventual `putfield` occurs after RHS evaluation. `original-run.txt` is the reference contract for every comparison.

## Three-way baseline

- **Original JVM:** compiled with `javac --release 8 -g:none -Xlint:-options`; `java -Xverify:all` completed the 16-case Runner. The frozen outer class and nested Box class are retained in the fixture.
- **JADX 1.5.6:** emits `target(z2).result = (z && b()) || c();`. The entire input jar, including nested Box and Runner, was decompiled; package lines were removed only in the isolated compile copy. Java 8 recompilation and `-Xverify:all` passed. `jadx-run.txt` is byte-for-byte equal to the original trace, and `jadx-javap.txt` shows the same BCI/opcode sequence for `one`, so this output neither duplicates receiver evaluation nor moves the null fault before RHS evaluation.
- **Jarde stable snapshot:** root supplied `/tmp/jarde-sept25-final-target/debug/jarde-cli`, a gateway-complete binary from before the local-values work; this evidence intentionally does **not** call it the latest CLI. Its executable SHA and identity note (`jarde-cli-snapshot.txt`), invocation logs, JSON report, complete outer-class text, and a separate nested-Box class presentation are saved here. `one(ZZ)V` is `quality=fallback`, `representation=mixed`, `content=explanation_only`, `syntax_status=not_java`, `semantic_validation=unproven`, `verification=not_performed`, with `jre_region_ownership_overlap`: canonical BCI 20 has more than one owner, so the complete method is quoted. The source text contains the full BCI quote and no executable statements for `one`.

Jarde's complete outer class-source text alone does not compile because the separately presented nested class is not bundled into it. Compiling that outer text with the same CLI's nested-Box presentation and the retained behavior Runner succeeds under Java 8; `jarde-run.txt` then shows all 16 cases as `OK:false:0:0:0`, including null receivers (no NPE). This is a compilable explanation-only refusal, not recovered behavior. The Jarde Runner uses a top-level source name `MixedShortCircuitField$Box` to match the emitted binary descriptor; it is retained under `jarde-src/` and is not the original fixture Runner.

A concurrent current-source CLI build was attempted while local-value work was incomplete and failed before execution because `ShortCircuitConsumer::Local` lacked a match arm. The failure was not used as the Jarde baseline. Root plans to rerun the completed current CLI and append that result separately; until then, this is only the explicitly hashed stable snapshot.

## Boundary from code inspection

The observed Jarde rejection is the Region ownership overlap above. It occurs before this fixture can tell whether the lower consumer proof accepts `putfield`; do not present the following source inspection as the observed refusal reason.

In the checked source, `Region::short_circuit_value` also restricts its consumer-anchor list to static field writes, `ireturn`, calls and local stores. It does not include `putfield`, so even if its preceding graph walk passed, this anchor check would return `None`; ordinary `if` construction may then re-own the shared producer. Source inspection alone has not excluded an earlier failing condition, so the first cause of the observed BCI 20 overlap still needs a Region trace. The private `ShortCircuitConsumer::Field` proof accepts `0xb3 putstatic` with descriptor `Z`, and rejects other field opcodes through its field-refusal arm. The unique-Phi consumer check also requires the consuming instruction's complete `reads()` to equal only `(phi stack slot, phi value)`, which does not model the receiver-plus-value reads of `putfield`. The consumer builder further requires `evidence.is_static` and `shape.receiver.is_none()`. These are independent static-only code gates that would remain after resolving the earlier ownership overlap.

`StmtKind::FieldAssign` and its emitter already support `receiver.field = value`; the emitter writes receiver before the right-hand expression. That syntax can preserve this sample's order and delayed null check when the receiver expression is proved to be owned by this field assignment and emitted once. Future admission therefore needs a distinct receiver/value proof: match the field plan's exact identity and `Z` descriptor; match the receiver SSA value to the operand at stack depth 0 and the Phi to the `putfield` value operand at depth 1; establish the Phi's unique direct value use and the receiver's unique expression owner; reject any receiver with a second read or independently emitted effect. Do not split the receiver into an earlier statement unless a separate proof establishes that doing so preserves both exception and RHS order.

The first future task must investigate why the sample's BCI 20 canonical block has overlapping Region owners and whether that overlap is a genuine structural conflict. Do not bypass it merely to reach the field-consumer branch. The new OpenSpec keeps the overlap investigation, the instance receiver proof, and the null/RHS behavior matrix explicit.

The root agent rebuilt the CLI after the local Boolean Store implementation and saved its [current report](jarde-after-local-root-report.json) and [complete outer class](jarde-after-local-root-MixedShortCircuitField.java). `one` still has `fallback/mixed/explanation_only`, `jre_region_ownership_overlap` at BCI 20, and the same 13 decoded starts in its whole-method quote. The local consumer change did not resolve this instance-field boundary; the original 16-path baseline remains the acceptance target.

## 2026-09-25 Region/SSA trace addendum

The preceding source-inspection paragraphs describe the uncertainty **before** the bounded trace. [The Region/SSA trace](region-trace.md) now establishes the first failed gate: `short_circuit_value` has collected the closed test/producer/consumer graph, but its consumer-anchor list rejects BCI 25 `putfield`. Only then does the generic `if` walk claim BCI 20 twice. The owner validator's overlap finding is a correct rejection of that generic tree, not a reason to waive ownership or a structural impossibility for the dedicated candidate. The implementation plan now requires exact two-stack `putfield Z`/receiver proof before the dedicated short-circuit region can claim the graph once. The frozen baseline reports above remain unchanged.
