## Context

[固定证据](../../evidence/java-syntax-2026-09-30/finallyonce-main-catches/README.md)把失败收敛为两个可独立复现的根因。`guard.rs::resources` 的注释把候选门槛表述为一个问题："try 之前是不是本资源自己的初始化？"——已实现的*否*答案有四个（无前置指令、handler 绑定 store、具名行前字段赋值、普通赋值 store），唯独漏掉"前置是非 store 语句"（void 调用、`astore` 以外的语句结尾）；此时行进入完整 TWR 证明，`initialisation` 以 `jre_guard_resource_init` 拒绝，该 verdict 阻断 Catches 呈现，handler 与汇合块成为未覆盖块，整方法回退。`concat.rs::verify` 的 split 检查在 walk 之前运行，用 `all`（SSA 块迭代序展平）首个同 owner `toString` 命名"链尾"；其存在理由是给"分支切断链"一个准确诊断，而 walk 本身才是准入门。

## Goals / Non-Goals

**Goals:** 补齐 resources 门槛族缺失的分支，使非 store 前置的具名 catch 交给 `catches`；使 `jre_concat_split` 的归属来自候选链自己的值流。二者共同闭合原始 `FinallyOnce.class` 完整类。复用现有 Catches/concat walk/来源与预算；不新增实体。

**Non-Goals:** store 前置家族（`r = open(); try…catch(E)`）的 TWR 降级拒绝不放宽；不新增泛用值流框架、不改 Region/AST；DEX 输入与任意"语句种类枚举"不在本次范围。

## Decisions

1. **门槛补分支，不改保守语义。** 在 handler-binding 与具名行字段赋值门槛之后、null-resource 检查之前，增加"`before` 不是 `Operation::Store` 则 `continue`"。真实 TWR 的资源 store 恒为紧邻前置（javac 对 `try (R r = open())`、Java 9 `try (r)` 的 `aload/astore` 拷贝均如此），故该分支不可能跳过任何可证明的 TWR；被跳过的行回到 `catches`，其具名类型由异常表自身陈述。与"无前置指令"分支同构，注释按同一问题表述。
2. **split 归属走链自身值流。** 从候选的 allocation/ dup 值起，对每个同 owner、`bci > head` 的 `toString`，沿其接收者 `ssa.value(..).def()` 经 append 返回值有界回溯（深度上限 16，超限视为不可达）：回到本链 allocation 即认定消费点。消费点在他块 → 维持 `jre_concat_split` 并命名该 BCI；不存在跨块消费点 → 不以此码拒绝，由 walk 的"块内无 `toString`"等自身理由拒绝。检查仍是纯诊断归属，walk 仍是准入权威；不改变 `plan_four_conditional_strings` 的独立证书。
3. **验收锚定原始 class 与家族正负例。** 原始 `FinallyOnce.class`（Java 11）全类 class-source 零 not-recovered、`main` 与固定 JADX 结构一致；M1/M2 既有输出逐字不变；M5 家族（含参数化变体）恢复且行为与原 class/固定 JADX 三方一致；N1 保持 `jre_guard_handler` 拒绝、N2b 保持 `jre_concat_split` 且指向真实消费点。全部经 `javac --release 8` 重编、`java -Xverify:all` 路径运行、来源覆盖与预算/取消回滚验证。

## Risks / Trade-offs

- **跳过分支误放行坏 TWR** → 真实 TWR 的前置恒为 store（javac 降低不变式）；以"store 前置 TWR 正例仍恢复、降级样例仍拒绝"回归覆盖。
- **值流回溯把别人的 `toString` 认成本链消费点** → 回溯只经 append 返回值，且终点必须是本链 allocation/dup 的 def；N2b 与 M3 第三条链（更早同 owner `toString` 存在）双向钉死归属。
- **诊断码语义漂移影响调用方** → `jre_concat_split` 的码与出现条件（存在跨块消费点）保持，仅归属 BCI 变准确；测试断言同步更新并有旧输出对照。
