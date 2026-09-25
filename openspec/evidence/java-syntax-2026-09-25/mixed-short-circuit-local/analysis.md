# Mixed short-circuit value stored in one local

The [frozen Java source, class and Runner](../../../../tests/fixtures/p3-conditional-values/mixed-short-circuit-local/README.md) establish a single `one(Z)Z` positive example. `javac --release 8 -g:none` produced class SHA-256 `8901ea8d51475567257d20b089c193fa4bd098fe70dc930755004cb72fca231b`; the frozen class was compared byte-for-byte with the fresh compiler output. `java -Xverify:all` ran all eight combinations of `a`, `bValue` and `cValue`. The [original trace](original-run.txt) and the trace from the recompiled [JADX 1.5.6 class](jadx-run.txt) are identical.

The [`javap -c -v -p` listing](javap.txt) for `one` has descriptor `(Z)Z` and this value path:

```text
 0 iload_0
 1 ifeq 10
 4 invokestatic b:()Z
 7 ifne 16
10 invokestatic c:()Z
13 ifeq 20
16 iconst_1
17 goto 21
20 iconst_0
21 istore_1
22 iload_1
23 putstatic result:Z
26 iload_1
27 ireturn
```

The two producers meet on the operand stack immediately before BCI 21. That `istore_1` is the Phi's only direct consumer: the later field write and return first execute `iload_1` at BCIs 22 and 26, so they read the local value produced by the store. The local is lexically initialized once before both reads; there is no debug local table because the class was built with `-g:none`. The remaining proof obligation is therefore to establish that the slot is a named Java `boolean` in a declaration scope containing both loads, rather than to treat the loads as additional Phi consumers.

The [eight original rows](original-run.txt) encode `mask:a,b,c:value:result:bCalls:cCalls`. The return value and field value agree on every row; `b()` runs iff `a` is true, and `c()` runs iff `a` is false or `b()` returns false. This is the observable check that storing the expression once and reading it twice does not repeat either RHS.

根代理独立复核冻结 class 的 SHA-256 `8901ea8d51475567257d20b089c193fa4bd098fe70dc930755004cb72fca231b`、原源码的 Java 8 字节级重编、原/JADX 完整类重编与 `java -Xverify:all` 八行逐字一致；Jarde 10:34 的基线仍待其他共享源码变更稳定后从最新 CLI 再导出。

JADX 1.5.6 emits `boolean z2 = (z && b()) || c();`, then uses `z2` for the field write and return. Its source is saved as [jadx-MixedBooleanLocal.java](jadx-MixedBooleanLocal.java). Only JADX's generated `package defpackage;` line was removed in a temporary recompilation copy; the complete class and Runner passed Java 8 compilation and `java -Xverify:all`, with no difference from the original trace.

The saved Jarde class-source report was generated from the shared worktree at **10:34** on the recorded `a83b5572a93dd316dac1eb7697208fecd6a6fdbc` HEAD. In that build, `one` is `quality=fallback`, `representation=mixed`, and `content=explanation_only`. Its body quotes all decoded BCIs and says the chain through BCI 13 reaches a shared value consumer at BCI 23 but has no SSA proof. BCI 23 is the later `putstatic`; the Phi itself is consumed by the intervening BCI 21 local store. The [complete Jarde class](jarde-MixedBooleanLocal.java) therefore has no method return statement, and the [Java 8 compiler log](jarde-javac.log) records `javac` rejecting the missing return. There is no Jarde execution trace because the emitted class did not compile; this is a pre-later-edit snapshot, not a claim about the current shared tree. `region.rs` and `build.rs` were modified after the report was written; their exact 10:34 source contents were not archived. The root agent will refresh only the Jarde comparison after the concurrent source work is delivered.

网关变更完成后，根代理又从最新 CLI 独立导出[完整报告](jarde-after-gateway-report.json)和[完整 Java 类](jarde-after-gateway-MixedBooleanLocal.java)。`one` 仍为 `fallback/mixed/explanation_only`，同一 14 个指令起点进入完整引用；将完整类按公共类名保存并用 `javac --release 8` 重编仍因该方法缺少 return 而失败，[编译日志](jarde-after-gateway-javac.txt)留存。这证明局部消费者缺口没有被网关扩展偶然修复，为独立 `recover-short-circuit-local-values` 实施提供了当前基线。

局部消费者实现后，根代理重新构建 CLI，保存[全证据](jarde-after-local-report.json)、[完整类](jarde-after-local-MixedBooleanLocal.java)和 [JVM 八行输出](jarde-after-local-run.txt)。`one` 为 `structured/java/contains_statements`：只声明一次 `boolean local1`，BCI 22 的字段写入和 BCI 26 的返回都读取该名字，不重发 RHS。14 个已解码指令起点均有 source map；完整类在 Java 8 下重编，`java -Xverify:all` 八行与原 class 逐字一致。[独立验收](../../../changes/recover-short-circuit-local-values/verification.md)保留负例最早拒绝点和未单独隔离的类型/作用域门，不能把 7/8 任务写成全部闭合。

This consumer is distinct from the single `(Z)V` invocation consumer in [the argument case](../mixed-short-circuit-argument/analysis.md): there the call reads the Phi directly, while this case requires one local write, a Boolean type and scope proof, and later name reads. The new [recover-short-circuit-local-values](../../../changes/recover-short-circuit-local-values/proposal.md) plan keeps those consumer contracts separate. The proposal is not an implementation result.

Reproduction commands from the repository root:

```sh
F=tests/fixtures/p3-conditional-values/mixed-short-circuit-local
javac --release 8 -g:none -Xlint:-options -d /tmp/mixed-local "$F/MixedBooleanLocal.java" "$F/Runner.java"
cmp "$F/MixedBooleanLocal.class" /tmp/mixed-local/MixedBooleanLocal.class
java -Xverify:all -cp /tmp/mixed-local Runner
javap -c -v -p "$F/MixedBooleanLocal.class"
jadx -d /tmp/mixed-local-jadx "$F/MixedBooleanLocal.class"
CARGO_TARGET_DIR=/tmp/jarde-mixed-short-local-target cargo run -q -p jarde-cli -- class-source --input "$F/MixedBooleanLocal.class" --policy single-class --class MixedBooleanLocal --evidence all --format json
```
