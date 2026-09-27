# DT-28 primitive conversion audit

The audit found one bounded Jarde gap before this change: a byte-valued conditional passed to a `byte` invocation parameter. The implementation closes that tested slice; the other tested conversion and conditional shapes remain passing. The fixed JADX output compiles and behaves like the original in both fixture groups, so no JADX defect was observed here.

## Frozen JADX evidence and production path

The inventory row is `DT-28` in `openspec/evidence/jadx-feature-inventory-2026-09-27/declarations-types.md`: primitive narrowing/widening and casts in conditional expressions. It lists `types/TestPrimitiveConversion.java`, `types/TestLongCast.java`, `TypeInferenceVisitor.java`, and `InsnGen.java`.

`TestPrimitiveConversion` is a smali test. Its embedded expected source is `putByte(j, z ? (byte) 1 : (byte) 0);`; the active assertion rejects `putByte(j, z);` and requires that casted call. `TestLongCast` is a Java integration test run without debug info; it requires `(long) c << 32` and exercises `(int) l >> 2`. These are the specific positive shapes audited, not evidence for every numeric conversion.

The fixed JADX checkout is `/Users/lordcasser/workspace/testzone/jadx` at `2fb1b16386941660fda07e9017285aec40fcb37f`, clean at replay time. In the pinned implementation, `InsnGen` dispatches `CAST` through its cast emitter and `TERNARY` through `makeTernary`, which writes its two arms. The replay confirms the exact byte-arm casts remain in JADX source. No general conclusion about every `TypeInferenceVisitor` path is made.

## Fixtures and observed results

The evidence has two independently rebuilt fixture groups. `supported` covers `(long) char` before left shift, `(int) long` before right shift, long-to-byte, int-to-short and int-to-char narrowing, a byte-valued conditional return, and an int/long conditional promotion control. The byte-call group models the active smali assertion with a `byte` return/parameter and `condition ? (byte) 1 : (byte) 0` passed to that parameter. The original classfile listing records its call descriptor as `(JB)B`, with the conditional's `iconst_1` / `iconst_0` arms joining at the `acceptByte(JB)B` invocation.

Each group is compiled independently from original source, complete JADX source, and complete Jarde class-source output with `javac --release 8`; successful builds run under `java -Xverify:all`. The supported group prints the same `94489280512`, `2`, `2:2:2`, `1:0`, `3:4`, and `ok` output in all three versions. Jarde's text differs while retaining semantics: for example, it emits `(byte) (arg0 ? 1 : 0)` for the byte return, `(byte) (int) arg0` for the long-to-byte narrowing, and `arg0 ? (long) arg1 : arg2` for int/long promotion.

For the byte-call group, original, JADX and post-fix Jarde sources compile with `--release 8`, pass `java -Xverify:all`, and print `1:0`. JADX preserves `acceptByte(j, z ? (byte) 1 : (byte) 0)`. Jarde presents `return acceptByte(arg0, arg2 ? (byte) 1 : (byte) 0);`; its two new casts are presentation choices, not recovered `i2b` instructions. The output retains the condition, arm, and call origins in source-map segments.

Before the change, the stop mapped to `Builder::invocation_argument` in `crates/jarde-java/src/build.rs`: the conditional presented as `int`, missed the direct `narrowed_constant` check, and reached the primitive conversion check where `int` to `byte` is `Unspellable`. The preserved diagnostic is in `pre-fix-jarde-diagnostic.txt`, and the complete pre-fix source is retained in the baseline commit `f39211c27c326df3b9f471970102a943627833b4`.

The original source's casted arms form a byte-valued conditional under Java 8, but javac emits only `iconst_1`/`iconst_0` here, with no `i2b`; the classfile does not prove the original cast spelling. The fix therefore requires both a `B` invocation descriptor and independent in-range proof for each int constant arm. It adds source casts to those arms without treating them as original bytecode instructions. Out-of-range constants, int local arms, a short descriptor, and an extra join predecessor still refuse atomically; budget exhaustion and cancellation publish no partial text or source map.

## Conclusion and boundary

- **Caught up in tested slices:** both `TestLongCast` conversions, the direct primitive narrowings listed above, a byte-valued conditional return, and mixed int/long conditional promotion pass full-source rebuild and verified runtime comparison.
- **JADX:** no failure in either audited group; it emits compilable source with the expected cast spellings.
- **Jarde fixed slice:** the byte conditional invocation and all support controls pass full-source rebuild and verified runtime comparison. Before/after results and the saved refusal diagnostic are in `outputs/results.json`.

This does not establish coverage for arbitrary cast expressions, nonconstant conditional arms, short/char conditional arguments, floating-point conversions, overload interactions beyond the descriptor in this fixture, or all sub-shapes in DT-28.

## Replay

From the repository root, run:

```sh
python3 openspec/evidence/java-syntax-2026-09-27/dt28-cast-audit/replay.py
```

The script verifies the pinned JADX commit and clean checkout, compiles each Java 8 original fixture, saves classfile `javap -c -p -s`, extracts all source declarations from JADX and Jarde, then recompiles and runs each complete source set. It builds Jarde using its own temporary `CARGO_TARGET_DIR`, removed on exit. Frozen input sources, complete source outputs, Jarde JSON reports, structured before/after results, and their SHA-256 manifest are in `outputs/`.

`cargo test --workspace --all-targets --all-features --locked` was attempted separately and stops while compiling the pre-existing `anonymous_allocation_candidates` integration test: its call to `class_source_anonymous_return_site` destructures two tuple elements although the current API returns three. The exact compiler diagnostic is saved in `full-workspace-test-blocker.txt`; it is independent of this change. `cargo check --workspace --locked`, `cargo fmt --check`, the four adjacent Java recovery integration test files (70 tests total), and strict OpenSpec validation pass.
