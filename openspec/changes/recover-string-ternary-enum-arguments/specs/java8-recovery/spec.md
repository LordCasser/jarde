## ADDED Requirements

### Requirement: Proved scalar String enum constructor arguments

对 Java 8 普通 enum 的单个 String 构造实参，系统 SHALL 仅在完整类和初始化器证据证明常量字段、构造器、参数值及唯一拥有的 String 字段写入相互一致时输出源级 enum 常量。实参可为可准确呈现的 ASCII String 字面量，或一个布尔条件在两个此类字面量之间作选择的条件表达式。条件 MUST 在对应构造器调用前求值恰一次；分支极性、两个 arm、常量顺序和可观察副作用 MUST 与原 class 一致。此证书不将 String 数组或 varargs 值当作标量 String。

#### Scenario: A literal scalar String argument is proved
- **WHEN** 每个常量的一个 String 字面量是其构造器 String 参数的唯一来源，构造器仅将该参数写入准确的本类 String 字段，且完整 enum 组闭合
- **THEN** 系统 SHALL 输出带该字面量实参的完整 enum 源码；原 class、JADX 与 Jarde 完整源码按 Java 8 编译和验证运行结果一致

#### Scenario: A conditional scalar String argument is proved
- **WHEN** 同类静态布尔条件调用后，两个 String 字面量 arm 各沿唯一分支汇合到该常量的构造器 String 参数，且所有常量均满足完整证据
- **THEN** 系统 SHALL 输出按真实极性排列的 `condition ? trueArm : falseArm`；原 class、JADX 与 Jarde 完整源码以 `java -Xverify:all` 运行时，两种 arm 的选择、条件调用次数和顺序一致

#### Scenario: An ambiguous or effectful argument is refused atomically
- **WHEN** 任一 arm 另有调用或字段写入、存在额外前驱/消费者或嵌套分支、分支目标/汇合/构造器或字段身份不唯一、异常路径覆盖该组、值无法准确呈现或完整事实读取停止
- **THEN** 系统 MUST 不投影该组任何源级 enum 常量，并 MUST 保留物理字段、构造器、初始化器及其来源或停止状态

#### Scenario: Existing enum argument forms keep their own proofs
- **WHEN** 输入为已支持的 int literal/int 条件实参、`String...` 字面量数组或普通无实参 enum
- **THEN** 既有完整投影与原子拒绝边界 SHALL 保持，不因单 String 证书改变参数类型或执行次数
