# CF16 nested-finally evidence

This directory contains a minimal complete top-level reproduction and a replay script. The Java 8-target fixture and reflection runner exercise all nine `(test1..test3, exception 0..2)` paths.

`TestTryCatchFinally12.java` and its fixed nested class file are copied from the pinned JADX test fixture. The nested class SHA-256 is `d9b9cb676203d943ee3cf97d66e62dda2a637125d7f737be1e22ca275c209185`. `three-method-bytecode.txt` preserves the fixed class's `test1`/`test2`/`test3` BCI, opcode, and exception-table excerpt. Replay compiles the top-level sample with `javac --release 8` and proves all three methods match those BCI/opcode/table shapes; see `bytecode-shape-check.txt`.

The pinned JADX checkout is `2fb1b16386941660fda07e9017285aec40fcb37f`, with binary version `dev`. `TestTryCatchFinally12-TestCls.jadx-default.java` has three extracted finally clauses. The pinned `--no-finally` output in `TestTryCatchFinally12-TestCls.jadx-no-finally.java` has seven cleanup copies, matching the fixture's `testWithoutFinally` assertion; `control-counts.txt` records the count contract. The minimal fixture's JADX output is saved as `FinallyMinimalProbe.jadx.java`.

The current Jarde baseline report is `jarde-baseline-report.txt`; the corresponding source presentation and runtime output are `jarde-baseline.java.txt` and `jarde-baseline.run.txt`. All three test methods are explanation-only: Jarde says at BCI 42/32/42 that the exceptional path repeats normal-path finally code but lacks a complete copy/range/ownership proof. The presentation compiles, yet the first path produces an empty string instead of `call-out-finally`. Baseline execution records original/JADX success and byte-identical nine-line stdout, while Jarde exits at runtime with an assertion failure; `baseline-results.txt` captures the statuses and equality result. The full report is 200 KB and retained because it includes the per-method BCI diagnostics.

`test3` 的三副本首片实现、定向负例与重放见 `test3-implementation.md` 和 `replay-test3.sh`；[root 独立验收](test3-root-acceptance.md)确认三路径已与原/JADX 一致。`test1/2` 的两副本实现、八个 verifier 有效负例和完整九路径重放见 `test12-implementation.md` 和 `replay-test12.sh`。上述 baseline 文件保留修前结果；本次完整最小类三方重放的原/JADX/Jarde 九路径一致，固定完整类的 `runTest`/family 仍为独立债务。

Replay with the requested CLI path; pass an output directory to retain a fresh run. Without the second argument, outputs go to a new temporary directory outside the repository. The script records the Jarde checkout HEAD, CLI version and binary SHA without pinning them, verifies the pinned JADX checkout and binary, recompiles the original and JADX sources with `javac --release 8`, runs each under `java -Xverify:all`, asserts their nine-path stdout matches, records Jarde compile/run outcomes, and regenerates the three/seven-clause JADX control. It does not build Rust code.

```sh
./replay.sh /private/tmp/jarde-cf16-nested-root-target/debug/jarde-cli /tmp/cf16-nested-replay
./replay-test12.sh /private/tmp/jarde-cf16-nested-root-target/debug/jarde-cli /tmp/cf16-nested-test12-replay
```
