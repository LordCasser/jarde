# 2026-09-25 独立验收记录

冻结 `SharedTrueShortCircuit.assign(Z)V` 的原 class、JADX 1.5.6、当前 Jarde 完整类各自经 `javac --release 8` 重编，`java -Xverify:all` 两行一致：`left=true,result=true,calls=0`、`left=false,result=true,calls=1`。Jarde 当前源码仅有一次静态 `result` 赋值，`assign` 的全证据报告是 `quality=structured`、`representation=java`、`fallbacks=[]`，source map 覆盖 BCI 0/1/4/7/10/11/14/15。见 [三方对照与实现后 JSON](../../evidence/java-syntax-2026-09-25/short-circuit-shared-true/analysis.md)。

拒绝边界分成两个可验证控制：[双字段写入](../../../tests/fixtures/p3-conditional-values/short-circuit-shared-true-controls/README.md)来自 `javac --release 8`；非规范生产者从已有单写入 class 的 BCI 10 `iconst_1` 改为 `iconst_2`，StackMap 仍是 `int`。两者均经 `java -Xverify:all`；后者 JVM `Z` 最低位令左真结果从 true 变 false，恢复 proof 精确拒绝为 `Producer`。fresh recovery 测试核对方法 Fallback、所有必要 BCI 同时出现在引用和 source map、没有结构化字段赋值。旧 shared-false 的非 1/0、独立效果、第二写入与错误字段身份控制继续通过。

架构师独立运行 `cargo test -p jarde-java --lib` 为 161/161、`p3_conditional_values` 为 2/2、`class_source` 为 47/47、CLI `class_source_cli` 为 16/16；重新生成并重编 Base，原/Jarde 仍输出 `observed=captured-value`、`visibleDuringSuper=true`；左假控制原/Jarde仍为 `false-result=false,calls=0` 与 `true-result=true,calls=1`；非规范 `Z` JDK 控制 1/1。`cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-disjunctive-field-writes --strict --no-interactive` 均通过。严格 `jarde-java` Clippy 被 16 个既存 lint 阻断，详见 [条件字段 change 的门禁记录](../recover-conditional-field-writes/verification.md)。

两测试 shared-true 正例已通过；2.1 的真正误极性/额外入口负例仍需独立控制。三测试 `extra || left || rhs()` 的共享生产者和来源丢失是 [独立 change](../recover-short-circuit-field-chains/tasks.md)，不把本次两测试改动冒称为任意长短路链支持。
