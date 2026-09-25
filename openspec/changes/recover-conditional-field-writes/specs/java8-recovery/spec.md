## ADDED Requirements

### Requirement: A short-circuit value consumed by one static field write keeps its effect

Java 8 恢复结果遇到短路条件经控制流汇合后供给一次 `putstatic` 时，MUST 保留原条件的短路求值、异常顺序、目标字段身份和恰好一次写入。若这些事实已证明，正文 SHALL 把该条件值写入原字段，且重编执行的字段值与原 class 相同。若无法证明等价 Java 表达式，MUST 保留该 `putstatic` 及必要值生产者的来源缺口，MUST NOT 把只有部分分支而没有字段写入的可编译正文标为完整恢复。此要求不凭目标字段的 `Z` 描述符把任意 JVM 整数归一为布尔值。

#### Scenario: A constructor writes a short-circuit result after virtual dispatch

- **WHEN** `Base()` 虚调用子类方法后，把 `inBaseConstructor && "captured-value".equals(observed)` 的结果通过汇合栈值写入静态布尔字段
- **THEN** 已证明的 Java 8 正文执行 SHALL 与原 class 一样输出 `visibleDuringSuper=true`；条件右侧只在左侧为真时执行，字段写入恰好一次

#### Scenario: An unproved join retains the write as a visible gap

- **WHEN** 汇合处还有额外入口、值来源不唯一或条件臂带有不能嵌入表达式的效果
- **THEN** 正文 MUST 保留目标 `putstatic` 的 BCI 与必要生产者为缺口，不能让一个省略写入的可编译类被认作等价结果

#### Scenario: A near-miss diamond is not split into incomplete nested ifs

- **WHEN** 两层条件仍经两个生产者汇入静态字段写入，但 `javac --release 8` 的 `a || rhs()` 使两条边共享 true 生产者、生产者不再是 1/0，或消费块先复制 Phi 后才写字段
- **THEN** Region 所有权 MUST 在普通嵌套 `If` 前覆盖该局部，发射证明失败时 MUST 完整引用分支、生产者、字段写入及同块后缀的指令 BCI，且报告 MUST NOT 为完整结构化 Java；MUST NOT 仅重复引用消费 BCI 而遗漏先行来源

#### Scenario: A protected short-circuit candidate has a throwing right operand

- **WHEN** `javac --release 8` 的 `result = left && mayThrow()` 位于带真实 `RuntimeException` 处理边的 `try` 内，且两次测试、两个生产者汇入一个 `putstatic`
- **THEN** 在无法同时证明正常路径与异常路径的 Java 语义时，恢复 MUST 一次引用该局部的测试 BCI 0/1/4/7、生产者 BCI 10/11/14 与字段写入 BCI 15，source map MUST 保留这些指令的来源；MUST NOT 在嵌套 `If` 中丢失真值生产者或生成无字段写入却声称完整的正文。已有 `catch` 的独立来源仍须保留

#### Scenario: An integer phi is not inferred as a boolean literal

- **WHEN** 目标字段为 `Z`，但汇合两臂供给非规范的整数值而非可证明的布尔表达式
- **THEN** 恢复 MUST 保留原 JVM 字段存储语义或拒绝表达式，MUST NOT 仅凭 `Z` 把两臂改写成 `true`/`false`
