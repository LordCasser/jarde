## Context

见 [proposal](proposal.md) 和 [冻结的三方对照](../../evidence/java-syntax-2026-09-24/foreach-cache-declarations/analysis.md)。当前 `reuse::plan` 在没有 LVT、或一个槽只有一个 LVT 名称时，总是回答 `LocalVariable::whole(slot)`；该名称即使只覆盖循环后，也被套到循环前的合成别名。`declarations::decide_types` 随后由此变量的第一写入决定 `int[]`，后继的整数赋值却仍用同一名字。`ReuseAfterForEach` 的 `-g`/`-g:none` 都重现此错误；它不来自增强 `for` 缓存声明，因此不在相邻清理 change 中修。

JADX 1.5.6 的 `InitCodeVariables` 先为 SSA 定义分配 `CodeVar`，将 phi 连通分量合并；`DebugInfoApplyVisitor` 通过类型更新门再贴 LVT 名称，`ProcessVariables` 最后以该身份的 assign/use 位置定声明。这种“值身份先于名字”的顺序值得借鉴；其整体 Dex 类型推断与可变 IR 不适合照搬。其 `ProcessVariables` 仍有按调试名称归并时需核对作用域的 TODO，并可能在找不到声明位置时回退到方法开头。Jarde 因而沿用自己的 SSA 值、phi、帧类型、canonical CFG、Region、`reuse::Plan` 和 `NameTable`，以有界 CFG 可达性及写入类型门给出更明确的证明/拒绝，不建立第二套全局局部变量模型。

## Goals / Non-Goals

**Goals:** 先阻止一个已恢复 Java 局部中确定的引用/原始类型冲突；再为 `ReuseAfterForEach` 可证明的先循环、后整数两个生命周期分别赋 `LocalVariable`，有无 LVT 都重编运行；保留已有多 LVT 名称分段、参数/receiver 与资源/`catch` 头部规则。其他类型可赋值性需要单独证据，不能把本门称为通用 Java 类型检查器。

**Non-Goals:** 不从任意字节码反推唯一原源码词法作用域；不做通用 SSA 到源码变量重建、不折叠后继局部、不改变数组增强 `for` 的准入或清理缓存声明；不把类级泛型与外部类型解析带入此项。含非平凡跨段 phi、从后段回到前段的循环回边、handler 或不可归属访问的更复杂形状优先如实拒绝。

## Decisions

1. **先守住确定的类型冲突，再扩大分段正例。** `declarations` 已集中决定每个 `LocalVariable` 的声明类型，但只取第一写入。核对同一身份所有会发射到源码的写入；现阶段引用与原始类型两个类别明确冲突时，若 `reuse` 未给出合法分段，拒绝该恢复区域并报告冲突写入 BCI，而不是发布自相矛盾的 `Recovered` 正文。引用间的子类型关系、原始类型间的转换另行处理，不从不完整信息假装得出“不兼容”。此门本身不创造新局部，也给后续分段实现一个可验证的安全基线。不能单靠 Java 编译器作运行时产品判据。
2. **在现有 `reuse::Plan` 中增加有界的值生命周期证明。** 对每个非参数/非 guard-header 的槽，核对 SSA 写入、读取和 phi 代表值；非平凡 phi 连接两段或访问无法唯一归属时不分段。首片只接受 `Ref` 与 `Int` 两类、各自完整的写入/读取值链、访问 BCI 不重叠，且 canonical CFG 的正常边证明后段无法返回前段访问块。前段自身可有平凡自 phi 和回边；互斥分支在各自访问完整、无后段返前段路径时也可形成两个源码局部，但不声称这两个局部在同一次执行中先后运行。异常/call-context 边存在时本推断直接退出。`-g` 中只有后段 LVT 记录时，记录仅命名与其读取链匹配的后段；`-g:none` 则以确定性槽/段序号命名。此方法复用已准备的 SSA/CFG/Region 和预算，不另建 pass 或跨 crate 解析器。
3. **名称证据可以逐段缺失。** `SlotEvidence::Split(Vec<String>)` 目前强制每段都有原始名；新形状需表达“第一段无 LVT、第二段有名”，且 `NameTable` 必须把未命名段计入 invented、对所有段执行同一去重和关键字规则。可以把 split 元素改为 `Option<String>`；项目不承诺内部类型后向兼容，避免用假造的原名欺骗 provenance。`variable_at(slot,bci)` 仍是 builder 唯一取身份的入口；其分段映射须覆盖本次会写进 AST 的每个本地读取/写入及表达式消费锚点，缺失即拒绝相关区域，不猜。
4. **分段后继续沿用声明规划。** `slot_uses`、`decide_types`、`Declarations::at_region` 已按 `LocalVariable` 键工作，故它们可独立决定数组别名与后继整数的类型和词法 owner。新计划应先形成，随后 NameTable 与声明计划按既有顺序运行。成功时不需新 AST 节点；增强 `for` 仍由既有证明投影，source map 保持原本 BCI。预算/取消在发布前完成，essential/all 共享同一文本。

## Risks / Trade-offs

- **仅按字节码线性顺序分段会错过回边或异常流** → 以 SSA 值连通和 canonical CFG 正常边可达性共同证明；后段可返回前段或出现 handler/call-context 边时不走新推断。互斥分支可分别成局部，前段自身的平凡自 phi 不阻止先循环、后直线局部。
- **帧给出宽泛引用类型而新数组操作提供更具体类型** → 使用现有 `written_type`/数组元素来源，不由 LVT 名称猜类型；不可写类型不强行分段。
- **只有后段有 LVT 名称，前段别名会错误继承它** → 逐段可选原名，按使用链贴名并保持 source-map BCI。
- **同槽相同类型写入被过度拆分** → 分段准入要求类型不兼容且无重叠；普通连续赋值保留一个变量，既有多 LVT 明确分段仍按其规则。
- **跨分支、phi、`catch`/资源与 category-2 相邻槽** → 参数与 header 声明不分段；互斥分支只有满足同一完整值链与不可返回证明才接受，重复回边、非平凡 phi 和异常边不走新推断。宽槽不误认；无完整证明只保留未恢复标记，不把类型矛盾的正文当作恢复结果。
- **预算/取消导致身份与名称半更新** → 在 `reuse::Plan` 构造中先对全部待访问点计费/轮询，完整计划建成后再交给 NameTable/Builder；停止沿现有报告路径发布前缀。
