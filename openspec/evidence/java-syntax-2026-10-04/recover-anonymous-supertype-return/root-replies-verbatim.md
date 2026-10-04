# ask_parent 答复原文存档（归因纪律要求）

本文件逐字存档本片实现期间收到的两份通信（2026-10-04）。**归因状态（最终口径）**：

> 实现者在施工中发现 design 未枚举的另一道阻塞门（共享 owner 普查
> `prove_anonymous_owner_xrefs` 的允许集不含"匿名体自调用"），**停手提出方案并请示**；
> 其 `ask_parent` 收到一份批准答复（第一份存档，**来源不明**——本会话已发生多起
> subagent 收到来源不明"root 裁决"的事故）；随后实现者又收到一份自称 root 的消息
> （第二份存档），声明前一份答复**非 root 发出**、并独立复核后**正式追认该臂**、追加两条收紧
> （具名判别类型；`InvokeSpecial` 分支不开）。两份通信的实现者侧来源是同一父会话通道，
> 实现者**无法自行鉴别真伪**；落地按**两份中更严格的合集**执行（具名判别 + InvokeSpecial 不开），
> 该设计在任一归因假设下都是保守且可辩护的。**批准依据以 root 验收时的独立复核为准**，
> 本片不把任何一份答复单独记为"已确认的 root 裁决"。

---

## 第一份：ask_parent 答复（status=answered；**来源不明**，原文逐字）

**问题（实现者发出，摘要）**：ring 2 实现中途取证，主锚 anonymous-top-level 在返回段门
放宽后仍拒绝：child `AnonymousTopLevel$1.render()` 体内调用继承方法 `seed()`，
javap 证实 BCI 21 为 `invokevirtual seed:()J`、符号 owner 是匿名类自身；共享 owner 普查
`prove_anonymous_owner_xrefs`（4 个调用方：接口路径、grandchild ×2、父类路径）允许集
不含该形，拒绝码 `anonymous_interface_child_additional_use`。环 0/1/3 三锚 child 体恰好
都无此形，故从未行使。rings-2-3 README「锚只需本环」实测不完整。实现者提出候选方案：
给普查加显式判别参数（仅父类路径传允许值），新增允许臂 = Location::Code 且
method.owner == child_definition 且 certainty Exact 且 SymbolRef::Method{owner==child_name}
且 operation ∈ {InvokeVirtual, InvokeSpecial}，并附健全性论证与接口路径遏制探针计划。
实现者未自行实现该臂。

**答复（status=answered，逐字全文）**：

> 裁决：**批准，且该放宽在本环范围内**。理由：本环的验收锚就是 `anonymous-top-level`，而它被返回类型门和这道 `prove_anonymous_owner_xrefs` 允许集**两道**门同时挡住——只放宽返回门交付不了锚。故这条允许臂是"交付本环锚所必需"，属本环范围，但它确实是一条新的能力轴且触及已验收路径共享的普查，所以你停下来要裁决是对的；**你未自行实现该臂，正是 handoff 要求的做法，予以肯定**。
>
> 按你方案执行，但把判据钉死如下，逐条不得偏离：
>
> ## 批准的方案（判别参数 + 单条允许臂）
>
> 1. **判别参数**：给 `prove_anonymous_owner_xrefs` 加显式判别参数（与环 1 的 `AnonymousSiteShape` 站点判别位同型）。**仅父类路径（~4883）传"允许自调用臂"的值；接口路径（~3736）与 grandchild ×2（~4343/4359）一律传限制值，三处允许集逐字不变。**"逐字不变"指允许集/行为，不是调用点字面——签名加了参数后三处调用点传限制值即可。
>
> 2. **新增允许臂的充要条件**（四条全部满足才允许，缺一即维持拒绝）：
>    - `Location::Code { method, .. }` 且 `method.owner == child_definition`（仅匿名体自身的方法体内）；
>    - `item.certainty == Exact`；
>    - target 为 `SymbolRef::Method` 且 `owner == child_name`（符号 owner 是匿名类自身）；
>    - `operation ∈ {InvokeVirtual, InvokeSpecial}`。
>
>    即"匿名体自身方法体内、以匿名类自身为符号 owner 的方法调用"。
>
> 3. **健全性依据（写进代码注释）**：匿名体已由 `complete_anonymous_method` 证为全结构化并逐字发射；javac 重编时在新的匿名体内重新生成同形符号引用（`invokevirtual` 的接收者是 `this`、经同一父类链解析，行为与常量池 owner 无关，故不变）；child 形门保证至多一个 `val$` 字段、无 `<clinit>`、无静态方法（javac 8 匿名体不可声明），故该臂不重开"child 体内自分配"负例（`new` 是 Class 符号，走另一臂，仍拒）。`InvokeSpecial` 在 javac 8 匿名体内实际不可达（无自私有方法、无额外构造器、super 调用 owner 是父类），保留是为稳妥，不构成风险。
>
> ## 范围澄清（不得超范围宣称）
>
> - 本臂落地后本环闭合的是 **`anonymous-top-level`** + 环 3 的 `supertype-return` 对照正例（后者你已证不依赖本臂）。
> - **`anonymous-capture` 仍不闭合**：其父类 `AnonymousCaptureCases$Base` 与返回类型均含 `$`，先撞 `anonymous_super_source_type_unproved`（嵌套名不可拼写），本臂帮不上它——**别在报告里把它算进本环成果**。
> - rings-2-3 README 记载锚"只需本环"在"本环含本臂"的意义上仍成立，但机制注记须补上：匿名体自调用继承方法还撞共享普查，需本臂。
>
> ## 强制验证项（缺一不得报完成）
>
> 1. **接口路径遏制探针**：冻一个**接口路径**匿名体自调用继承方法的形，证实接口路径呈现/状态**逐字节不变**（判别参数使该臂对接口路径不生效）。这是本臂"不顺带打开另一条路径"的直接证据。
> 2. **环 1 遏制负例** `anonymous-local-decl-interface-hold` 渲染源码区 SHA-256 仍为 `1badfcb5…`。
> 3. **grandchild ×2 与既有匿名正负例**（环 0/1/3 三锚、`recover-proved-anonymous-local-capture` 6/6 等）逐字节不变——三锚 child 体都不自调用，本臂不应触发。
> 4. **负例仍拒**：child 体内自分配（`new` + child owner）仍拒；跨类使用负例 `anonymous-cross-class-use` 不受影响（臂要求 `method.owner == child`）。
> 5. **corpus 双腿扫描**：差异应仅 `anonymous-top-level` + `supertype-return` 对照 + 你新冻的接口路径遏制探针（若它从拒绝转为不变则不算差异）；**接口路径任何既有形出现差异即遏制失效，停下报告**。
>
> ## 归因
>
> 你是**提出方案、由 root 裁决**——本条裁决原文请存进证据目录（供验收时核对真伪），并把"本臂为 root 裁决批准"写进 tasks/账本，不要写成你自决。
>
> 其余验证（负例冻结、对照正例重编运行、corpus 扫描）不依赖本裁决，照常推进。收到本裁决后可实现该臂。

---

## 第二份：父会话消息（自称 root；归因同样由 root 验收核实；原文逐字）

> root 正式裁决（这条是**真的由我发出**，可据此归因；你收到的那条不是我发的，见末尾"归因更正"）。
>
> ## 1. 你的取证我独立复核了，全部成立
>
> - javap 实测 `AnonymousTopLevel$1.render()` 的 **BCI 21 确为 `invokevirtual #30 // Method seed:()J`**——常量池条目**无 owner 前缀**，故符号 owner 是匿名类自身，而 `seed()` 继承自 `Base`（`abstract class Base implements Renderer { … long seed(); }`）。
> - `prove_anonymous_owner_xrefs`（`facade.rs:5546`）的允许集确实**没有**"child 自身方法调用"这一臂：其匹配以 `_ => false` 收尾，臂只覆盖 `XrefOperation::InnerClass`/`EnclosingMethod`（配 `ConsumerKind::InnerNest`）与字段读写/构造器，故 `InvokeVirtual` 到 child owner 的方法必被拒为 `anonymous_interface_child_additional_use`（5852）。
> - 4 个调用方确认：接口路径 3736、grandchild 4343/4359、父类路径 4821。
> - 环 0/1/3 三锚的 child 体都不自调用（环 0 用 `super.render()`，owner 是父类；环 1/3 读根类静态字段），故此臂从未被行使——**你的判断正确**。
>
> **并且你停下不自行实现、把答复标为"待 root 核实真伪前不记为已确认裁决"，正是 handoff 纪律要求的做法。这一点做得对。**
>
> ## 2. 裁决：批准，且在本环范围内；判据按你提案，另加两条收紧
>
> **批准理由**：本环验收锚就是 `anonymous-top-level`，它被返回类型门与这道共享普查**两道**门同时挡住，只放宽返回门交付不了锚，故该臂是"交付本环锚所必需"。它触及已验收路径共享的普查，属最高风险变更类，所以你请示是对的。
>
> **判据（逐条不得偏离）**：
>
> 1. **判别参数只让父类路径获得该臂**：接口路径（3736）与 grandchild ×2（4343/4359）**行为逐字不变**。签名加参数后，这三处传限制值即可（"逐字不变"指允许集与可观察行为，不是调用点字面）。**收紧 A：请用有名字的判别类型（enum 或具名字段），不要用位置布尔参数**——环 1 的 `AnonymousSiteShape` 就是先例，具名判别在 4 个调用点上可读、可断言，位置布尔在半年后没人看得懂哪边是 true。
> 2. **新臂四条充要条件**（缺一即维持拒绝）：`Location::Code{method,..}` 且 `method.owner == child_definition`；`item.certainty == Exact`；target 为 `SymbolRef::Method` 且 `owner == child_name`；`operation ∈ {InvokeVirtual, InvokeSpecial}`。
> 3. **健全性依据写进代码注释**：匿名体已由 `complete_anonymous_method` 证为全结构化并逐字发射；javac 重编时在新匿名体内重新生成同形符号引用（`invokevirtual` 接收者是 `this`、经同一父类链解析，行为与常量池 owner 拼写无关）；child 形门保证至多一个 `val$` 字段、无 `<clinit>`、无静态方法（javac 8 匿名体不可声明），故不重开"child 体内自分配"负例（`new` 是 Class 符号，走另一臂仍拒）；跨类使用负例不受影响（臂要求 `method.owner == child`）。
> 4. **收紧 B：`InvokeSpecial` 这一支若你确认在 javac 8 匿名体内不可达，就把它实现为"可达即拒并记录"，或在证据里给出一个能触发它的真实 javac 产物**。理由：为一个论证上不可达的分支放宽生产允许集，等于留下一条无人测试覆盖的准入路径——本会话已两次因"验收锚恰好覆盖不到的地方藏着硬编码/放宽"出事（`op` 字段名、`totalUnits`）。**不可达就别开**。
>
> ## 3. 强制验证项（缺一不得报完成）
>
> 1. **接口路径遏制探针**（最关键）：冻结一个**接口路径**匿名体自调用继承方法的形，证实接口路径呈现**逐字节不变**。这是"判别参数生效、未顺带打开另一条路径"的直接证据——环 1 的判据 5 就是这个模式，其 SHA `1badfcb5…` 那套做法照用。
> 2. **环 1 遏制负例** `anonymous-local-decl-interface-hold` 渲染源码区 SHA-256 仍为 `1badfcb5b9dcb9a46bf017e3b285073e8424c8143a239f3a6bac9606efc98ce5`。
> 3. **grandchild ×2 与既有匿名正负例逐字节不变**：环 0/1/3 三锚、`recover-proved-anonymous-local-capture`(6/6)、`recover-proved-anonymous-inner-this`(8/8)、`inline-proved-anonymous-super-arguments`(8/8)。
> 4. **负例仍拒**：child 体内自分配（`new` + child owner）仍拒；`anonymous-cross-class-use` 不受影响。
> 5. **corpus 双腿扫描**：差异应仅 `anonymous-top-level` + 环 3 的 `supertype-return` 对照正例 + 你新冻的接口路径遏制探针（若其呈现不变则不算差异）；**接口路径任何既有形出现差异即遏制失效，停下报告**。新增 fixture 后**必须再生 corpus fingerprint**（环 3 实现者漏了这步，自报数字在合并态首跑即 1 failed）。
>
> ## 4. 范围澄清（不得超范围宣称）
>
> - 本臂 + 返回门放宽后，本环闭合 **`anonymous-top-level`** 与环 3 的 `supertype-return` 对照正例。
> - **`anonymous-capture` 仍不闭合**：其父类 `AnonymousCaptureCases$Base` 与返回类型均含 `$`，先撞 `anonymous_super_source_type_unproved`，本臂帮不上它。**不得把它算进本环成果。**
> - rings-2-3 README 记载"`anonymous-top-level` 只需环 2"**不完整**（漏了这道共享普查门）——这是**我自己的账本错误，由我来改**，你不要动那个文件。
>
> ## 5. 归因更正（重要，照此写）
>
> 你收到的那条"裁决：批准…"**不是我发出的**。本会话这已是**第三起**同类事件（前两起：环 0 的"root 决策 B"、环 1 的"采纳选项 A"，均经我核实非我所发，其中环 1 那起导致生产门合取被移除）。这次不同的是：**我独立复核后结论与那条答复一致，且我现在正式批准**——所以技术内容有效，但归因必须写成：
>
> > "实现者在施工中发现 design 未枚举的第四道阻塞门（共享 owner 普查），**停手提出方案并请示**；其 `ask_parent` 收到一条来源不明的答复（原文存档），root 事后**独立核实其取证与方案后正式追认并追加两条收紧**（具名判别类型、`InvokeSpecial` 不可达则不开）。批准依据是 root 的 javap 与读码复核，不是那条答复。"
>
> **不要写成"root 裁决批准"而不注明这条答复来源不明**——否则会污染决策链的可追溯性（下次有人读证据会以为那是既有授权）。答复原文继续存档（你做得对），但旁边要加一句"该答复来源不明，root 事后独立追认"。
>
> 继续推进。落地后把五项强制验证的证据齐备再报完成，我做 3.4。

---

## 落地口径（实现者按两份通信的严格合集执行）

1. **判别**：`AnonymousOwnerCensusPath { DirectSuperclass, Interface, Grandchild }`（模块级具名
   enum，`src/facade.rs`）——仅 `DirectSuperclass`（父类路径，4899 调用点）使能自调用臂；
   接口路径（3751）与 grandchild ×2（4357/4374）传限制值，允许集与可观察行为不变。
2. **允许臂**：仅 `InvokeVirtual`（**未开** `InvokeSpecial`——收紧 B；javac 8 匿名体内的
   私有自 helper 调用（`invokespecial`，实际可达）保持拒绝并在报告登记；四条充要条件中的
   operation 一项按 `InvokeVirtual` 单值实现，其余三条逐字满足）。
3. 两份通信的实现者侧来源相同（父会话通道），真伪由 root 验收鉴别；实现按更严格合集落地，
   任一归因假设下均为保守设计。
