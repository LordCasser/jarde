## ADDED Requirements

### Requirement: Recovered local declarations remain within lexical scope

恢复器 SHALL 只在 Java 词法作用域内输出局部变量声明与读取。跨 region 的读取 MUST 有位于共同可见作用域内的声明，并且其写入/赋值证据必须覆盖该读取所依赖的可执行路径；仅因局部槽或名称已在先前遍历中出现，不得视为它在后续 sibling 或外层 region 可见。无法证明时 MUST 拒绝包含相关定义与读取的最小完整结构，按既有 refusal 契约保留相应 bytecode、origin 和拒绝位置，不得输出越界名称或将效果静默丢弃。

#### Scenario: A handler-only local stays inside its catch clause
- **WHEN** 一个局部变量只在 catch 参数或 catch 块内声明、写入和读取
- **THEN** 恢复文本 SHALL 将它限制在该 catch 的词法作用域内；正文放入成员签名后 MUST 通过 javac，且执行时返回值、异常和可观察副作用与原 class 一致

#### Scenario: A local written in try and catch is read after their join
- **WHEN** 受保护区与 handler 向同一局部写入，控制流在 try/catch 后汇合并读取该局部
- **THEN** 若两侧写入和异常路径均可表示且已证明汇合处 definite assignment，恢复文本 SHALL 在包含 try 与汇合读取的共同作用域声明该局部，并保持各写入位置和异常语义；javac 与原 class 的执行对照 MUST 通过

#### Scenario: A preinitialized local has a computed update inside a loop's try
- **WHEN** 一个局部在循环与 try 之前被已呈现的赋值初始化，try 的正常路径用该局部的旧值与一次可能抛错的调用结果计算新值，catch 路径也写该局部，循环之后读取它
- **THEN** 恢复器 SHALL 以共同词法 owner 和完整定义—使用、表达式归属及异常边证据判断可恢复性；不得仅因正常路径写入不是整数常量而拒绝。证据闭合时，输出 MUST 把计算和调用保留在 try 内、调用恰好一次、catch 只覆盖原字节码保护的范围，并让原类与完整输出在正常和抛错输入上通过 Java 8 重编及执行对照
- **AND** 若计算值或任一写入不能完整呈现，恢复器 MUST 拒绝覆盖该局部依赖的结构，不得在外层声明一个后续读取却缺失必要赋值的变量

#### Scenario: A fallback region prevents a sound cross-region declaration
- **WHEN** try/catch 的某个定义区或其必要异常边只能作为 fallback 呈现，而方法后续仍读取该局部
- **THEN** 恢复器 MUST 拒绝覆盖该定义到读取的最小完整结构，拒绝范围 SHALL 同时覆盖相关 try/catch 区域和后续消费者；输出 MUST NOT 留下仅在 handler 内声明、却在 try 外读取的变量，也 MUST 保留范围内每条被拒指令的 bytecode 与 origin

#### Scenario: Nested or overlapping handlers do not leak sibling locals
- **WHEN** 嵌套 try、多个 handler 或交叉异常区域使一个局部的声明/使用路径不能被唯一放入合法共同作用域
- **THEN** 恢复器 MUST 保留已证明且不依赖该局部的结构，并拒绝完整的依赖切片；MUST NOT 仅凭 slot 复用、局部名称相同或线性 BCI 顺序将 handler 内局部提升或跨 handler 读取

#### Scenario: Independent catch parameters reuse one JVM slot
- **WHEN** 无 LVT 的两个 catch clause 复用同一局部 slot，且 handler 入口定义与 SSA 使用证明每个参数只在自己的 clause 内消费
- **THEN** 恢复器 SHALL 将它们视为独立词法绑定，按原异常表顺序恢复各 catch；MUST NOT 因 slot 数字相同而拒绝或把一个参数声明提升到另一 clause/方法块

#### Scenario: Missing debug tables do not weaken scope checks
- **WHEN** class 文件没有 LocalVariableTable 或源码行信息，局部只能以确定性 local 名称表示
- **THEN** 所有声明范围和跨 region 读取仍 SHALL 由局部身份与控制流证据决定；缺少 debug metadata MUST NOT 放宽词法作用域约束
