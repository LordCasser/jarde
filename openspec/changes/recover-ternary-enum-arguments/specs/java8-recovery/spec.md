## ADDED Requirements

### Requirement: Proved int ternary enum constructor arguments

对 Java 8 enum 常量实参中的 int 条件表达式，系统 SHALL 仅在完整 `<clinit>` Code 证明一个布尔条件、两个 descriptor-compatible int literal arm 和唯一汇合点，且汇合值唯一消费于对应 enum constructor call 时输出 `condition ? trueArm : falseArm`。条件及其可观察效果 MUST 与原 class 求值一次并保持在原 constructor call 前；左右 arm 与 source literal MUST 按原分支语义匹配。该证明 MUST 与同一完整常量组、常量字段写入顺序和 `$VALUES` 前缀绑定。多消费者、额外前驱/后继、未解释指令、异常/预算/取消停止 MUST 原子拒绝整个组，保留物理 enum 字段和 initializer 来源。String 与 varargs 构造实参不属于本要求。

#### Scenario: A conditional int value reaches one enum constructor argument
- **WHEN** enum initializer 对每个常量都执行一个完整、唯一的条件 branch diamond，其两个 int literal arm 汇合后作为该常量唯一 constructor int 参数，且 predicate 的每次 invocation 顺序与次数可证明
- **THEN** 源 enum SHALL 以等价 `condition ? trueArm : falseArm` 拼写每项构造实参；原始 class、JADX 与 Jarde 完整源码均按 Java 8 编译并以 `-Xverify:all` 运行相同，含能分别选择 true/false arm 的测试输入

#### Scenario: A ternary with ambiguous or effectful arms is refused
- **WHEN** branch arm 有额外字段写/调用，条件或 arm 发生多次消费、control-flow join 不唯一、BCI/constructor 对应不一致、handler 覆盖该区域或完整事实读取停止
- **THEN** 系统 MUST 不生成该 ternary，也 MUST 不只投影部分 enum 常量；完整 enum group 保持物理表示并报告拒绝/停止来源

#### Scenario: Literal integer constructor arguments remain supported
- **WHEN** enum 实参是已有证书支持的整数 literal，没有条件分支
- **THEN** 现有 literal 投影行为 SHALL 保持，且不制造条件或额外求值
