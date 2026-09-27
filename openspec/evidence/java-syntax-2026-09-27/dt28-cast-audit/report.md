# DT-28 primitive conversion audit

This audit found one bounded Jarde gap: a byte-valued conditional passed to a `byte` invocation parameter. The other tested conversion and conditional shapes pass. The fixed JADX output compiles and behaves like the original in both fixture groups, so no JADX defect was observed here.

## Frozen JADX evidence and production path

The inventory row is `DT-28` in `openspec/evidence/jadx-feature-inventory-2026-09-27/declarations-types.md`: primitive narrowing/widening and casts in conditional expressions. It lists `types/TestPrimitiveConversion.java`, `types/TestLongCast.java`, `TypeInferenceVisitor.java`, and `InsnGen.java`.

`TestPrimitiveConversion` is a smali test. Its embedded expected source is `putByte(j, z ? (byte) 1 : (byte) 0);`; the active assertion rejects `putByte(j, z);` and requires that casted call. `TestLongCast` is a Java integration test run without debug info; it requires `(long) c << 32` and exercises `(int) l >> 2`. These are the specific positive shapes audited, not evidence for every numeric conversion.

The fixed JADX checkout is `/Users/lordcasser/workspace/testzone/jadx` at `2fb1b16386941660fda07e9017285aec40fcb37f`, clean at replay time. In the pinned implementation, `InsnGen` dispatches `CAST` through its cast emitter and `TERNARY` through `makeTernary`, which writes its two arms. The replay confirms the exact byte-arm casts remain in JADX source. No general conclusion about every `TypeInferenceVisitor` path is made.

## Fixtures and observed results

The evidence has two independently rebuilt fixture groups. `supported` covers `(long) char` before left shift, `(int) long` before right shift, long-to-byte, int-to-short and int-to-char narrowing, a byte-valued conditional return, and an int/long conditional promotion control. The byte-call group models the active smali assertion with a `byte` return/parameter and `condition ? (byte) 1 : (byte) 0` passed to that parameter. The original classfile listing records its call descriptor as `(JB)B`, with the conditional's `iconst_1` / `iconst_0` arms joining at the `acceptByte(JB)B` invocation.

Each group is compiled independently from original source, complete JADX source, and complete Jarde class-source output with `javac --release 8`; successful builds run under `java -Xverify:all`. The supported group prints the same `94489280512`, `2`, `2:2:2`, `1:0`, `3:4`, and `ok` output in all three versions. Jarde's text differs while retaining semantics: for example, it emits `(byte) (arg0 ? 1 : 0)` for the byte return, `(byte) (int) arg0` for the long-to-byte narrowing, and `arg0 ? (long) arg1 : arg2` for int/long promotion.

For the byte-call group, original and JADX sources compile, verify, and print `1:0`. JADX preserves `acceptByte(j, z ? (byte) 1 : (byte) 0)`. Jarde's report identifies the exact stop at BCI 10 in `run(JZ)B`: the invocation's parameter 1 has descriptor type `byte`, but its conditional argument is presented as `int` and the current recovery layer has no proven conversion. Jarde therefore leaves this method body empty; the complete source set fails Java 8 compilation with `missing return statement`.

The stop maps to `Builder::invocation_argument` in `crates/jarde-java/src/build.rs`: after the direct `narrowed_constant` case, a conditional expression presented as `int` reaches the primitive conversion check, where `int` to `byte` is `Unspellable`. This is a Jarde recovery refusal, not a javac failure of the original source. The original source's casted arms form a byte-valued conditional under Java 8, but javac emits only `iconst_1`/`iconst_0` here, with no `i2b`; the classfile does not prove the original cast spelling. Each in-range constant can instead be rendered as byte at the invocation's proven `B` target without changing the selected descriptor or value.

## Conclusion and boundary

- **Caught up in tested slices:** both `TestLongCast` conversions, the direct primitive narrowings listed above, a byte-valued conditional return, and mixed int/long conditional promotion pass full-source rebuild and verified runtime comparison.
- **JADX:** no failure in either audited group; it emits compilable source with the expected cast spellings.
- **Jarde gap:** the tested byte-valued conditional passed to a `byte` invocation parameter is refused, even though the original and JADX rebuild and run correctly.

This does not establish coverage for arbitrary cast expressions, nonconstant conditional arms, short/char conditional arguments, floating-point conversions, overload interactions beyond the descriptor in this fixture, or all sub-shapes in DT-28. A narrow planning change, `recover-proved-byte-conditional-invocation`, records the proposed behavior; production code is unchanged by this audit.

## Replay

From the repository root, run:

```sh
python3 openspec/evidence/java-syntax-2026-09-27/dt28-cast-audit/replay.py
```

The script verifies the pinned JADX commit and clean checkout, compiles each Java 8 original fixture, saves classfile `javap -c -p -s`, extracts all source declarations from JADX and Jarde, then recompiles and runs each complete source set. By default it builds Jarde using its own temporary `CARGO_TARGET_DIR`, removed on exit; `JARDE_CLI=/absolute/path/to/jarde-cli` reuses an already built binary. Frozen input sources, complete source outputs, Jarde JSON reports, structured results, and their SHA-256 manifest are in `outputs/`.
