## Context

见 [proposal.md](proposal.md) 与 [Region trace](../../evidence/java-syntax-2026-09-25/ternary-in-if/region-trace.md)。`bothMatch` 的 BCI 62 是共享 `iconst_1; ireturn`，普通前驱为 45→62 与 48→62；BCI 64 是 `iconst_0; ireturn`。六个条件分支都没有实际单一 post-dominator，`Region::If` 的单 join/互斥 arm 模型会把 62 认领两次。已有 `ShortCircuitValue` 可扫描有界测试图，但其两个 producer 汇入**一个** Phi consumer；本图有**两个**终结返回，没有 Phi 或共同 consumer。`overlapping_owner` 的整体拒绝正确，不能删除。

## Goals / Non-Goals

**Goals:** 只对完整、单入口、前向无环、双布尔终结返回的图在 Region 提交前证明一次所有权，复用现有条件表达式和 Return 发射，令可观察求值与字节码一致。

**Non-Goals:** 不建立通用布尔 CFG IR、不推断原源码的 `?:`/`&&`/`||` 写法，不放宽非终结 join、异常/循环区域或任意多出口，不更改既有 Phi 值消费者的准入。

## Decisions

1. **采用窄的私有双出口 Region 候选。** 在 `region_at_inner` 构造普通 `If` 双臂前，识别入口测试、按物理 BCI 有序的测试/真实 fallthrough 与 taken 边、纯 `goto` 转接、两个终结块。候选先验证所有正常前驱集合、唯一外部入口、每条边前进、无外部 owner、无第三出口；计费并轮询取消后才一次提交每个 `visited` 块。用 `Region::TwoExitReturn`（或等价的明确私有形态）承载该图，避免把零后继返回伪装成 `ShortCircuitValue` 的 Phi consumer，也避免在普通 `If` 中偷偷共享子树。保留 completed-tree 的 overlap validator。候选证明失败时普通路径仍可保守引用，但不得在半提交 visited 后继续。
2. **共享图边界思想，不复制值证明。** `ShortCircuitValue` 的有界发现、exact predecessors、测试边极性、gateway 与预算方式可以抽出小的私有 helper；其 1/0 push→Phi→单 consumer 的 SSA 证明不能复用为双返回证明。Builder 重新核对 Region 图与 canonical CFG/SSA、方法描述符 `Z`，两个终结块只能各含正确的 `iconst_1/0; ireturn`，且无多余读写或可观察操作。测试块中的 `getfield`、`equals` 等可观察求值必须由现有 `test_expr` 完整呈现并维持原惰性次序；独立语句或不可呈现的测试拒绝。
3. **从出口向入口逆向组装布尔表达式。** 两个叶子分别是带真实 BCI 来源的 `true`/`false`，纯转接继承目标表达式，测试严格按物理 fallthrough/taken 对应构造已有 `Conditional` 表达式，并设置展开上界；只在根表达式 Java 类型为 `boolean` 且每个测试来源完整时创建一个 `Return` 语句。单条语句的来源联合两条终结返回及全部图块，两个实体出口只发射一次。允许输出嵌套 `?:` 或等价布尔组合，不承诺原作者写法。若某个子表达式渲染失败，整图沿既有 fallback 输出。
4. **JADX 只作候选算法参照。** 本地 JADX 1.5.6 的 `IfRegionMaker.checkForTernaryInCondition` 比较两臂后续 `If` 的 dominance frontier，并仅在目标相同或互换时合并；本例实际输出是等价的嵌套 `if`/早退，并未还原 `?:`。这些判据提示先匹配路径与极性，但不足以替代 Jarde 的 exact physical predecessor、SSA、异常和一次所有权证明。JADX 仓库为 Apache-2.0；只参考思路，不复制代码或加入运行依赖。现有 Rust CFG/SSA、Region/AST 足够，不引入新库、crate 或全局 pass。

解析和 Java 8 方言检查由现有 reader/descriptor 提供，runtime selection 不参与此方法局部恢复；verifier-valid class 和 Java/JVM 执行仅用于离线测试，不在引擎内执行目标代码。图正确性仍由 Jarde 自己的 CFG/SSA 与物理来源证明，JADX 文本不能替代验证。

## Risks / Trade-offs

- **[共享终点被误当普通 join]** → 候选必须一次占有两个终结块；失败仍由 overlap validator 拒绝，绝不删重。
- **[反向极性或副作用次数]** → 保留 opcode 的 taken/fallthrough，按原图逆序建表达式；八条 null/非 null 路径与调用次数在原/JADX/Jarde 完整类上验证，并以可观察中间语句作拒绝控制。
- **[合法但更宽的图仍引用]** → 先限制两个精确布尔常量返回、单入口、无异常范围与无循环；后续形状另案，以免扩大本 change 的证明面。
- **[来源或预算半提交]** → Region 候选和 Builder 语句各自先证明后发布，逐节点计费，终结块与 gateway 均进入 source map；低预算/取消测试验证没有部分 Java。
