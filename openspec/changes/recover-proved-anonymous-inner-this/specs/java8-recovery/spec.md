## ADDED Requirements

### Requirement: Proved anonymous enclosing-instance receiver

当所选 Java 8 class-source 单元包含一个已证明的匿名类分配点，匿名类的 `EnclosingMethod` 与词法外围类及该物理方法一致，唯一的合成外围实例字段和匿名构造器参数可精确对应，且所有该字段读取都位于已闭合的匿名类方法体内时，系统 SHALL 在该分配点以匿名类源码呈现该类，并将已证明的外围实例读取写为合格的 `Inner.this`。输出 SHALL 保留外围实例身份、读取/写入顺序及方法行为；经证明的 `this$0` 字段和赋值 SHALL 只在完整源码投影中省略，不得通过把物理字段写入移到 `super()` 后来伪造 Java 构造器。构造器身份、分配点闭包、源码正文或预算证据不完整时，系统 MUST 保留物理类表示，不得发布部分匿名类投影。

#### Scenario: One anonymous class reads its lexical enclosing instance

- **WHEN** `Inner.make()` 唯一创建一个匿名 `Runnable`，其方法读取 `Inner.this` 并更新 `Inner.this.f`，匿名类只捕获外围 `Inner` 实例且完整源码关系可证明
- **THEN** 根源码在该分配点包含匿名 `Runnable` 类体和 `Inner.this` 访问；完整源码以 Java 8 重编并在验证模式运行，输出与冻结原 class 一致，且匿名 synthetic 字段和构造器脚手架不出现在源码中

#### Scenario: Anonymous enclosing relation is absent or ambiguous

- **WHEN** `EnclosingMethod` 不匹配、外围字段/构造器映射不唯一、存在第二个分配点、额外捕获字段或范围外物理使用，或所需扫描因预算/取消未完成
- **THEN** 系统不内联匿名类、不输出未经证明的 `Inner.this`，保留可查询的物理类报告及原停止/降级事实，且不提交部分根源码投影

#### Scenario: A local variable is also captured

- **WHEN** 匿名类除外围实例以外还捕获局部变量，且这些物理参数/字段不属于本要求所证明的唯一外围实例字段
- **THEN** 系统 MUST 保留物理类表示；不得把局部捕获误分类为外围接收者或删除其值

#### Scenario: The physical capture write precedes the superclass call

- **WHEN** 匿名构造器先写入 `this$0` 再调用 `Object.<init>`，且该构件已由成功的匿名类源码投影完整隐藏
- **THEN** 完整源码 MUST 编译为 Java 8；实现 MUST NOT 将该物理写入重排到 `super()` 之后，也不得在投影不完整时只隐藏该写入
