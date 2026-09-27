## Context

固定 JADX checkout `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `TestTryCatchFinally12.TestCls.runTest(II)String` 有具名行 `[11,61)→64 IllegalArgumentException`。canonical BCI 0 block 包含 `[0,1,4,5,8,11,12]`，其中 BCI 8 是 `this.sb = new StringBuilder()` 的 `putfield`，BCI 12 是 switch dispatch；case 入口 40/48/56，唯一正常汇合 BCI 61，handler 64，后续 return 79。完整诊断见 `openspec/evidence/jadx-feature-inventory-2026-09-27/switch-inside-catch-region-debt.md`；可运行 fixture 和近邻放在 `openspec/evidence/java-syntax-2026-09-28/cf16-switch-catch/`。

当前 `guard::resources` 把覆盖 BCI 11 的 named row 当候选：BCI 8 非 local `Store` 导致 `copy=None`、`header=true`，`twr::initialisation` 以 `jre_guard_resource_init` 拒绝；`guard::catches` 收到 `Some(Refused)` 后不再解析 named row。`completed_field_assignment` 已可证明静态字段赋值的单语句/栈闭合前缀，尚不接受实例字段。临时只排除本例假候选后，`catches` 得到 handler64 与 `join_after(61)=79`，`try_level` 形成 `Try→Switch→Catch`。这时 body walk 返回 `body_next=61`，现有 `let (body, _)` 丢弃了它；switch 的 join 也停在 61，于是 BCI 61 成为唯一 uncovered block。临时用现有 `region_at(61)` 续走并交给 `split`，会产生紧随 Try 的 `Straight[61]`，无新 Region 类型；另一种把它放进 Try 正文的办法虽然可运行，却把保护区外块并入正文，边界更大。

## Goals / Non-Goals

**Goals:** 对上述固定字节码形状恢复 `runTest` 的外层具名 catch、完整 switch 和后续返回；同布局完整类的原/JADX/Jarde Java 8 重编、`java -Xverify:all` 路径行为一致，物理 BCI/异常行来源和唯一块 owner 闭合。

**Non-Goals:** 任意字段前缀和资源头的猜测、跨多个异常范围的 switch、任意 goto 链吸收、switch arm 局部 catch、其它 family 输出、Test13 的多段 finally。对于不满足本证书的输入保留已有安全拒绝。

## Decisions

1. **字段前缀先证明不是资源声明。** 复用 `completed_field_assignment` 的单语句、SSA 消费和块入口/出口栈闭合检查，允许完整实例 `putfield` 和既有 `putstatic` 在具名 row 前回答“不是 TWR 资源头”；前缀必须在当前 block 且位于异常范围外。只调整 `resources` 的候选门，绝不让 `catches` 全局忽略 `Some(Refused)`。局部 `Store` 来自 `new`/调用、Java 9 resource copy、nullable close 轮廓等真实 TWR 候选仍走原证明和拒绝路径。Builder 若无法陈述字段赋值仍由既有 checkpoint 原子回退。
2. **出口块只在完整闭合时续走。** `try_level` 已有 `body_next` 和 lexical boundary；仅当 `body_next` 是该具名异常行 exclusive end 所在 canonical block、块内恰有一条 `Operation::Transfer`、不在保护区/handler 内、普通后继唯一且正是 `join_after(end)`，并且所有普通前驱已由当前 Try 正文拥有、无竞争异常入口时，才用现有 `region_at` 在同一有界 frame 续走。保留现有 `split` 将纯跳转作为 Try 后 sibling 所有，后续 return 仍由正常 run 认领。证书不依赖具体 BCI 数字、字段名或 case 值。若某项不闭合，保留 uncovered/fallback 诊断，不吞任意尾块。
3. **来源和失败原子性随物理 owner 检查。** BCI 61 是由源码词法顺序表示的派生控制流，虽然不产生 Java `goto` 文本，仍需在来源图中出现一次；BCI 0 同块 lead 中的赋值只写一次，switch arm 40/48/56、handler64、后续79 各一个 owner。已有 ownership/fallback/checkpoint 规则继续约束预算或取消后的输出。对每个物理指令、异常行与 normal/exception edge 做定向检查，不以生成源码可编译代替语义证明。
4. **JADX 仅供固定行为参照。** 参考其异常 scope 与 switch region 拆分顺序，不复制仅凭指令相似就合并结构的假设。Jarde 的判定来自现有 classfile、canonical CFG、NormalFlow 与 SSA；Java 8 编译/JVM 运行只用于测试，不进入产品路径。无需新依赖或方言选择逻辑。

## Risks / Trade-offs

- [误把真实 TWR 当普通 catch] → 仅证明字段写入完成时跳过当前候选；所有 local-store 资源头与已有 TWR refusal 回归保持原样。
- [出口块有其它前驱、异常入口或后继] → 不续走；保留来源缺口而不编造 `try` 后顺序。
- [case 范围只覆盖部分 arm] → 异常表和完整 body owner 不闭合，拒绝把局部 catch 提升到整 switch。
- [整类其它方法失败混淆 runTest 结果] → 固定三方法和 runTest 分别报告；运行验收采用完整类和逐方法报告，不以一个方法的成功推断家族成功。
