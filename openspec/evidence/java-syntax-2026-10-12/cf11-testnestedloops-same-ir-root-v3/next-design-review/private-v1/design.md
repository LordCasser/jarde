# 下一片 OpenSpec 设计建议：嵌套 Iterable loop 的 iterator 声明 owner

## 建议收敛的最小目标

先只覆盖已有 exact Iterator bytecode/SSA proof 的 inner loop：iterator 的空声明已经由祖先 lexical owner 发出，而 `iterator()` 初始化是当前外围 loop body 中的 `Assign`。准入后把该 inner while 投影为 `for (element : iterable)`，只从当前 owner 的 statement list 删除已证明的 iterator 初始化赋值。祖先的 `Iterator local5;` 允许暂时保留为无用声明。对这一片不做全树声明清理，不跨 frame 删除，不改 generic signature projection，也不把外层元素 `Assign` 绑定顺带投影。

## 复用现有所有权事实

这条路径可以复用现有 `LocalVariable` 和 `DeclarationPlacement`，无需新增可变的跨 frame 删除协议。candidate 已从 init Store 取得 slot；以 `reuse::Plan::variable_at(slot, init_store_bci)` 得到变量身份，并在 hasNext/next load 的位置验证仍是同一个 `LocalVariable`。以 `Declarations::placements` 确认该变量有已证明的词法 owner，拒绝 `Incomplete`；同时确认该 owner 是当前 loop 的祖先/当前可见 owner，且现有 planner 已在相应 owner 发出了声明。变量名仅用于 AST 拼写，不作为跨范围身份。

这片的删除意图只能指向当前被保存后又恢复的 `self.stmts`：删除包含 `iterator()` 调用的单条 `Assign`（若现有同-owner形状需要，也只删除同一当前向量里的声明项）。对本 fixture，空声明由祖先列表拥有且不在 inner candidate 的当前向量中，完全不索引、不修改、不搬移。保留空 iterator 声明是合法 Java 且语义不变；后续若要清理死声明，应由独立的稳定 AST owner 编辑设计处理。

对 fixture 中的次序，`local5 = l2.iterator()` 原在 outer body 中、紧邻 inner while 之前。转成 inner enhanced-for 后，`l2` 的表达式和 `iterator()` 在同一 outer iteration 的 inner-loop entry 求值，仍位于前序 `s1 = next()` 之后；不改变外层控制结构。现有完整 proof 仍必须检查 exact owner/descriptor、iterator init 单消费者、header local Phi 输入、hasNext 与 next 对同一 iterator、next/cast/store 的唯一使用，以及各关键 effect 的 handler 集合相等。当前诊断未执行这些后续 gates，设计不能把它们记作已验证。

## 外层元素 Assign 单独切片

外层入口现在因 body 首条 `s1 = (String) local3.next()` 而在 candidate 前置形状门拒绝。它不是“声明找错 owner”的同一问题：增强 for header 会隐式执行 next，若原 Assign 仍被保留就可能双取元素；若移除它，则要证明全部读取确实改由 header 变量满足，并确认 `LocalVariable` 的全部写、读和声明 owner/scope，无额外写、循环外使用或 split-slot 混淆。任何 cast/元素类型投影也要由该局部的实证决定。建议另开片，不以 synthetic name 改写绕过写读证明。

## 不混入的类型问题

当前报告指出 List 参数的 generic source projection 尚未被 Program/SSA 证明。foreach 结构修复不提供 `List<String>` 的 source type 证明；保持现有 generic refusal 原样，在后续结果中单独观察。不要参照 JADX 的 `fixIterableType` 把泛型参数写回 Jarde iterable AST。

## 可验收场景

- **WHEN** frozen `cf11/UpstreamNestedLoops.test` 的 inner candidate 通过完整现有 SSA/effect/handler proof，且 iterator declaration placement 是可见祖先 owner，**THEN** 仅 inner `while (local5.hasNext())` 变成 foreach；`local5 = l2.iterator()` 从 outer body 移除，祖先的 `Iterator local5;` 可保留；outer while 与 `s1 = (String) local3.next()` 保持原形，generic refusal 保持原形。
- **WHEN** iterator slot 在 init/test/next 间 split、声明 placement incomplete/不可见、存在额外写/读、phi 输入不匹配，或异常处理 effect 不一致，**THEN** 不做 foreach 投影，不编辑任何祖先语句向量；原有 while/拒绝证据保留。
- **WHEN** candidate 请求把 outer `s1 = cast(next())` 投影为外层 foreach，**THEN** 当前片拒绝/不变；该形状只由另一个 requirement 覆盖，并须证明元素写/读/作用域映射，不得额外调用 `next()`。
- **WHEN** full class default/all evidence 被请求，**THEN** 结构投影与 RuleDetails 的 site decision 一致；空 iterator 声明不作为增强 for 的额外 source write 证据，generic refusal 仍独立存在。

## 任务边界与后续验收

1. OpenSpec 明确复用现有 `LocalVariable`/`DeclarationPlacement` owner，不增加跨 frame mutable delete list。
2. 在 inner candidate 上增加 owner-aware lookup：当前 initializer statement 由当前父列表标识，祖先 declaration 通过 planner identity/owner验证；只对本地列表内可删除的 BCI 作索引。
3. 保持候选目前的 Iterator/hasNext/next、SSA phi/use、element cast、handler/effect checks；任何首拒移位都要由实际 same-IR 捕获和新负例确认。
4. 加一个以 frozen class 为输入的 Builder/report 测试：验证生成 AST/Text 的 loop 层级、inner init 删除、outer binding 原样、generic refusal；同时核对 Rules summary/gaps/default/all 一致。保留一个 declaration-owner不完整或 iterator local slot split 的拒绝 control。
5. 只在实际 builder/test 证明元素绑定重写的读写 owner/唯一性后，再单独提 outer `Assign` slice。

## 当前不能下的结论

当前 raw diagnostic 只证实了旧 candidate 的两个早期拒绝阶段与 1/0/0 测试摘要；没有证明 candidate 越过 declaration lookup 后的 SSA、handler/effect gates，也没有完整 `LocalVariable`/`DeclarationPlacement` 运行期值。本设计不是实现，未在当前 pipeline 运行；不把此前源码快照或诊断 acceptance 当作代码正确性证明。
