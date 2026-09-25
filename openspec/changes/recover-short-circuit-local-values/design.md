## Context

基线与边界见 proposal 及 [冻结证据](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-local/analysis.md)。当前短路值 Region 与 SSA 证明已支持字段、直接返回和单参数静态调用消费者，但未将 local `istore` 识别为 Phi consumer。普通局部 `Store` 已有声明/赋值写入、变量复用、稳定命名和 `ExprKind::Local` 读取路径。

重要限制：冻结样本使用 `-g:none`，而且当前局部声明计划不会从该 Phi 得出 `Type::Boolean`。`decide_types` 的 `boolean_proof` 对 `Definition::Phi` 返回 `BooleanEvidence::None`；随后该写入经普通 `value_type(Value::Int)` 落为 `Type::Int`。`LocalDebugName` 也只保留 slot、范围和名字，不保留 LVT descriptor。因此本 change 必须在类型/声明决策中增加一条窄的前置证明，不能假定现有计划已给出 Boolean，也不能仅凭 `istore` 或 `1/0` 常量把 int-like local 猜成 boolean。

## Goals / Non-Goals

**Goals:** 将栈 Phi 的一次已证明 Store 与既有局部声明/赋值 AST 接合；只对布尔类型、名字和词法范围均可证明的局部开放；保留 Store 时点的惰性求值和后续 local reads。

**Non-Goals:** 不新增公开 AST/IR/pass；不扩展到数组 `bastore`、实例 `putfield`、多 Phi 消费者、任意 int-like local、跨 Region 作用域或不闭合控制流；不把 invocation argument 的调用目标绑定并入本 change。

## Decisions

1. **沿现有图只扩一个消费者形态。** 沿用测试分支、真实边、`1/0` producer、栈 Phi 唯一 use 的证明。新 Store anchor 必须是 `istore` 形态，精确读该栈 Phi 一次，并写到 SSA 与 slot reuse 同时指认的同一 local；`astore`/`lstore`/`fstore`/`dstore` 不属于候选。
2. **在现有类型/声明决策前置证明 Boolean local。** JVM 验证类型把 boolean 与其它 int-like 值放在同一类，单凭 `istore` 不足以推出 Java 类型。为这个消费者候选，先证明：短路 Phi 的分支 producer 精确为 `1/0`；Phi 唯一直接 use 是指定 `istore`；该 Store 对应唯一 SSA local/slot 定义；该局部的全部可达定义-使用链明确包含其后所有 load；每个 load 都能沿 SSA use 到显式 Boolean 消费上下文（冻结样本为 BCI22→BCI23 `putstatic result:Z`、BCI26→BCI27 `(Z)Z` 的 `ireturn`）；不存在未覆盖的读、写、别名或作用域外使用。仅在这些前置证据齐全时，向既有 `decide_types`/declaration placement 输入该局部的 Boolean 证据，使其输出 `Type::Boolean` 并按原 placement 规则选声明或赋值。任何一环不明都不得提升类型，整体拒绝候选。不要新增通用类型推断 pass。
3. **只在 Store 发射条件值。** 把条件表达式适配为已决定的 Java boolean 后交给现有 `Declare`/`Assign` 写入；Store BCI 是该语句的直接来源，测试与 producer BCIs 按既有短路值 origin 作为派生来源。成功提交后抑制原测试/producer 的重复语句输出；之后的 `iload` 只读已声明变量名。
4. **允许 local 值多次读取，限制 Phi 直接使用。** Phi use list 仍必须只有该 Store 一项。Store 产生的 local SSA value 可有多个读者，但每个读者必须能在该声明的词法范围内写同一个名字；这些读取不复制条件表达式，也不增加 RHS 调用次数。把 Phi 直接消费数与 Store 后 local 值的 use 数分开核对。
5. **拒绝保持原子。** 第二 Phi use、local 类型/变量身份不明确、use 越过声明范围、额外边/effect、ownership overlap 与预算/取消均保留当前整体 fallback；不能先折叠分支再把不兼容的 Store/后缀遗留为半结构结果。

## Risks / Trade-offs

- `[风险] int-like verifier slot 被误判为 boolean` → 不复用当前 plan 的默认 Int 作为已证明 Boolean；先按精确 Phi、store 与全部 local use 的闭合证明向现有类型决策提供 Boolean evidence，拒绝任一其它/未解析读者。
- `[风险] 条件值被 Store 与后续读取重复发射` → 只在 Store 写一次，检查后续 AST 仅使用 local 名，并用两处实际 `iload` 的受控轨迹验证 RHS 次数。
- `[风险] 多个写入点或不同 lexical owner 使 local 声明失效` → 首个版本只支持单个闭合候选 Store；其它路径、跨作用域读者和 Region 重叠作为拒绝控制。
- `[风险] invocation argument 已在独立 change 实施，消费者证明发生交叉]` → 共享其图事实但保留独立的 Store 变体、布尔类型与 declaration-scope 验收，不修改调用消费者边界。
