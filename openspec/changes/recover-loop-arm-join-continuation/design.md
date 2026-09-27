## Context

固定 Java 8 `LoopIfJoin.run(ZZ)I` 的 BCI 0 外层分支后支配点是 34，BCI 6 内层分支后支配点是 26。真臂 BCI 10–12 到 26；假臂 BCI 15 循环的更新 BCI 20 回边到 15，唯一正常退出也到 26；BCI 26 加 10 后到 34。Jarde 的原有 `continue_inner_join_arm` 只从两个 `Region::Straight` 臂提取各自最后一个 block，并要求 BCI 26 有这两个准确入边、尾部是一段直线。该函数因此拒绝循环臂，即使 Region 构造和 CFG 已可能具备闭合事实。局部 slot 2 跨引用区诊断发生在 Region 失败之后。

## Goals / Non-Goals

**Goals:** 对一个已结构化、无额外 break/continue/return 的单出口 `Region::Loop` 臂和一个直线臂，证明两者汇于内层 join；同次走访把 join 到外层 arm boundary 的唯一、无外部入口直线尾部归入外层 arm；保持各 BCI 一次所有权与条件极性。

**Non-Goals:** `NotIndexedLoop` 的带效果双出口，循环内额外 break/return、异常/子程序转移、多个循环臂、任意 region 终点抽象、局部 SSA/声明放宽或将 `while` 强行改写成 `for`。

## Decisions

1. **从实际 Region 与 CFG 证明出口。** 在 `continue_inner_join_arm` 已有一个 `Region::If`、局部 join 和父 frame boundary 的前提下，只给单个 `Region::Loop` 臂增加私有候选：其 `exit` 必须是内层 join，唯一正常出边的源块必须由该循环持有，不得有 loop body 的另一条外出或异常/子程序入口；另一臂沿现有 Straight 末块。若 loop region 事实上未完整恢复、`exit` 不一致或存在任何未归属块，保持原有拒绝。
2. **闭合 join 和直线尾部后再认领。** 继续要求 join 的 canonical 入边恰为两个候选出口、没有额外正常/异常入口；现有尾部走访、`continuation_claims_are_exact`、同路径、逐块单出边及外层 boundary 检查继续适用。只在全部证明完成后把续接 Region 追加到当前 arm；失败时撤销本次 visited 变化并沿原子 fallback 路径返回。
3. **来源和行为优先于源码形状。** BCI 0/6/10/12/15/20/23/26/29/32/34/35 的物理动作、判断、更新、跳转与返回必须在方法来源中可追踪；`value += 10` 恰执行一次，三个输入在原/JADX/Jarde Java 8 类中均输出 `4 / 13 / 11`。同 join 的简单嵌套循环、已有 if/loop 回归不变。带额外 loop exit、第二 join 入边、异常边、预算或取消仍保守拒绝或中止。

## Verification

使用 [固定输入及基线](../../evidence/java-syntax-2026-09-27/cf08-endless-loops/join-baseline/report.md)构建原 class/JADX/Jarde 全源码，核对 class SHA、JADX commit、源码 BCI owner 和 `java -Xverify:all`。增加单元与集成正反例，并重放 CF-08 双出口纯网关、CF-07/09 循环出口及 `NotIndexedLoop` 现存负边界。运行适用 crate 测试、fmt、check、OpenSpec strict；清理独立 Cargo target。
