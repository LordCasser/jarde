# 多重捕获、返回与 finally 副本的三方对照（2026-09-25）

## 冻结输入与执行结果

[`MultiCatchProbe.java`](MultiCatchProbe.java)和[`Runner.java`](Runner.java)是自行编写的 Java 8 探针；保留的 `MultiCatchProbe.class` 为 `javac --release 8 -g:none -Xlint:-options` 产物，SHA-256 为 `907946397a2bcc29b7aeac3bec911e5269eb53c527727f82c908b156f7d95c92`。同目录的两个 class、`javap.txt`、三方整类文本/报告及编译、执行日志固定了输入和观测。本轮 Jarde CLI 是当前工作树构建的 `/tmp/jarde-multicatch-target/debug/jarde-cli`，SHA-256 为 `c381a1865c53a0e7db32add60b2efab7abbc896d2083d60d4906b0c93bd84aad`；这不是固定提交的发布构建。JADX 使用本机 1.5.6 对相同 class 的输出。

`Runner` 依次执行 mode 0–3，逐次清零 `finallyCalls`；原 class 在 `java -Xverify:all` 下的四行见[`original-run.txt`](original-run.txt)，正常返回与两个捕获出口的 cleanup 次数均为 1。JADX 的整类文本仅移除了工具加入的虚构 `package defpackage;` 后通过 `javac --release 8`；其四行见[`jadx-run.txt`](jadx-run.txt)。mode 0/3 的 `finallyCalls` 为 **2**，其余两个模式为 1，故虽然可编译，JADX 的 `chooseFinally` 在正常出口 2/4 行不等价。它在 `try` 中保留了 `finallyCalls++`，同时又生成一个包含 `finallyCalls++` 的 `finally` 子句；不能把 JADX 当此场景的语义判据。`choosePlain` 的四行则与原 class 相同。

为了让普通 catch 的**整类**重编验收不受未恢复的 `chooseFinally` 阻断，另冻结 [`PlainMultiCatch.java`](PlainMultiCatch.java) 与 [`PlainRunner.java`](PlainRunner.java)。前者的 class SHA-256 为 `ac46b05865c086ccb03eea8e1e1890c3bdff1e45771a71c7acc0579c833b8eef`；[`plain-javap.txt`](plain-javap.txt)确认 `choose(int)` 的 BCI 0–66 和 `[0,32)→33` 双命名行与组合探针的 `choosePlain` 形状相同。原 class 与仅移除虚构 package 的 [JADX 完整类](plain-jadx-PlainMultiCatch.java)都经 Java 8 重编，四行 `-Xverify:all` 输出在[`plain-original-run.txt`](plain-original-run.txt)与[`plain-jadx-run.txt`](plain-jadx-run.txt)逐字一致。后续 Jarde 的整类正向验收以这份独立输入为准；组合探针继续保留 typed-catch/finally 相邻拒绝证据。

当前 Jarde 的 `choosePlain` 为 `fallback/mixed/contains_statements`，报告 `jre_region_exception_edge`：已有 `catch (IllegalArgumentException | IllegalStateException)`，但正常 `return "ok"` 的 BCI 30/32 被引用，方法缺少返回。`chooseFinally` 为 `fallback/mixed/explanation_only`，含 `jre_guard_finally_copy` 和 `jre_region_uncovered_blocks`，没有可执行正文。保存的[`jarde-MultiCatchProbe.java`](jarde-MultiCatchProbe.java)经 Java 8 重编产生两个“缺少返回语句”，因此不存在可比较的 Jarde 运行轨迹；详见[`jarde-javac.txt`](jarde-javac.txt)。拒绝比猜测错误的 `finally` 更忠实，但普通命名多重捕获的正常返回仍是覆盖缺口。

## 字节码边界与架构判定

`choosePlain` 的两条命名异常表记录都保护半开区间 `[0,32)`，指向同一 BCI 33 处理器。BCI 30 的 `ldc "ok"` 受保护，BCI 32 的 `areturn` 不在保护范围。当前 [`region.rs::exception_edge_accounted`](../../../../crates/jarde-java/src/region.rs) 只允许块中位于行尾之后的 `Operation::Transfer`；规范块 BCI 30 同时含 BCI 30 和 32，因 `Return` 在 `[0,32)` 外而被认定为未结算的异常边。`leaving_edge` 因此报告 `jre_region_exception_edge`，尽管多重捕获分组和处理器正文已经正确构造。这是**已有半开范围与融合块结算规则的保守边界**，无需新增多重捕获 AST、异常图层或 pass。最小候选是仅允许同块、行尾后的无效果终止 `Return` 作为该 `try` 的正常出口；还须核对返回值在保护范围内的唯一来源、无额外后缀指令、命名行与处理器一致，以及生成表达式不把范围外的可能抛错计算挪入捕获范围。若任一证明缺失，维持引用；尤其不能普遍允许任意 post-range 指令或 catch-all。

`chooseFinally` 的异常表另有 `[0,33)→87`、`[43,77)→87` 两条 catch-all；正常副本为 BCI 33–38，命名捕获出口副本为 77–82，异常出口副本为 88–93。它们在复制相同 `finallyCalls++` 时还涉及嵌套保护范围与返回值快照，属于现有 [`recover-proved-finally-cleanup`](../../../changes/recover-proved-finally-cleanup/design.md) 中未完成的完成语义/分支与 typed-catch 扩展，不能由普通 `Return` 放行顺带解决。

本地 JADX `2fb1b163` 的 [`MarkFinallyVisitor.java`](/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/finaly/MarkFinallyVisitor.java) 第 277–351 行先以 `findCommonInsns` 比较各出口清理副本，要求每条规范清理指令的候选数等于 `handlerScopes.size() - 1`，通过后给重复候选加 `DONT_GENERATE`。这是可参考的候选发现/抑制流程，但我们的整类输出仅证明正常副本未被抑制；没有逐 pass 跟踪，不能断言是候选发现、数量门还是更早的区域选择失误。[`SameInstructionsStrategyImpl.java`](/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/finaly/SameInstructionsStrategyImpl.java) 第 31–35 行在常量赋值比较中以 `assignInsn.getArguments()` 和自身比较，是独立可疑的源码事实，也尚无证据说明与本案例有关。Jarde 不应照搬未经完整出口与异常范围验证的重复指令抑制；现有 finally change 要先证明副本等价和每条出口只清理一次。

## 拆分处理

普通 `choosePlain` 应有独立、窄范围 OpenSpec：在现有 `Catches`/`Region`/`Operation` 上证明保护区间边界处的正常返回，不改 `finally_copy` 或字段/短路规则。`chooseFinally` 保留为现有 finally change 的后续回归候选，先验证副本等价、所有出口只清理一次及异常覆盖优先级，再恢复结构。两个场景共用一个源探针，但实施合同互不混合。
