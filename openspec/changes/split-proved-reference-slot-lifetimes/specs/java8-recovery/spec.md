## ADDED Requirements

### Requirement: Disjoint ordinary reference lifetimes retain separate source variables

无调试局部表的普通方法中，同一非参数、非守卫头部槽先后持有至少两种有明确来源、可拼写的引用类型，且每个读取能唯一归属一个写入段、旧值没有跨段消费者、正常流不能从后段返回前段时，恢复 SHALL 分别呈现这些生命周期的声明、类型和名称。完整源集 SHALL 可以独立 Java 8 重编，验证运行的输出、返回值和副作用顺序 SHALL 与原类一致。引用类型之间的继承关系 SHALL NOT 被当作生命周期证明。

#### Scenario: Array followed by collection without debug information
- **WHEN** 完整类的方法先遍历存于局部槽的 `int[]`，该值已死后同槽存入 `ArrayList` 并调用其方法，且满足上述正常流证明
- **THEN** 输出 SHALL 分别声明数组与集合，后段 SHALL NOT 沿用数组名称或类型，完整类 SHALL 独立重编并以 `-Xverify:all` 运行一致

#### Scenario: Ordinary objects and arrays have the same lifetime rule
- **WHEN** 无调试信息完整类包含可证明的 String→StringBuilder、String→数组、Deque→StringBuilder、List→Map、Map→StringBuilder、StringBuilder→数组或 String→Queue 两段槽复用
- **THEN** 各输入 SHALL 分段并独立重编运行一致，真 javac 8 与 javac 23 的 Java 8 产物 SHALL 按相同证据规则处理

#### Scenario: Unsupported evidence cannot create a new lifetime
- **WHEN** 读取无唯一写入归属、旧值跨段存活、跨段 phi、后段能返回前段、存在异常/call-context 边，或类型仅为未知/null 时
- **THEN** 新规则 SHALL 不发布分段，既有可恢复结果或拒绝及来源 SHALL 保持；SHALL NOT 以 BCI 顺序、局部名或扩大为 Object 来替代证明

#### Scenario: Established naming and allocation paths retain their behavior
- **WHEN** 输入已由不同名 LVT、数组间分段、Ref/Int 或证过的 monitor 分段呈现，或只是同类型重复赋值、参数/receiver、资源头部、同名 LVT
- **THEN** 本次普通引用规则 SHALL 不改变已有源文本及拒绝边界，SHALL NOT 以此宣称同名 LVT 已恢复

#### Scenario: Stop before publication has no partial identity change
- **WHEN** 分段证明遇到预算耗尽或取消
- **THEN** 操作 SHALL 沿既有停止路径报告，SHALL NOT 发布身份已拆分但名称或声明未完成的成功正文；essential 与 all 证据 SHALL 得到同一源文本
