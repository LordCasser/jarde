# CF16 switch inside catch evidence

修复后的三方 Java 8 重编、九路径 `-Xverify:all` 运行、来源和拒绝边界验收见 [`after/README.md`](after/README.md) 与 `after/results.txt`。以下文字和 `baseline/` 保留修复前的冻结结果。

This fixture isolates the fixed JADX `TestTryCatchFinally12.TestCls.runTest(II)String` control-flow shape. The minimal class retains the original `test1/2/3`, `call`, and `runTest` bodies needed by the runner; its `runTest` BCI, opcodes, and exception row compare equal to the fixed class. The verified row is `[11,61) -> 64 IllegalArgumentException`; BCI 12 is `tableswitch`, BCI 61 is `goto 79`, and BCI 79 begins the return sequence. The per-method comparison is in `baseline/method-shapes.txt`.

`replay.sh` recompiles the minimal source with Java 8, runs the original and JADX presentations under `-Xverify:all`, and separately checks the pinned original class against its pinned JADX source on all nine `runTest` inputs. Both original/JADX pairs match. Jarde reports `runTest` as explanation-only with `jre_region_ownership_overlap`; `test1/2/3` have recovered statements and no fallback in both the fixed class and the minimal class. The Jarde Java presentation cannot compile because the refused non-void method has no return statement (javac exit 1), so its runtime status is `not_run`.

The fresh CLI used for the frozen run was built from checkout `5bf6d91723af706f8692fa03b8ade6876965a5da` in `/private/tmp/cf16-switch-cargo-target`; its SHA-256 and the two class input hashes are recorded in `baseline/toolchain.txt` and `baseline/input-sha256.txt`. Exact CLI invocations and complete method reports are retained in `baseline/jarde-invocations.txt`, `baseline/jarde.report.txt`, and `baseline/pinned-jarde.report.txt`. The pinned TestCls input SHA-256 is `d9b9cb676203d943ee3cf97d66e62dda2a637125d7f737be1e22ca275c209185`.

Four near-negative classes are Java 8 compiled, verifier-valid, and executed through `runTest(1,0)`. Three protect only one switch arm at a time; the fourth wraps the switch in a real try-with-resources construct. Their exact `runTest` exception rows are listed in `baseline/variant-shapes.txt`. These demonstrate partial-arm and real-resource boundaries; no invalid class variant is counted.

Replay using a fresh Jarde CLI and the pinned JADX checkout:

```sh
openspec/evidence/java-syntax-2026-09-28/cf16-switch-catch/replay.sh /path/to/jarde-cli /tmp/cf16-switch-catch-replay
```

The replay script does not build Rust. For this captured run, Cargo used the isolated target directory named above; it must be cleaned after replay.
