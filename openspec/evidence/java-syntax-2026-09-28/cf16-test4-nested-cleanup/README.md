# CF-16: `TestTryCatchFinally4.TestCls.test()V`

This fixture is scoped to the single pinned JADX test method `TestTryCatchFinally4.TestCls.test()V`. It freezes the JADX source and compiled class, captures the complete class-source outputs from pinned JADX and Jarde, verifies a Java 8 source transcription against the pinned method's BCI/opcode/exception-row shape, and includes an explicitly separate helper-seam control for the cleanup paths.

The pinned JADX checkout is `2fb1b16386941660fda07e9017285aec40fcb37f`. The source file `original/TestTryCatchFinally4.java` has SHA-256 `57b52e955fdf2db03a6949b02acc3ce9a096f939ed33c44a05c5fc36b8e0e846`. The nested target class is major version 55 with SHA-256 `2bf1b8932e521aade29fa1d562269903f9b8e9942f86981eb458d4f552d67060`. Its exception rows are `[22,31) -> 34 IOException`, `[17,22) -> 38 any`, `[40,49) -> 52 IOException`, and `[38,40) -> 38 any`. `original.javap.txt` is the frozen `javap -c -v` output of that pinned class.

`original/TestTryCatchFinally4$TestCls.java` is the exact target method extracted into a standalone top-level class so it can compile without the JADX test framework. Compiling it with `javac --release 8` preserves the fixed method's BCI/opcode sequence and exception rows; `bytecode-shape.txt` records the check. The Java 8 output class SHA-256 from the captured replay is `4918d1b726506100c4b9213d2c7bbdf0fbffb38de7e4d7ee679de8b58e3f0a0d`.

The pinned JADX complete class source reconstructs the method as a body `try` with a nested cleanup `try/catch` in `finally`. Jarde's captured complete class-source presentation has no recovered `test()` statements. Its report exposes both the initial recovery refusal and the downstream source-scope refusal: the fallback diagnostics are `jre_guard_handler` at BCI 38 and `jre_region_uncovered_blocks` for five live blocks; the final explanation says local 1 crosses a quoted fallback region and cannot be bound as one lexical definition-use slice. Preserve both levels when interpreting this case: the initial guard/region proof fails first, and the local-binding message describes why the fallback presentation also cannot be emitted as ordinary statements.

The fixed target directly creates a `FileOutputStream`, so the pinned method offers no injection seam for forcing a body `IOException`, cleanup `IOException`, or cleanup `RuntimeException`. Its original source, pinned JADX output, and Jarde presentation compile with `--release 8` and verify-run with `java -Xverify:all`, but the target runner only observes normal completion; that result does not prove behavior recovery. The separate `Control.java` helper-seam probe drives four paths and logs operation order. Original and pinned JADX outputs match for normal completion, body `IOException`, cleanup `IOException`, and cleanup `RuntimeException`. Jarde's explanation-only control compiles as an empty method and produces empty event logs, so it does not match. This control exercises the source-level nesting semantics only: it has different bytecode and is not evidence that Jarde recovered the pinned `TestCls.test()` behavior.

## 修复后独立回放

上文的 `jarde.java.txt`、`jarde.report.txt` 与旧运行记录保留修复前的两级拒绝基线。修复后的 fresh CLI 输出保存在 [`recovered/`](recovered/)：固定目标的原始、JADX、Jarde 三份完整 Java 8 源码均可重编，并在 `-Xverify:all` 下正常运行一致；Jarde 的 `test()` 为 `structured`，只有一份 `finally { try { close(); delete(); } catch (IOException ...) {} }`，source map 覆盖全部 31 个物理 BCI。原始源码与固定 JADX 全文仍在本目录的 `original/` 和 `pinned/`，修复后的 Jarde 全文及 report 在 `recovered/`。目标的异常路径仍没有可注入入口，因此没有将 control 的动态结果写作固定目标的动态证明。

独立 `Control.run` 的 Java 8 异常表只有三行，和固定目标的四行不同。修复后 control 的原始与固定 JADX 输出在六条路径一致，其中新增 `body-io+cleanup-io` 保留正文原异常，`body-io+cleanup-runtime` 由清理未检异常覆盖正文原异常。Jarde 对此三行形态继续安全拒绝。`replay.sh` 按这些分层断言重新运行，旧捕获文件不被覆盖。

`mutate-neighbors.py` 从固定 class 确定性生成四个 verifier 有效近邻：`close→flush`、`delete→exists`、`IOException→Throwable`、首个 named catch 起点 `22→26`。`check-neighbors.sh` 使用 fresh CLI 和 JVM 验证四者均拒绝唯一 finally 投影；`delete→exists` 的正常运行留下临时文件，其余三者无残留。另一个可注入、字节码不同的 `negative/WrongReceiver.java` 在正文异常时关闭 `other` 而不是 `first`，运行事件可区分，Jarde 同样拒绝。覆盖缩窄变异在固定目标的正常路径无动态差异；其异常差异由异常表和独立 control 的清理 `IOException` 路径说明，未宣称已动态触发固定目标。

Run the deterministic replay with a Jarde CLI built from this checkout and a fresh output directory:

```sh
./replay.sh /path/to/jarde-cli /tmp/cf16-test4-replay
```

The script verifies the pinned class/source hashes and JADX HEAD, compiles with `javac --release 8`, compares the target bytecode shape, decompiles with pinned JADX, invokes Jarde `class-source`, and runs the target and control variants with `java -Xverify:all`. The output directory retains the regenerated Jarde reports and run transcripts. It does not modify production code or build Rust artifacts. 近邻另以 `./check-neighbors.sh /path/to/jarde-cli /tmp/cf16-test4-neighbors` 重放。

root 在合入 Test11 后用主线 fresh CLI 独立重放了 `replay.sh` 与 `check-neighbors.sh`：固定目标 31/31 BCI 来源齐全，原/JADX/Jarde 完整 Java 8 源码重编及正常路径验证运行一致；四个固定 class 变异和一个独立错接收者控制均为 verifier 有效且 Jarde 拒绝。原/JADX 三行 Control 的六条异常路径一致，Jarde 仍拒绝。主线 `cargo test -p jarde-java --tests --locked`、`cargo check --workspace --locked`、fmt 和两个并行 OpenSpec strict 均通过，Test11 的三方回放也再次通过。验收范围仅是固定四行嵌套清理形态：固定目标没有可注入异常入口，Control 的六路径不能冒充目标方法的异常运行证明。
