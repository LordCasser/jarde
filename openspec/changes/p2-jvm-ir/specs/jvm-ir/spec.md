## Purpose

从共享 reader 的保真指令事实建立有界 JVM 方法 IR，处理历史子程序、异常和栈语义，为后续 Java 恢复提供有 origin 与不变量的输入。

## ADDED Requirements

### Requirement: Typed bytecode facts and target validation

IR SHALL 消费共享 reader 提供的类型化 immediate、local、CP、branch 和 switch 操作数，保留 raw opcode、BCI、字节范围、异常表顺序及读取终止位置。原始 facts MUST 不被规范化覆盖。所有控制流目标和保护区间 SHALL 在 checked 运算后核对有效指令边界。

#### Scenario: Branch enters an operand

- **WHEN** branch/switch/handler target 指向操作数字节、越界或计算溢出
- **THEN** 返回带原 BCI 的无效目标诊断，不生成看似有效的 CanonicalCFG

#### Scenario: Switch and wide operands

- **WHEN** 方法含 wide local/iinc、正负相对分支、tableswitch 或 lookupswitch
- **THEN** 类型化 facts 与原始字节及指令边界一致，保留 switch default/key/target 和 local/immediate，不通过展示字符串重建语义

### Requirement: Bounded analysis storage and work

闭包和 IR 阶段 SHALL 共用一个请求预算生命周期。系统 MUST 在分配、排队、加边和克隆前计费，并限制 IR 存储项、边、分析步骤与规范化克隆；frame 槽、phi 输入和 origin 成员也必须计入，不能仅限制 block 数。输出 SHALL 继续受结果/字节限额约束。

#### Scenario: Small input causes large derived state

- **WHEN** 小方法的高扇出异常边、大 locals 状态或共享子程序导致派生结构放大
- **THEN** 在超过已声明限额的分配前停止并报告相应 usage/reason；不以输入字节已经有界替代派生存储上界

#### Scenario: Worklist does not converge within budget

- **WHEN** Frame/SSA 或 returnAddress 工作列表耗尽步骤/时间预算或收到取消
- **THEN** 停止并保留最后有效阶段，不把半初始化 facts 发布为完整分析

### Requirement: Phase-ordered JVM IR

系统 SHALL 按 raw facts、raw CFG/returnAddress、dialect normalization、CanonicalCFG、Frame、stack/local SSA 与 type/effect 前置关系创建 IR。每个 Pass MUST 声明 phase、required/produced facts、失效分析、dialect/capability、scope 和 budget class。Region/Java AST 不属于本阶段输出。

#### Scenario: Pass dependency violation

- **WHEN** 配置在 Frame facts 前运行依赖 Frame 的 pass，或依赖有环
- **THEN** 启动或请求校验拒绝该顺序并报告原因，不执行半初始化 IR

#### Scenario: Analysis invalidation

- **WHEN** pass 改变 CFG 或异常边
- **THEN** dominator、liveness、SSA 等相关分析失效，后续必须重新计算或拒绝使用

### Requirement: Legacy normalization before canonical frames

系统 SHALL 先分析 raw CFG、returnAddress 和调用上下文，再在预算内规范化 jsr/jsr_w/ret。解码可读性、dialect 合法性与 verification MUST 分开报告；非法版本或不能可靠规范化时保留原始 Bytecode 与原因。

#### Scenario: Historical finally under Java 8 profile

- **WHEN** Java 8 profile 读取包含历史 jsr/ret 的允许版本 class
- **THEN** 保留共享/嵌套调用上下文及异常范围，规范化节点可映射同一原 BCI；超界或不支持时明确 fallback，不能用线性替换伪称语义完整（验收 A09）

#### Scenario: Forbidden legacy opcode in modern class version

- **WHEN** classfile 51+ 包含 jsr/jsr_w/ret
- **THEN** 原始取证事实仍可展示，但报告 dialect 违规，不将其标为合法规范化输入

### Requirement: JVM frame exception and effect semantics

Frame/SSA SHALL 处理 category-1/category-2、双槽、dup/swap、uninitializedThis、new-site、初始化转换、handler entry 和 null/数组/引用合流。异常边 SHALL 保留 throwing instruction、handler 顺序、保护区间和该点 locals/effect 状态。未知类型或 effect MUST 保守保留。

#### Scenario: Overlapping exception regions

- **WHEN** 同一保护区间的不同 throwing instruction 具有不同 locals/effect，且 handler 顺序影响选择
- **THEN** handler 输入保留各 throw-site 的对应状态，不统一使用 block 尾部状态，也不改变 effect 顺序

#### Scenario: Canonical SSA merge

- **WHEN** 正常或异常 predecessor 合流产生 stack/local phi
- **THEN** 输入对应真实 predecessor 与值形状，定义/use、category-2 和 origin 不变量均通过；矛盾时返回诊断和最后有效阶段

### Requirement: IR invariants and honest verification

每层 SHALL 保留物理方法身份、OriginSet 和 diagnostics；origin 以 class offset/BCI 锚定，规范化不能制造新的物理 XRef。未完整执行规范 verifier 时 MUST 返回 verification=NotPerformed。

#### Scenario: Missing debug or stack maps

- **WHEN** 缺少 LVT、LineNumberTable 或 StackMapTable
- **THEN** 从 descriptor 和数据流分析 Frame，按版本诊断必要约束；推导成功不等于输入通过 verifier，也不能把缺少 debug 当作无法分析的理由（验收 A10）
