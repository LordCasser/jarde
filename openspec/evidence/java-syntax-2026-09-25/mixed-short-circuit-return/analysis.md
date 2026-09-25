# 混合短路值作为 `return` 的独立 parity 边界

[冻结 Java 8 源码/class/Runner](../../../../tests/fixtures/p3-conditional-values/mixed-short-circuit-return/README.md)中的 `MixedLocalReturn.value(Z)Z` 返回 `(a && rhsB()) || rhsC()`。`javac --release 8 -g:none` 的 class SHA-256 为 `688ad8ed65e7da2cd3663b497ac6caa94e53885acae7656f468d457be5bd3288`。`Runner` 对 `a`、`bValue`、`cValue` 的八种组合逐一记录返回值及两次 RHS 调用次数；[原 class](original-run.txt)与 [JADX 1.5.6 完整类](jadx-run.txt)在 `java -Xverify:all` 下 8/8 行一致，JADX 类体用 `javac --release 8` 重编通过。JADX 自动加的伪 `package defpackage;` 仅在重编副本中去掉。

[反汇编](javap.txt)中 BCI 1 `ifeq 10`、7 `ifne 16`、13 `ifeq 20` 决定两个 `iconst_1/0` producer，BCI 21 是唯一 `ireturn`。这是与[混合字段写入](../mixed-short-circuit-field/analysis.md)相同的闭合无环测试图，区别只在消费点从 `putstatic Z` 改为方法描述符 `(Z)Z` 的 `ireturn`。[Jarde 基线](jarde-baseline-MixedLocalReturn.java)把 BCI 16 的重复 owner 原子引用，方法为 `quality=fallback`、`representation=mixed`、`content=explanation_only`，缺少返回语句，完整类 [javac 报错](jarde-baseline-javac.log)。混合字段图实现并独立验收后，又从当前 CLI 导出[整类与报告](jarde-after-field-graph-report.json)；该返回方法仍是同一个单一 owner 拒绝，说明是消费者边界而非尚未实现的混合测试图。

实施前的架构裁决：复用现有 `ShortCircuitValue` 的闭合图枚举、解码边、逐测试 SSA 依赖、两个 1/0 producer 与唯一 Phi 证明；把消费证明从唯一 `putstatic Z` 限定地扩到唯一 `ireturn` 且方法返回描述符为 `Z`，用已有 `Return`/`Conditional` AST 发射。不能只因 `ireturn` 与 producer 在同一区间就跳过外部前驱、异常边、第二消费者或惰性副作用证明。字段图先独立验收，再开返回值 OpenSpec，避免在字段实现里扩大范围。

该方向现已由独立的[返回值任务](../../../changes/recover-short-circuit-return-values/verification.md)实现并复验：[当前 Jarde 完整类](jarde-after-return-MixedLocalReturn.java)经 Java 8 重编，`java -Xverify:all` 的[八行](jarde-after-return-run.txt)与原 class/JADX 逐字一致，`value` 为 `structured/java` 且无引用。`MixedIntReturn.value(Z)I` 是同形 `ireturn` 近例，javac Java 8 class 与冻结字节相等；[当前 Jarde 证据](jarde-int-near-miss-report.json)仍完整引用全部十个 BCI，不把返回描述符 `I` 猜成布尔。上文“当前 Jarde 基线”描述的是本任务实施前时点。
