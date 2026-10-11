# CF12 同类常量投影：v5 语义边界审核

状态：只读源码分析；未修改 workspace，也未运行 Git、Cargo、rustfmt、JDK 或 CLI 工具链。

## 结论

`integer_constant_projections` 改写的是 root 自己的方法 AST 中由整数值识别出的常量字段引用。这个投影的身份与安全性来自当前物理 class read 的字段候选、该次方法恢复保留的 AST、方法内 shadow/重复值过滤、现有文本重建相等检查，以及输出范围所带的物理 field 和 method anchors。root 的 member-family scan 负责的是是否能把 child/family 文本装配进 root source unit。二者证明的对象不同。

因此，对于完整 execution 下的 `ClassSourceMemberFamily::Refused { child: None, .. }`，不应要求 refusal 唯一来自 root self row，也不应要求没有 direct-child row。只要三个 sibling family 仍精确为 `Absent`，同类常量投影自身的 current-text/rebuild gate 仍然生效，那么允许该 `RefusedNone` 不会把 child 文本引入输出，也不会把 root 字段绑定到 outer。

这是应覆盖真实 Labels 目标的原因：给定 `render-root-v1` 的 javap 证据，`TestSwitchLabels$TestCls` 的 `InnerClasses` 同时包含 root self row 和 direct child row。scanner 按顺序在 root self row 上 Refused；v3/v4 的 no-direct-child 条件虽能归因拒绝，却挡住了该目标。family projection 仍然是 refused，root source 仍不渲染 child；允许同类字段名只补足 root 方法中现有物理常量的源代码拼写。

## 责任分界

同类常量候选只从同一次 root class read 的完整字段表派生。每个候选要求该字段有可写声明、`static final`、descriptor 为 `I`、恰好一个 `ConstantValue`、读取结果为整数、字段名 UTF-8 且为 Java identifier；整个字段表和结构必须完整。随后按所有字段名拒绝重复名称，并按候选字段值拒绝重复常量值。该列表带着 root `ClassSourceField` 的物理 identity/index，不查询 InnerClasses、child 或 outer 的字段。

根方法 AST 是在同一轮 root method prepare/recovery 中保留，并以 `PhysicalMethodId` 与同一 report 中的 method 精确匹配。请求只采集包含 integer `switch` 或可处理的 `int[]` return initializer 的 AST。恢复必须是 `Structured + Java + ContainsStatements + no fallbacks`；AST 投影本身还要求 `complete_code`、完整参数名和实际整数使用。

投影器收集参数和 AST statement 中的局部变量、foreach、for、try resource、catch 名称，组成保守的 method-wide occupied set。候选名被任一这些声明占用时不投影；重复候选数值已经由 facade 过滤。对普通 switch/return，只有 AST 的 `int` 值、限定结构和候选映射匹配时才改名。对数组 initializer 仅接受方法直接返回的一维 `int[]` 初始化叶子，要求节点的 `presented == int`、来源 method 匹配、BCI 存在于同 AST 指令表，并用 emit replay map 将对应文本片段唯一映射回同一个 method/BCI。

switch-only 改写也不是直接改输出字符串。它从变换后的同一 AST 重新 emit body，保留原 recovery envelope、当前 method declaration 和 annotations；markers 非空时拒绝。数组用法额外要求 AST 原始 body 重建后的完整方法文本等于当前 method text，并跳过已有同 member staged text。最终 `source_text_with_method_projections` 先用 ordinary writer 从当前 root declaration/fields/methods 重建整份 `report.text`，不相等就拒绝。它再组装 staged method text，要求 derived 数量精确、每个 span 在最终文本内有效且非空、每个 entry 有 anchor。每个投影 span 对应 `IntegerConstantName`，锚到候选字段的 `PhysicalFieldId + index` 和当前方法的 `PhysicalMethodId + BCI`；range 位于重建的完整 root source text 中。

这条链约束的是“root source 本身可重建、字段和使用点来自真实物理成员、重命名不改变常量值和控制流”。它不依赖 member-family refusal 的具体原因。`RefusedNone` 不持有 child report，不表明 child text 已装配；若 parent-child projection 成功，family 会是 Prepared/PreparedPair/PreparedStatic/PreparedFold 等状态。若 child 准备过但关系投影拒绝，通常是 `Refused { child: Some(..) }`，仍必须挡住。

## 拒绝边界与实际反例检查

### `RefusedNone` 内可能存在 direct child

这不是同类常量投影的语义反例。child row 会影响独立的 source-family 证明；它不会改变 root 自己的 static-final-int 字段值，也不会改变 root method bytecode/AST 对 int switch key 的物理对应。只要 `RefusedNone` 下没有已装配 child 文本，增加 `case VALUE:` 仍指向 root 当前声明的同一个 compile-time constant。外类没有隐式词法成员查找进入一个单独输出的 Java 顶层类；且只做 unqualified same-class name，不尝试 `Outer.VALUE` 绑定。

如果存在任何直接 child row并成功装配，`member_family` 不能仍为 `RefusedNone`。如果候选 child 有物理 report 但 source family 关系拒绝，family 变体带 `child: Some`，由 gate 拦住。故不需要用 direct-child 的存在与否替代 member-family 状态来判断是否有 child 文本。

### 同类名被局部值遮蔽

一般方法参数和声明名称在投影前被放进 occupied set，候选同名时不替换。其遍历覆盖 block/if/loop/switch/try 的 statement 子树；对数组候选同样执行。测试已覆盖数组参数 shadow 与局部变量 shadow。

源码层面值得保留的残余核查点：occupied-set walk 不遍历表达式内部的 lambda parameter。当前 projector 只能改写 AST `StmtKind::Switch` 与直接 `StmtKind::Return` 的整数，并把数组限制为方法顶层的直接 return initializer；表达式 lambda 的 body 是 Expr，不能承载可被本 pass 遍历/改写的 switch statement。故目前没有可达的 lambda-parameter shadow 反例。若未来 AST 新增 switch-expression、block lambda 或局部类型语句并将其纳入此 projector，必须同步扩展 lexical-scope/shadow walk，不能沿用现有保守条件。

字段候选的名字另受完整 field-table 唯一名检查；同名字段重复时不会入候选。名称空间中的类型名与值名不同，不会因直接 nested child 的类型名碰撞而改变 unqualified value lookup。候选具有 root 自己字段声明，所以 Java field lookup 的 own-class member 优先于 inherited/outer scope；outer binding 不发生。

### 同一个 method 先被另一种 projection 改写

核心投影是在当前 method text 上重新拼 envelope+AST body。它可能覆盖与 AST 不一致的 staged body，因此必须确认 writer-equality 与各 projection gate 的实际组合。当前 pipeline 中：

- assert sugar 在候选非空时由 `project_class_source_assert_statements` 直接拒绝；
- array-helper 与 lambda-helper 输出通过 ordinary source writer 之外的 staged method-text/context channels 组装，integer writer 用无这些 channels 的 ordinary writer 重建；实际 text 改变时 equality 失败。数组类 use 还有同 member staged-text 与 AST 原始 body equality 检查；
- enum initializer/declaration、nested enum/annotation、member child/fold、anonymous child 等若声明了 source claim，report 会有对应 non-Absent family state或改变 ordinary writer 之外的 text，因此 sibling gate 或 writer equality 阻挡；
- direct-override projection改变的是声明/annotations，不改物理 body；integer method 重建用当前 declaration。带 source marker 的方法会被 `integer_constant_projection_text` 拒绝；
- pool-spelled structural rerun 若替换当前 method，guard 结果是 refusal/非 statement body；integer pass 只处理 `Structured + ContainsStatements + no fallbacks` 的 recovered report。未被 rerun 替换的 method仍与原 retained AST 同一物理 identity。

switch-only 单独没有数组分支的 `original_body_matches`，这是一个必须被 pipeline 事实约束的点。现有可能改写 root body 的前置 projections 要么因候选非空而不启动，要么采用 ordinary writer 无法重现的 staged channel，或留下非-statement recovery而被跳过；没有发现一个在 Complete + three siblings Absent + member RefusedNone 情况下能让 root writer equality 通过、却使当前 switch method body 与保留 AST 不一致的现存路径。未来若增加直接写入 `method.text` 且普通 writer会随之重建的 body projection，需给 switch-only path 加同等的 original-body equality/staged-member check。

### 字段值、BCI 和 source-map 失效

不依赖 nesting 的潜在错映射需要候选/AST/field physical identity 不再属于同一 root 读取，或来源 BCI 不能唯一定位。当前 facade 在一个 `prepare_physical_class_source` 过程中构造字段候选、methods 和 ASTs；AST以 physical method identity 配对；switch-label BCI来自该 AST 的 switch statement origin；数组 additionally 验证 instruction BCI 和 source-map replay。最终 field anchor 来自相同候选对象，method anchor来自当前 method record。writer必须重建当前全文相等并验证 derived spans。没有看到 direct-child/self-row 状态能让这些坐标失效。

一个可测试的真实性约束是：根字段候选内容不能通过外类同名字段来补齐。若 root 无字段候选，列表为空，不会投影；若字段值有重复或 name shadow，candidate被过滤或被投影器拒绝。`Inner → outer` 常量绑定仍是单独的产品债务，不属于本次允许范围。

## 最小 gate 建议（未应用）

在当前完整 execution 检查内，将 member-family 的精确 `Absent` 允许条件放宽为：

```rust
matches!(
    report.member_family,
    class_source::ClassSourceMemberFamily::Absent
        | class_source::ClassSourceMemberFamily::Refused { child: None, .. }
)
```

三个 sibling gate继续要求各自精确 `Absent`；保留 Complete execution、Recovered method Complete、candidate non-empty、structured Java statement/no-fallback AST、shadow/duplicate过滤、array same-member replay/current-body checks、final full writer equality、derived anchor/span验证。`Prepared`、`PreparedPair`、`PreparedStatic`、`PreparedFold`、`Refused { child: Some(..) }`、`RefusedPair` 一律不允许。无需向 helper 传 `root_nesting`，无需解析 refusal reason、识别拒绝 cause 或附加 no-direct-child 条件。

执行完整性 gate在产品入口仍然有价值：停止态不能因局部候选/AST 已存在就继续把 partial evidence当作当前可发布 root presentation。execution gate是独立的原子发布条件，不能因 member-family 无 child而移除。

## 建议的验证目标

1. 使用真实 `TestSwitchLabels$TestCls` frozen class：报告确认 root self row 和 direct-child 同时存在、member family仍 RefusedNone、siblings Absent；验证同类 `switch` 常量投影发生且 derived entries锚到 root 的物理字段与 root method BCI。
2. 仍拒绝 `Refused { child: Some(..) }` 与 `RefusedPair`，并分别拒绝三种 sibling family 的非-Absent状态。
3. 通过现有同类常量用例确保 duplicate-value、duplicate-field-name、参数/local shadow、unsupported array形态和 method AST缺失仍不产生 projection。
4. 依赖真实 full-text writer equality覆盖一个先前 family/array/initializer text claim：若 ordinary writer无法完整重现当前 report.text，投影保持为空。
5. 不新增 outer lookup，也不修改 child/family recovery/fallback policy。
