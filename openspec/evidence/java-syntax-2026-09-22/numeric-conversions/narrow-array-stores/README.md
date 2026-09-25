# Narrow integer array-store audit

This is source-only evidence for the verifier-valid bytecode shape that Java source cannot express: each selected method is compiled as an `int[]` store (`iastore`), then a small class-file patch changes only its method descriptor and that method's one store opcode. The source has no explicit `i2b`, `i2c`, or `i2s` conversion. `NarrowArrayStoreEffects.value` is a normal or throwing producer so the audit observes producer-before-store ordering, null handling, and bounds handling.

The frozen CLI was `/tmp/jarde-cli-deferred-interim-948a`, with SHA-256 `948a6f9b7db009fbb89a792f83328c9ab63e42ac0cb62e691d97bec28c2c8ba0` both before and after the audit. `run_audit.py` is the replay entry point. It uses a temporary directory that is removed by Python's `TemporaryDirectory`, runs `javac --release 8 -g:none`, `javap`, `java -Xverify:all`, the frozen CLI, and JADX, then stores the source, patch, reports, outputs, statuses, and hashes in this directory. No Cargo command is involved.

## Exact patch

The source class has 13 `Code` methods including its constructor and eight selected `store*` methods. The eight selected methods had the following source shapes and exact one-byte changes:

| method | source descriptor | patched descriptor | Code length | opcode offset | opcode |
| --- | --- | --- | ---: | ---: | --- |
| `storeByte` | `([III)V` | `([BII)V` | 5 | 3 | `iastore` → `bastore` |
| `storeChar` | `([III)V` | `([CII)V` | 5 | 3 | `iastore` → `castore` |
| `storeShort` | `([III)V` | `([SII)V` | 5 | 3 | `iastore` → `sastore` |
| `storeBoolean` | `([III)V` | `([ZII)V` | 5 | 3 | `iastore` → `bastore` |
| `storeByteProduced` | `([IIIZ)V` | `([BIIZ)V` | 9 | 7 | `iastore` → `bastore` |
| `storeCharProduced` | `([IIIZ)V` | `([CIIZ)V` | 9 | 7 | `iastore` → `castore` |
| `storeShortProduced` | `([IIIZ)V` | `([SIIZ)V` | 9 | 7 | `iastore` → `sastore` |
| `storeBooleanProduced` | `([IIIZ)V` | `([ZIIZ)V` | 9 | 7 | `iastore` → `bastore` |

`patch-report.detail.json` contains the per-method constant-pool descriptor index, Code offset, before/after Code hashes, and stack/local sizes. The source class is `e7f92710bf26eb45bb3fb01dcd6e22d0d370d53cd3a9fa28468f51753a9528b5`; the patched class is `8bbb683045fdc2a85ee376383515920331fd15c53710176e447ba93f0d04004b`. The patched class has no `iastore`; its opcode counts are 10 `bastore`, 5 `castore`, and 5 `sastore`, including the ordinary typed-store controls. The Code lengths of all selected methods stay unchanged.

## JVM behavior

`NarrowArrayStoresRunner` executes 21 values (`-32769` through `Integer.MAX_VALUE` at the selected boundaries) for each direct B/C/S/Z store, then normal producer calls, a producer exception, null, and out-of-bounds calls. Every call is independently caught. It emits 196 lines: 92 direct cases, 100 producer cases, and 4 ordinary `javac` controls. Both the original `int[]` class and the patched class pass `java -Xverify:all` with status 0; the full outputs are in `original-runtime.txt` and `patched-runtime.txt`.

The observed narrowing matches the JVM store instructions: byte and short results wrap at their respective widths, char results are zero-extended 16-bit values, and the boolean store keeps the low bit (odd values become `true`, even values `false`). Producer failures happen before a store; `null-producer-fail` and `oob-producer-fail` therefore report the producer exception and one call, while `oob-producer-ok` reports `ArrayIndexOutOfBoundsException` after one call. Ordinary Java source stores compile and run as the typed `bastore`/`castore`/`sastore` forms; their output is the final four lines of the runtime files.

The Java SE 8 JVMS defines `bastore` for byte or boolean arrays and its null/bounds exceptions at [§6.5 bastore](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-6.html#jvms-6.5.bastore), defines truncation to `char` at [§6.5 castore](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-6.html#jvms-6.5.castore), and defines truncation to `short` at [§6.5 sastore](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-6.html#jvms-6.5.sastore). The current JVMS text states the boolean low-bit rule explicitly at [§6.5 bastore](https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-6.html#jvms-6.5.bastore); the Java 8 target class's actual execution above records the same behavior.

## Recovery comparison

The complete frozen-CLI output is in `jarde.java.txt`, with the complete diagnostic evidence in `jarde-report.txt`. The CLI returns status 0 and emits all eight selected methods with an `array_write` recovery refusal: its `meeting_position` only accepts the widening primitive conversion it can prove, so an `int` producer feeding a byte/char/short/boolean array element is rendered as a refusal comment followed by `return;`. This is an internal refusal rather than a silent conversion in the recovered text, but the generated source still compiles (`jarde-javac.status` 0) and the whole-class JVM run is semantically wrong: `jarde-runtime.txt` preserves the initial array values, returns from null and out-of-bounds direct stores, and leaves producer call counts at zero. `patched-runtime.txt` and `jarde-runtime.txt` therefore have different hashes.

JADX output is preserved unchanged in `jadx.java.txt`. Its full class source fails `javac --release 8 -g:none` at the expected eight int-to-byte/char/short/boolean assignments (`jadx-javac.status` 1), so there is no JADX runtime result to claim. The support helper/runner used to attempt that compile is generated under the temporary replay directory and does not alter the JADX source.
