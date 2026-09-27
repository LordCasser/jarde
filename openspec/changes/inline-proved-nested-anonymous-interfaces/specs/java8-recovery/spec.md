## ADDED Requirements

### Requirement: Proved two-level nested anonymous interface chain

类源码视图 SHALL 仅在同一次恢复证据证明一个精确两节点匿名接口链时，才将根的直接返回匿名接口实例及其一个实现方法中直接返回的匿名接口实例组合成嵌套 `new Interface() { ... }` 表达式。两节点的分配、构造、接口合同、物理 enclosing 关系、完整方法正文及所选输入范围内的身份引用 MUST 闭合；内层 child 的唯一合成 immediate-parent 捕获字段 MUST 只由对应构造器写入且不得被正文读取。呈现 MUST 保持 Java 8 可编译性和原有行为，物理类与方法仍可独立查询。任一额外分配/身份使用、其他捕获或初始化效果、证据不完整、并存源码投影冲突、预算停止或取消 MUST 保留完整物理表示，不得发布半份匿名树。

#### Scenario: One nested direct-return interface allocation

- **WHEN** 静态根方法直接返回唯一匿名 `Factory`，其完整 `make()` 实现直接返回唯一匿名 `Action`，两接口是顶级可访问类型，`run()` 只更新根类静态计数器，物理分配与 enclosing 关系精确匹配，内层 immediate-parent 合成字段只在构造器写入，且所选范围没有其他身份引用
- **THEN** 根源码呈现 `new Factory() { ... return new Action() { ... } }`，不引用物理匿名类二进制名；原源码、固定 JADX 与 Jarde 完整源码均可由 Java 8 编译并在 `-Xverify:all` 下执行出相同结果

#### Scenario: A duplicate site or external identity reference exists

- **WHEN** 任一匿名物理节点存在第二个分配 BCI、额外外部构造/身份引用，或完整 XRef 范围不能排除这些使用
- **THEN** 根类不得投影任一节点；物理类使用点、成员与诊断作为一个完整结果保留

#### Scenario: The lexical capture is read or exceeds the one-edge relation

- **WHEN** 内层 child 读取 immediate-parent 捕获字段、捕获另一个值、含额外实例字段/构造效果，或 nesting/enclosing method/接口签名不一致
- **THEN** 双层内联被拒绝，完整物理源码保持不变；不得仅因 `$` 类名形状或相似构造器字节码接受关系

#### Scenario: A family member or output step is incomplete

- **WHEN** 任一方法 AST/Code、类级引用扫描、接口声明、所需源码族闭合不完整，或共享预算/取消在关系证明及输出期间停止
- **THEN** 结果不含部分内联文本，执行平面保留实际停止原因，物理方法和类报告继续按既有契约呈现
