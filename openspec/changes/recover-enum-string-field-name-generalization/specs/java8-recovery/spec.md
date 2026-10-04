## ADDED Requirements

### Requirement: String 实参枚举常量体投影不得依赖字段名

系统 SHALL 在证明带一个 String 源实参的枚举常量体时，以**构造器字节码中 `putfield` 目标的已证字段名**为唯一权威来源，不得要求该名字等于任何固定字面量；发射的构造器文本 SHALL 用该已证字段名拼写赋值目标。已证字段名不可从证明结果取得、被该构造器写入的 String 字段不唯一、或该字段的既有属性检查（描述符 `Ljava/lang/String;`、owner 为本枚举定义、非 static/synthetic/隐式枚举成员、`ACC_PRIVATE`、有 `declaration`、无 markers）任一不成立时，系统 SHALL 保持物理类文本并记录拒绝原因，不得发射引用未经证明字段名的文本。

#### Scenario: 字段名非 `op` 的 String 实参枚举常量体可恢复

- **WHEN** 一个 Java 8 枚举有两个常量、各带一个可无损拼写的 ASCII String 源实参与常量专属体，其承载该实参的私有字段名为 `op` 以外的名字（如 `t`、`value`、`label`），完整源集经 `javac --release 8` 与 `java -Xverify:all`
- **THEN** 常量以源级常量列表形式呈现（`TIMES("*") { … }, DIVIDE("/") { … };`），构造器文本以已证字段名拼写赋值目标（如 `this.t = arg0;`），完整源集编译通过且运行结果与原 class 一致

#### Scenario: 字段名为 `op` 的既有锚呈现逐字节不变

- **WHEN** 冻结锚 `TestEnums2a/DoubleOperations`（字段名恰为 `op`）经同一投影
- **THEN** 其呈现与 `recover-proved-string-arg-enum-constant-bodies` 验收时的转录**逐字节相同**——该锚同时是"名字恰为 `op`"与"名字由字节码证明"两种实现的共同正例，故本片不得改变其输出

#### Scenario: 被写入的 String 字段不唯一时拒绝

- **WHEN** 枚举有两个私有 String 字段且构造器 `putfield` 写入的目标无法唯一确定（或该构造器写入多个 String 字段）
- **THEN** 保持物理类源码文本并记录拒绝原因，不产出半投影

#### Scenario: 已证字段名无法取得时拒绝

- **WHEN** 构造器 BCI 序列中 `putfield` 的目标字段名不在证明结果内可得（例如构造器 Code 不完整或该指令非 `putfield`）
- **THEN** 保持物理类源码文本并记录拒绝原因，**不得**回退到任何固定字面量字段名
