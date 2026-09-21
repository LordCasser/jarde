## ADDED Requirements

### Requirement: A single class has a readable source convenience view

系统 SHALL 提供一个库级单类源码视图操作，接收显式类身份（名字或物理定义）、环境策略与 `Budget`，交付一个类声明、其字段与每个成员方法结果的装配文本，以及逐成员的 typed 结果。CLI SHALL 通过薄适配器提供同名子命令并默认输出文本。该视图是既有按需恢复结果的装配，MUST NOT 引入第二套解码、恢复或成员扫描实现：同一请求内的类读取与成员列举复用现有类视图事实，每个方法体复用现有单方法恢复链路。视图 MUST NOT 声称交付可编译 Java 工程、已解析 imports 或 resources。

#### Scenario: Recover the readable source of one class

- **WHEN** 一个名字唯一绑定到一个物理类定义，且调用方请求其源码视图
- **THEN** 输出包含类修饰符、名字、super/接口与包内成员；有产出的方法给出签名与体文本，成员顺序取自声明序；每个方法保留自己的 content、quality、coverage、execution、diagnostics 与 source map（B11；A13/A16）

#### Scenario: Ambiguous name is not guessed

- **WHEN** 请求的名字绑定到多个物理类定义
- **THEN** 返回候选集合且不执行任何恢复，CLI 以既有的歧义退出状态结束（B11；A07）

### Requirement: The class source view states what it could not recover

装配文本 SHALL 用稳定、显式的标记区分：无 Body 的声明（abstract/native）、未产出或拒绝的方法、只解释（explanation_only）的方法，以及带停止的诊断。MUST NOT 用空方法体或省略成员来掩盖未产出；MUST NOT 因单个方法失败而把顶层 execution 发布为 Complete。汇总 SHALL 分别计数已产出、仅解释、未产出与失败成员。

#### Scenario: A refused method inside a readable class

- **WHEN** 类中某些方法被拒绝而其余方法正常产出
- **THEN** 文本保留全部成员的声明位置并在相应位置标记拒绝原因码；汇总把该成员计入拒绝而非已产出，顶层 execution 至少 Partial（B11；A13）

#### Scenario: A class whose header cannot be read

- **WHEN** 目标类的声明或成员表无法读取
- **THEN** 操作返回带物理位置的失败或非 Complete 结果并保留诊断，MUST NOT 返回空类或伪造完整输出（B11；A07/A16）

### Requirement: The class source view is deterministic and bounded

同一输入、同一配置与同一 `Budget` 形状下，装配文本 MUST 逐字节可重复（不得包含耗时或调度字段）。视图 SHALL 受既有维度、`Limits` 与取消约束；一次请求的读取次数 SHALL 与现有类视图加逐成员恢复的既有形状一致，MUST NOT 为每个成员重新读取同一个类 Header 或重新列举成员表。请求执行期间观察到的取消 MUST 停止后续成员处理并保留已确认的成员结果。

#### Scenario: The view repeats byte for byte

- **WHEN** 同一 fixture 与同一环境配置被请求两次
- **THEN** 两次装配文本逐字节相同，成员身份与顺序相同，usage 单独记录（B11；A18）

#### Scenario: Cancellation keeps the delivered members

- **WHEN** 视图请求在若干成员完成后被取消
- **THEN** 已完成成员仍出现在结果中，未处理成员不被标记为产出，顶层 execution 为 Cancelled 且 usage 不为零（B11；A14）
