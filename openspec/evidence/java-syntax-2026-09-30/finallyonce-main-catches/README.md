# `FinallyOnce.main` 闭合：前置语句具名 catch 与拼接链归属

这是 CF-16 登记债务 [`full-original-main-debt.md`](../../java-syntax-2026-09-27/cf16-finally/full-original-main-debt.md) 的取证快照（2026-09-30，主线 `b3a01e24`）。它不改变恢复代码，也不构成 OpenSpec change；对应实施任务见 [recover-preceded-statement-catches](../../../changes/recover-preceded-statement-catches/)。

原始 class `FinallyOnce.class` SHA-256 `3ad6857285368c95c3176a520300c4abba09084443fe7fc08e8972330e9cedd2`（Java 11, major 55）的 `main([Ljava/lang/String;)V` 异常表只有一行 `[68,79) → 82 IllegalStateException`：一个普通具名 catch，无 finally。三段 `StringBuilder` 拼接链的 BCI 分别为 3–28、37–62（两段在入口直线块）与 86–111（在 handler 块内，`toString` 在 BCI 111）。

## 拆分与根因

按债务文档要求构造同布局最小场景（[fixture](fixture/)，SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)）：

| 场景 | 主线 `b3a01e24` 行为 |
| --- | --- |
| M1：两条顺序拼接链，无 try | 完整恢复 |
| M2：具名 catch 内一条拼接链（try 从块首开始） | 完整恢复 |
| M3：M1+M2 组合（= `main` 形态，Java 8） | `main` 整方法 explanation-only |
| M5：`helper(); try{…}catch(IllegalStateException){…}`，无任何拼接 | `main` 整方法 explanation-only |

两个独立根因，均已用最小局部修改在临时 worktree 复证（见下）：

1. **guard 资源证明吞掉了前置非 store 语句的具名 catch**（M5 即触发，与拼接无关）。`crates/jarde-java/src/guard.rs::resources` 的候选门槛处理了"try 前无指令"（跳过）与"try 前是普通赋值 store"（`copied_local` 失败跳过），但"try 前是**非 store 语句**（如 void 调用）"落入完整 TWR 证明，`initialisation` 在前置 BCI 处拒绝（`jre_guard_resource_init`），该拒绝 verdict 阻断普通 Catches 呈现，handler 与汇合块成为未覆盖块。M5 主线诊断：BCI 0 `jre_guard_resource_init` + `[22, 17]` 未覆盖。模块注释自身把问题表述为"try 之前是不是本资源自己的初始化"——非 store 语句对这个问题同样回答*否*。
2. **`concat.rs::verify` 的 split 检查误归属链尾**。检查用 `all`（SSA 块迭代序展平）中**首个**同 owner `toString` 当作每条候选链的尾；`main` 第三条链（头 86，handler 块内，自身 `toString` 111）被配到入口块的 BCI 28，报 `jre_concat_split`。同块内含更早 `toString` 的候选不受影响（头 3/37 均在含 28 的块内），因此只有"链所在块晚于含首个 `toString` 的块"的候选暴露误配。

两项各自单独都不足以恢复：只修 (1) 时第三条链仍被 (2) 拒绝；只修 (2) 时 `jre_guard_resource_init` 依旧阻断（本目录 [results/M3.exp.recover.json](results/M3.exp.recover.json)：`3 presented, 0 refused` 但 `jre_guard_resource_init`@65 与未覆盖块仍在）。

## 复证实验

在主线 `b3a01e24` 的临时 worktree 施加两个最小修改并全量验证（实验产物不进入主线）：

- concat split 候选加 `bci > head` 界（实验简化；生产实现必须按值流归属）；
- `resources` 候选循环在 handler-binding/field-assignment 门槛后增加"前置指令不是 `Store` 则该行不当资源头"。

结果（[results/fo.exp.java](results/fo.exp.java)）：

- M1/M2 输出不变；M3、M5 完整恢复，M5 与固定 JADX CLI 输出结构一致（`helper(); try/catch` 全保留）。
- **原始 `FinallyOnce.class` 全类零 `not recovered`**，`main` 与固定 JADX 源码结构一致；`handled`/`escaping` 既有输出不变。
- N1（store 前置具名 catch，`r = open(); try…catch(E)`）：两版本行为一致，保持 `jre_guard_handler` TWR 降级拒绝——模块注释明确的保守边界，本次不放宽。
- N2b（手写跨块 builder，真 split 链）：两版本均以 `jre_concat_split` 拒绝且指向真实消费点 BCI 32；生产实现必须保持该行为（含"方法内存在更早同 owner `toString`"时的归属正确性，M3 第三条链即为该反例：修复前误配 28，修复后应呈现而非拒绝）。
- `cargo test -p jarde-java --tests --locked` 于实验 worktree 23 组全绿：无既有正例/负例依赖被移除的行为。

## 边界

- N1 的 store 前置家族（`r = open(); …`）继续按资源降级拒绝，不在本切片内解决；这保持"证明不了的 TWR 永不伪装成用户 catch"的既有决策。
- 本目录证据为 JVM Java 8/11 classfile；不外推 DX/DEX 输入。
- `handled`/`escaping` 已由 [cf16-finally 各验收](../../java-syntax-2026-09-27/cf16-finally/) 独立闭合，本切片只针对 `main` 及其两个根因家族。
