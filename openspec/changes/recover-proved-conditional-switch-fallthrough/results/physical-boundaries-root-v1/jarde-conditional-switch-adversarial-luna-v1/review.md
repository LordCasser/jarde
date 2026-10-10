# Conditional switch fallthrough adversarial review

范围：只读审阅 integrated `region.rs`, `build.rs`, `report.rs` 与 `integrated.patch`，并对照 `recover-proved-conditional-switch-fallthrough` 的 spec/design/tasks 和当前周边实现。没有运行 CLI、Git、Cargo 或 JDK，也没有改产品源码。

## 结论

目前没有静态证据表明 patch 会把 `SwitchBreak` 绑定到错误的最近 switch/loop，或会把未闭合的 case 图误收为有效 fallthrough。以下关键路径在代码上形成了相互约束：

- `region.rs::Frame::switch_arm`（约 2140–2175）在新 switch join 存在时覆盖 owner branch/join，子 frame 与普通 frame 派生会复制这些字段；`switch_region`（约 12860–12955）只将已证明的目标传给 arm frame。
- `switch_break_transfer` 与 `switch_break_branch`（约 13042–13165）核对精确 join identity/path 和完整 source 的 canonical Normal 出边；transfer 叶要求唯一 `Normal -> join`，条件分支叶要求恰好两个 Normal successor 并包含 join。直接跨到 join 的证据不会占有 join block。
- `build.rs` 两个 switch 渲染分支（约 17938–17972、18057–18080）在逐 arm 构建期间推入 branch/join/loop-depth/switch-depth target；`Region::SwitchBreak`（约 18430–18456）只接受栈顶目标 identity 相同且 loop/switch depth 未变化的叶。内层 loop 或 switch 中携带父 switch identity 的叶会拒绝；嵌套 switch 自己的叶由栈顶目标处理。finally checkpoint 保存并恢复两项新增状态（约 15551–15581）。
- `prove_switch_fallthroughs`（约 13468–13790）保留 canonical 平行边行，逐个 case 对完整 outgoing Normal multiset 与 view successor multiset 排序比较并拒绝重复项；对入口与 interior 检查完整 canonical incoming rows、边种类和 owner。有限 DFS 以灰色节点拒绝环；其他 case 和 join 作为边界；零出边只接受最后 SSA 指令对应的 Return/Throw operation。最多一个 case 目标，并由后续 BCI 排序的 `valid_order` 再要求目标紧邻。

## 尚需 fixture 才能确认的边界

这些不是已证实缺陷；源码审阅无法替代集成执行。尤其应按 design/tasks 中的实际 class fixture 验证：

1. **最近 breakable 作用域与深度恢复**：把候选父 switch 的 join-exit leaf 放进内层 loop，再放进内层 switch，并覆盖该内层 switch有/无 join 的形状；结果应明确拒绝跨越内层作用域的父叶。再用同一个 Builder 在嵌套结构之后处理一个正常 switch，确认 checkpoint/作用域恢复后栈顶 identity 和深度没有残留。相关消费点为 `build.rs::Builder::region` 的 `SwitchBreak` arm（约 18430）及两个 switch arm 构建循环。
2. **条件两路径的方向与混合出口**：一支到公共 join、另一支落入相邻 case；反转条件真假方向，并使 terminal transfer block 前带 side effect。应各自只在正确分支生成 break，副作用仍先于 break，下一 case 正文只出现一次。覆盖直接 `If` join edge 和经单 successor `Transfer` 到 join 两种 leaf（`region.rs::switch_break_branch` / `switch_break_transfer`）。
3. **完整边 multiset 与 incoming ownership**：对候选 interior/entry 分别加入 Exception、Call/Return、重复 Normal 行、clone path、外部 incoming，确认 `prove_switch_fallthroughs` 拒绝；另加只在 NormalFlowView 被投影隐藏的 canonical edge。函数入口的 iterator contract 要求每条 canonical edge 恰好传一次，真实 canonical 构造路径及测试扰动路径都需验证这一点。
4. **终点与环/排序拒绝**：无 outgoing 的普通指令、Return/Throw 后仍有 canonical outgoing、路径经 join 回到候选正文、路径先经过另一 case 再回流、两个不同 case 目标，以及 BCI 排序下非相邻 fallthrough 均应拒绝。尤其检查 branch terminal BCI 取自 `ssa.block(id).instructions().last()` 与 `operations` 的对应是否在实际 clone/path fixture 中仍是准确物理 terminal。
5. **完整恢复及来源**：原 CF12 class、partial-break、inner-loop-break、inner-switch-break 和 terminal-case fixture 需要实际运行；核 case2 只归属一次、join 不归 arm、switch-break BCI 有来源、没有假 Loop/arm overlap/uncovered。静态代码不能证明 emitter 完整输出、边界拒绝分类或 runtime 等价。

以上是本次静态审查结论；没有运行验收，不能据此声称 fixture、编译或运行结果通过。
