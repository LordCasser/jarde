# Java 8 extra-entry boundary for a short-circuit field chain

`ChainExtraBoundary.assign(ZZZZ)V` is compiled from the ordinary Java source `result = (gate ? extra : left) || other || rhs()`. The class file has major version 52 and was rebuilt byte-for-byte with `javac --release 8 -g:none -Xlint:-options`. `ChainExtraRunner` executes all 32 combinations of the four arguments and the configurable RHS result under `java -Xverify:all`; its exact output is in [runner-output.txt](runner-output.txt).

The relevant `javap -c -v -p` trace is in [javap.txt](javap.txt):

```text
 1 ifeq 11       gate selects the other ternary arm
 5 ifeq 15       extra=false enters the continuation
 8 goto 25       extra=true enters the shared true producer
12 ifne 25       left=true enters the shared true producer
16 ifne 25       other=true enters the shared true producer
22 ifeq 29       rhs=false enters the false producer
25 iconst_1      shared true producer
26 goto 30
29 iconst_0      false producer
30 putstatic result:Z
33 return
```

The BCI 8 `goto` is a genuine additional normal predecessor of BCI 25 from the ternary arm. It is not a fourth test in a linear OR chain. The local three-test candidate beginning at BCI 12 has short-circuit exits to BCI 25, but BCI 25 also receives the BCI 8 edge. The consumer at BCI 30 has one `putstatic Z`.

Fresh Jarde `class-source --evidence all` produced [jarde-report.json](jarde-report.json) and [jarde-ChainExtraBoundary.java](jarde-ChainExtraBoundary.java). For `assign`, it reports `quality=fallback`, `representation=mixed`, and four `jre_region_loop` fallbacks. Its body has no structured `ChainExtraBoundary.result =` statement, but it repeatedly quotes BCI 25 as a re-entered block and quotes BCI 30 for an unavailable stack value. The text's `@bytecode` lines mention only BCI 15, 25, and 30. The full source map includes BCI 0, 1, 4, 5, 11, 12, 15, 16, 19, 22, 25, 30, and 33; it omits the decoded instruction starts BCI 8, 26, and 29. The field detail says BCI 30 `presented=true` although the emitted body contains no field assignment. Thus the conservative refusal avoids a false structured assignment, but does not meet task 2.1's complete quote/source-map condition.

架构师又独立重编三份完整类并使用同一个 32 组 Runner 执行。原源码重编 class 与冻结 class 字节相同；[JADX 1.5.6 完整类](jadx-ChainExtraBoundary.java)和 Jarde 完整类均可 `javac --release 8` 编译、`java -Xverify:all` 执行。JADX 输出 `if (!z ? !z3 : !z2) { z5 = true; } else if (z4 || rhs()) ...`，与原 class 有 **16/32** 行不同；Jarde fallback 文本有 **30/32** 行不同，不能以可编译性代替语义证明。最简单的 mask 0（五个布尔输入均假）原 class 给 `0:false:1`，JADX 给 `0:true:0`，Jarde 给 `0:false:0`；mask 3（gate 与 extra 真）原 class 给 `3:true:0`，JADX 给 `3:false:1`。JADX 对条件合并与极性反转的输出在本例错误；具体是 `IfRegionMaker` 的哪一步导致，还需追踪 pass 中间态，不能仅凭最终文本断言根因。重编副本只删除了 JADX 为无包名 class 写的伪 `package defpackage;`；类体未改。编译与运行日志见 `jadx-*`、`jarde-*` 文件。

按 Runner 位掩码归类，JADX 的 16 条错误恰好是 `other=false` 的全部路径（mask 0–7、16–23）；`other=true` 的 16 条路径碰巧值与调用次数均正确。错误不只是字段真值翻转：例如 mask 16 的最终结果仍为 true，但原 class 调用 `rhs()` 一次，JADX 调用零次。这进一步要求验收同时检查短路 RHS 次数，而不是只比较返回/字段值。

The pending [preserve-region-fallback-instruction-origins](../../../changes/preserve-region-fallback-instruction-origins/design.md) change identifies that `Builder::covered_bcis` currently uses canonical *block starts* as though they were instruction starts. Its planned per-instruction fallback origin repair should address the missing BCI 8, 26, and 29 where the refused regions own their containing blocks. That proposal explicitly leaves Region selection unchanged. It does not by itself give the unclosed candidate one atomic refusal owner, remove repeated BCI 25 ownership/false loop diagnostics, or correct the field `presented=true` detail. Those are separate boundaries of this short-circuit change; no production fix is included in this evidence.

Reproduction from the repository root:

```sh
E=openspec/evidence/java-syntax-2026-09-25/short-circuit-chain-extra-entry
javac --release 8 -g:none -Xlint:-options -d /tmp/jarde-chain-extra-rebuild "$E/ChainExtraBoundary.java" "$E/ChainExtraRunner.java"
cmp "$E/ChainExtraBoundary.class" /tmp/jarde-chain-extra-rebuild/ChainExtraBoundary.class
java -Xverify:all -cp "$E" ChainExtraRunner
CARGO_TARGET_DIR=/tmp/jarde-chain-extra-source-target cargo run -q -p jarde-cli -- class-source --input "$E/ChainExtraBoundary.class" --policy single-class --class ChainExtraBoundary --evidence all --format json
```

The frozen class SHA-256 is `2630a7ea3e395b74121053dfa9370288dc510ea4d1fbfcf3bd9fbb9d04e1ba55`.
# 所有权与来源联合修复后的独立复验

[新完整类](jarde-after-owner-ChainExtraBoundary.java)及[全证据](jarde-after-owner-report.json)现在只给 `assign` 一个 `jre_region_ownership_overlap` whole-body fallback，正文标出全部 16 个已解码 BCI，包含旧漏项 8/26/29，也不再虚构四个 `jre_region_loop` 或结构化字段赋值。该完整类以 `javac --release 8` 重编通过，`java -Xverify:all` 的 [32 条输出](jarde-after-owner-run.txt)全部不同于原 class：引用文本不是语义恢复，不能用可编译掩盖。字段 rule-detail 的 `presented` 仍需单独核对最终发射。所有权任务的定向及相邻回归见 [独立验收](../../../changes/refuse-overlapping-region-ownership/verification.md)。

本地 JADX checkout `2fb1b163` 的源码复核把错误定位到候选范围，而非定案某一行：`jadx-core/.../maker/IfRegionMaker.java:479–508` 的 `checkForTernaryInCondition` 先比较两 arm 的 dominance frontier，再按 then/else 相同或交叉的关系构造 `IfCondition.ternary`；交叉时会反转 `nextElse`。普通短路合并在同文件 528–544 行，`getNextIf` 的前驱检查在 598–624 行。后续 `IfRegionVisitor.java:119–125` 还可能为排 `else if` 反转整个 region。已知输出把短路首项的补集放在写 true 的分支，但没有保存这些 pass 的中间态，故不能断言首次错极性发生在哪次反转。Jarde 的设计不能只依赖两 arm 的相同汇合点或地址顺序：BCI 8 进入共享真生产者的额外前驱必须由物理边和 Phi 消费证明解释，否则整体拒绝。

正向恢复前的架构边界已定位：当时私有 `Region::short_circuit_value` 只允许双后继测试节点与“首指令为常量 Push”的单后继 producer，BCI 8 的无副作用 `goto 25` 不属于两者，所以在区域认领阶段便拒绝；即使把它误当第三个 producer，`prove_short_circuit_value` 对测试边和生产者精确前驱的检查也会失败。已在**现有有界无环测试图内部**增加仅负责转接的私有 gateway：先证明它只有一个正常入口/出口、没有独立效果或可抛异常，再沿真实目标将逻辑边归并，同时保留 BCI 8 的物理 owner 与来源；所有生产者的物理前驱仍与测试边加已证明转接边精确相等。值树从解码的 taken/fallthrough 与 producer 的真实 1/0 推导，并在唯一 `putstatic Z` 的 Phi 上复核，未搬 JADX 的 `IfCondition` 极性。实现和 32 路径独立验收见[验证记录](../../../changes/recover-short-circuit-transfer-gateways/verification.md)；没有新建公开布尔 IR 或全局优化 pass。
