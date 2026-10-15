# jvm-ir Specification

## Purpose

从共享 reader 的保真指令事实建立有界 JVM 方法 IR，处理历史子程序、异常和栈语义，为后续 Java 恢复提供有 origin 与不变量的输入。

## Requirements

### Requirement: Typed bytecode facts and target validation

IR SHALL 消费共享 reader 提供的类型化 immediate、local、CP、branch 和 switch 操作数，保留 raw opcode、BCI、字节范围、异常表顺序及读取终止位置。原始 facts MUST 不被规范化覆盖。所有控制流目标和保护区间 SHALL 在 checked 运算后核对有效指令边界。

#### Scenario: Branch enters an operand

- **WHEN** branch/switch/handler target 指向操作数字节、越界或计算溢出
- **THEN** 返回带原 BCI 的无效目标诊断，不生成看似有效的 CanonicalCFG

#### Scenario: Switch and wide operands

- **WHEN** 方法含 wide local/iinc、正负相对分支、tableswitch 或 lookupswitch
- **THEN** 类型化 facts 与原始字节及指令边界一致，保留 switch default/key/target 和 local/immediate，不通过展示字符串重建语义

#### Scenario: Effective opcode and allocation operands

- **WHEN** 方法含 wide load/store/ret、newarray、multianewarray 或 invokeinterface
- **THEN** 共享 facts 保留有效 opcode、atype、dimensions 与 count；CFG/Frame 使用该事实，raw opcode/width/BCI 保持原样，不能因有效 opcode 丢失而漏掉 ret 终结或误拒绝现代 wide load/store

### Requirement: Bounded analysis storage and work

闭包和 IR 阶段 SHALL 共用一个请求预算生命周期。系统 MUST 在分配、排队、加边和克隆前计费，并限制 IR 存储项、边、分析步骤与规范化克隆；frame 槽、phi 输入和 origin 成员也必须计入，不能仅限制 block 数。输出 SHALL 继续受结果/字节限额约束。

#### Scenario: Small input causes large derived state

- **WHEN** 小方法的高扇出异常边、大 locals 状态或共享子程序导致派生结构放大
- **THEN** 在超过已声明限额的分配前停止并报告相应 usage/reason；不以输入字节已经有界替代派生存储上界

#### Scenario: Worklist does not converge within budget

- **WHEN** Frame/SSA 或 returnAddress 工作列表耗尽步骤/时间预算或收到取消
- **THEN** 停止并保留最后有效阶段，不把半初始化 facts 发布为完整分析

#### Scenario: Private call-context storage exhausts its limit

- **WHEN** returnAddress 分析需要创建 context、状态槽、worklist、local/token/handler 关系或装配结果，而剩余 IrItems 不足
- **THEN** 在增长前停止，保持相应预算原因且不发布 CallContexts；crate-private 载荷、临时状态和乘积数量不因输入已计费而豁免，IrItems 为零时不能成功创建非空上下文

#### Scenario: Throw-site snapshots multiply local storage

- **WHEN** 一个 block 在多个 throw-site 同时保留 max_locals 槽的快照，或 Frame/SSA 发布阶段复制 entry/exit/phi/origin 集合
- **THEN** 所有实际新增并保留的槽与成员 SHALL 在分配前计费；移动已计费载荷可以复用其计费，不以“阶段内临时量”豁免乘积存储，也不能在 collect/clone 后才发现限额不足

### Requirement: Phase-ordered JVM IR

系统 SHALL 按 raw facts、raw CFG/returnAddress、dialect normalization、CanonicalCFG、Frame、stack/local SSA 与 type/effect 前置关系创建 IR。每个 Pass MUST 声明 phase、required/produced facts、失效分析和实际可能计费的 budget 类别集合；dialect/capability 与 scope 由请求和固定阶段契约约束。Region/Java AST 不属于本阶段输出。

#### Scenario: Pass dependency violation

- **WHEN** 配置在 Frame facts 前运行依赖 Frame 的 pass，或依赖有环
- **THEN** 启动或请求校验拒绝该顺序并报告原因，不执行半初始化 IR

#### Scenario: Analysis invalidation

- **WHEN** pass 改变 CFG 或异常边
- **THEN** dominator、liveness、SSA 等相关分析失效，后续必须重新计算或拒绝使用

#### Scenario: Call-context analysis consumes effects

- **WHEN** 调用上下文阶段要读取 effects，但该事实尚未产出或已经失效
- **THEN** requires 校验拒绝执行；阶段成功必须保留实际产物供后继使用，不能只登记 produced 而丢弃其载荷

### Requirement: Legacy normalization before canonical frames

系统 SHALL 先分析 raw CFG、returnAddress 和调用上下文，再在预算内规范化 jsr/jsr_w/ret。解码可读性、dialect 合法性与 verification MUST 分开报告；非法版本或不能可靠规范化时保留原始 Bytecode 与原因。

#### Scenario: Historical finally under Java 8 profile

- **WHEN** Java 8 profile 读取包含历史 jsr/ret 的允许版本 class
- **THEN** 保留共享/嵌套调用上下文及异常范围，规范化节点可映射同一原 BCI；超界或不支持时明确 fallback，不能用线性替换伪称语义完整（验收 A09）

#### Scenario: Forbidden legacy opcode in modern class version

- **WHEN** classfile 51+ 包含 jsr/jsr_w/ret
- **THEN** 原始取证事实仍可展示，但报告 dialect 违规，不将其标为合法规范化输入

#### Scenario: Ret reads the wrong or overwritten local

- **WHEN** jsr 返回地址存入 local 0，但 ret 读取 local 1，或原槽已被普通值覆盖
- **THEN** 不得仅因 CFG 可达而建立返回边；报告无法证明的 returnAddress 值流并 fallback，不发布可被规范化消费的完整 CallContexts

#### Scenario: Reference store is not return address proof

- **WHEN** ret 所读槽只有一次支配它的 astore，但该指令存入 null/普通引用，或 jsr token 已被 pop 丢弃
- **THEN** MUST NOT 仅按写入位置建立返回点；不发布可供规范化消费的完整 CallContexts，阶段不得报告 Completed，保留原 Bytecode 与无法证明的原因

#### Scenario: Nested subroutine overwrites an outer return address

- **WHEN** 内层子程序改写外层 ret 所读取的地址槽，并返回外层 continuation
- **THEN** 外层证明 SHALL 消费内层写入后的状态；不能因为写入属于不同 active context 而忽略它，不可靠返回点必须阻止规范化

#### Scenario: Aload cannot transfer a return address

- **WHEN** astore 保存了 jsr token 后，aload 或 aload_n 尝试把它作为引用加载
- **THEN** MUST NOT 将该形态作为合法返回地址中转或规范化证据；保留诊断与原始 facts，verification 仍为 NotPerformed

#### Scenario: Handler returns within a subroutine

- **WHEN** 子程序抛出后由 handler 改写 locals 并回接 ret
- **THEN** 按 throw-site 与 active context 传播 handler 状态，包含该路径的 locals 和返回地址变更；无法确定归属时 fallback，不得跳过全部异常边后声称上下文完整

### Requirement: JVM frame exception and effect semantics

Frame/SSA SHALL 处理 category-1/category-2、双槽、dup/swap、uninitializedThis、new-site、初始化转换、handler entry 和 null/数组/引用合流。异常边 SHALL 保留 throwing instruction、handler 顺序、保护区间和该点 locals/effect 状态。未知类型或 effect MUST 保守保留。

#### Scenario: Overlapping exception regions

- **WHEN** 同一保护区间的不同 throwing instruction 具有不同 locals/effect，且 handler 顺序影响选择
- **THEN** handler 输入保留各 throw-site 的对应状态，不统一使用 block 尾部状态，也不改变 effect 顺序

#### Scenario: Canonical SSA merge

- **WHEN** 正常或异常 predecessor 合流产生 stack/local phi
- **THEN** 输入对应真实 predecessor 与值形状，定义/use、category-2 和 origin 不变量均通过；矛盾时返回诊断和最后有效阶段

#### Scenario: Incompatible dead locals merge

- **WHEN** 两条路径给同一 local 写入不兼容的值，但合流后不读取该槽
- **THEN** local 合流为不可用 Top，不仅因此拒绝方法；后续读取 Top 才诊断失败，operand stack 的形状冲突仍须拒绝

#### Scenario: Constructor initializes aliases

- **WHEN** 同一 new-site 或 uninitializedThis token 经 dup/astore 留下多个别名，适用的 invokespecial init 正常完成
- **THEN** 当前 frame 的 stack/locals 中全部同 token 别名同步初始化；构造器返回 void，异常后继不能套用正常完成后的初始化状态

#### Scenario: Multiple throw sites share one raw edge

- **WHEN** 同一 block 内两个 throwing instruction 经同一 handler 记录到达入口，raw CFG 聚合为一条异常边
- **THEN** Frame/SSA 仍区分各 throw-site/context 的逻辑输入及 locals/effect，不用聚合的 raw edge 数替代 phi 输入数

#### Scenario: SSA does not depend on internal block storage order

- **WHEN** 同一 CanonicalCFG 在预算充足时仅改变内部 block 存储顺序，控制流、指令顺序、throw-site/context 与物理身份均不变，且某 use 的前驱定义尚待处理
- **THEN** 最终 use 来源、phi 输入对应关系及 origin/effect 保持等价；前驱尚未处理不得被最终解释为无定义或可用 undef，循环回边也不得因遍历顺序遗漏

### Requirement: IR invariants and honest verification

每层 SHALL 保留物理方法身份、OriginSet 和 diagnostics；origin 以 class offset/BCI 锚定，规范化不能制造新的物理 XRef。未完整执行规范 verifier 时 MUST 返回 verification=NotPerformed。

#### Scenario: Missing debug or stack maps

- **WHEN** 缺少 LVT、LineNumberTable 或 StackMapTable
- **THEN** 从 descriptor 和数据流分析 Frame，按版本诊断必要约束；推导成功不等于输入通过 verifier，也不能把缺少 debug 当作无法分析的理由（验收 A10）

#### Scenario: Normal exit is unchanged while exception input changes

- **WHEN** 回边或另一前驱改变了 block 入口的 local，较早 throw-site 消费该值，而该块在正常出口前又把槽覆盖成与上一轮相同的值
- **THEN** Frame SHALL 继续传播已变化的逐 throw-site 输入直到固定点，handler entry 与 SSA 输入必须一致；正常出口相等不得跳过异常更新，也不得通过放宽 SSA 一致性检查接受陈旧 Frame

### Requirement: Canonical block protection reflects physical member ranges

完整 canonical 图 SHALL 对每个 block 发布按物理异常表声明顺序去重的保护 ordinal：当且仅当该节点至少一个真实原始成员块内有指令位于该半开保护范围。融合节点 MUST 合并成员的范围事实，不能把非连续块之间的包络空隙算作成员；clone 的事实保持其原始块、调用路径与物理方法身份。该字段不承诺整个融合节点均受保护。

#### Scenario: Protected nonthrowing member remains visible

- **WHEN** 一个原始块包含受某异常保护范围覆盖的非抛出指令，而同一记录的实际异常边只来自其他 throwing block
- **THEN** 本块的公开保护 ordinal 包含该记录，handler-row 与 throw-site 事实仍仅按其原有异常边契约发布，不因本字段扩大异常输入

#### Scenario: Fused members cover only part of the node

- **WHEN** 融合节点的一部分真实成员受保护，其他成员不在范围内
- **THEN** 保护 ordinal 为真实成员范围事实的有序去重并集，物理原始块和 origin 保持；不得解释为整个节点的全覆盖

#### Scenario: Forward fusion skips a protected gap

- **WHEN** 前向跳转融合两个非连续块，其包络内的另一个原始块受保护，但融合成员都不受该记录保护
- **THEN** 融合节点不发布该 ordinal，被跳过的块仍按自身真实范围发布，不由包络推导覆盖

#### Scenario: Boundaries overlapping rows and clones preserve identity

- **WHEN** 保护范围端点等于块边界、多条记录重叠，或同一原始块在多个 live/dead jsr 调用路径下克隆
- **THEN** 使用半开边界核实际成员，多个 ordinal 按声明顺序保留一次；各 clone 仅描述自己的真实成员与路径，不能被折叠为一份物理控制流证据

### Requirement: Canonical protection growth obeys normalization budgets

系统 SHALL 在新增范围扫描、ordinal 存储及保留副本增长前按现有 analysis/storage 预算计费；既有载荷移动可复用已计费存储。预算不足或取消时 MUST 按既有 normalization 终止契约保留原因和最后有效阶段，不发布带部分保护事实的完整 canonical 图；无异常记录的图不因本功能增加无用扫描或 ordinal 存储。

#### Scenario: Protection storage or work cannot be admitted

- **WHEN** 真实保护 ordinal、融合成员或装配处理的剩余 IrItems/AnalysisSteps 不足，或处理中收到取消
- **THEN** 在相应增长前停止，保持预算维度或取消原因，不把不完整范围事实发布为成功图
