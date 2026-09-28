# CF-16: JADX `TestEmptyFinally.TestCls.test(FileInputStream)`

实现后的四方行为重放、三份 verifier 有效近邻及验收结果见 [implementation-results.md](implementation-results.md)。原始固定捕获和失败报告保留在本目录，便于对照实现前后的结果。

This evidence pins the upstream test and its built nested class, compares the complete source Jarde and JADX emit for that class, and runs the same injected `FileInputStream` behavior probe against the fixed class, an exact Java 8 source extraction, and JADX's source. It contains no production or OpenSpec implementation change.

The pinned JADX checkout is `/Users/lordcasser/workspace/testzone/jadx` at `2fb1b16386941660fda07e9017285aec40fcb37f`. Its upstream test is `jadx-core/src/test/java/jadx/tests/integration/trycatch/TestEmptyFinally.java`, SHA-256 `258da505cd3729bfcf4e54598314fff43d838f7ca3f8cc536c206f6ae790cc3f`. The built target `TestEmptyFinally$TestCls.class` is major version 52, SHA-256 `dcf5de9a4037ddd2169f103e426be38fac60dba8fb2d5cb38dc6ffe3c648018a`. Both are copied under `original/` so replay does not select a different build output. The fixed complete class disassembly is `original.fixed.javap.txt`.

`original/TestEmptyFinally$TestCls.java` is the target method extracted from the upstream source as a standalone class named after the classfile. `javac --release 8 -g` produces major version 52 class `original.standalone.class`, SHA-256 `6623b834b01f7114b0303dda40a60fabf003f78998ad46026f5855d0b4f2e001`. Its `test(FileInputStream)` instruction sequence and exception rows are byte-for-byte equivalent at the method-shape level to the fixed class: `0 aload_1; 1 invokevirtual FileInputStream.close; 4 goto 14; 7 astore_2; 8 goto 14; 11 astore_3; 12 aload_3; 13 athrow; 14 return`, with `[0,4) -> 7 IOException` and `[0,4) -> 11 any`. See `bytecode-shape.txt` and both full `javap` captures.

The fixed JADX source is `pinned.jadx.java`, SHA-256 `d5dacf0e3f68da00f03ca44c681662397b574b25415c638d02e87a598d571d77`. It keeps `try { f1.close(); } catch (IOException e) {}` and drops the empty `finally`. Its Java 8 class has only the named IOException row; its full disassembly is `pinned.javap.txt`. Jarde's complete output and report are `jarde.java.txt` and `jarde.report.txt`. The method report contains `jre_region_exception_edge` and `jre_region_uncovered_blocks`; the emitted source comments the protected `close()` block as unrecovered, keeps the named empty catch, emits a `return`, and marks BCI 11–13 as an uncovered exceptional-only live block. That source is not a valid Java 8 compilation: `jarde-javac.stderr` records javac rejecting the `IOException` catch because the commented-out body cannot throw the checked exception. Since the source does not compile, the evidence deliberately has no Jarde `-Xverify:all` behavior trace.

`BehaviorProbe.java` injects a `FileInputStream` subclass. The override calls `super.close()` once, then either succeeds, throws `IOException`, or throws `IllegalStateException`; the target method must swallow the checked exception, propagate the runtime exception, and call close once on each path. `behavior.fixed-original.txt`, `behavior.standalone-original.txt`, and `behavior.pinned-jadx.txt` are identical. Each was run with `java -Xverify:all`; the fixed target class, the standalone extraction, and the pinned JADX source were compiled or loaded with Java 8-compatible bytecode. The probes demonstrate all three outcomes and the close invocation count. They do not claim that the Jarde source has behavior, since javac rejects it.

## Existing Guard/Region path

The recovery needs a narrow proof for the extra `any` exception-table row, not a new CFG, Region kind, source emitter, or general decompilation path. `RegionWalker::try_region` already asks `guard::catches` for ordinary named catches and recursively presents each named handler. `catches` builds clauses from named rows; this fixture has two rows, one named `IOException` row and one `any` row, so its single-row `exception_only_catch` path does not apply. The ordinary region walk then encounters the protected block's exceptional edge and leaves the catch-all handler at BCI 11 outside the claimed source structure.

There are related rethrow checks, but their contracts exclude this shape. `finally_copy` recognizes an `any` handler that stores an exception, performs nonempty cleanup, reloads that same value, and rethrows it; here the handler is only `astore; aload; athrow`, with no cleanup. `exception_only_catch` handles exactly one catch-all row and requires a specific nonempty static integer cleanup plus a throwing protected tail; this fixture has a named row too and its protected operation is `close()`. The `shared_finally_candidate` call runs before ordinary catch reading, but the fixture has no cleanup operation to prove as a finally. A future fix can reuse `Facts`, handler-local binding, and SSA value-identity checks from those existing proofs, but needs a narrow certificate for a same-range, effect-free, unchanged-Throwable rethrow and a way for the ordinary catch reader to consume its handler as compiler scaffolding. No broader mechanism is needed.

## Replay

Build the pinned Jarde checkout into a disposable target, then run:

```sh
CARGO_TARGET_DIR=/tmp/cf16-empty-finally-cargo-target cargo build -p jarde-cli --locked --release
./replay.sh /tmp/cf16-empty-finally-cargo-target/release/jarde-cli /tmp/cf16-empty-finally-replay
```

The replay checks the frozen source/class/JADX revisions, compiles with `javac --release 8`, captures complete original/JADX/Jarde sources and bytecode, runs the three behavior probes with `-Xverify:all`, and records the expected Jarde source compilation failure. The temporary Cargo target used to produce the captured CLI was removed after replay; the target path in the command is disposable.
