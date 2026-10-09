## ADDED Requirements

### Requirement: 子数组初始化器 SHALL 使用已证明的赋值兼容关系

当子数组值是父数组对应元素写入的唯一完整表达式，且真实类型可证明赋值给父组件时，系统 SHALL 恢复完整嵌套Java数组初始化器。系统 MUST 保留每层实际数组类型、每次构造/调用/写入的求值顺序、次数、异常与物理来源；父组件与子数组实际类型不相同 SHALL NOT 单独构成拒绝理由。

#### Scenario: 平台数值和接口子数组

- **WHEN** 完整Java家族把Integer[]/Long[]写入Number[][]，或把ArrayList[]/HashSet[]写入Collection[][]，且唯一消费、效果和实际赋值关系均有证明
- **THEN** 生成完整嵌套initializer，全部生成源隔离重编与验证运行的实际exit/stdout/stderr与原程序一致，子数组和元素保持原顺序且每个生产者只执行原次数

#### Scenario: 选中家族的间接继承关系

- **WHEN** DerivedA经Mid继承Base，DerivedB直接继承Base，完整选中家族把两种子数组写入Base[][]，并在各确切store位置证明对应类型关系
- **THEN** 恢复完整Java正文，保留两条继承链所依赖的事实与各allocation/constructor/store来源，完整家族重编和运行一致

#### Scenario: 缺少继承中间类或错误证明

- **WHEN** 某个store的source/target关系未知、证明锚定其他BCI或类型pair，或选中家族缺少所需Mid头部
- **THEN** 不从类名或另一个元素的成功猜测关系，完整方法保持拒绝并保留真实来源，不把局部initializer或结构记录计为Java成功

#### Scenario: primitive数组关系与运行时类型检查

- **WHEN** 子数组涉及primitive leaf、rank不同或实际不兼容的verifier-valid引用store
- **THEN** 仅接受Java已有数组赋值关系；不把int[]协变成long[]，不删除实际ArrayStoreException检查，不因结构可组合而生成语义不同的成功initializer

#### Scenario: 额外读取或不闭合效果

- **WHEN** child retained value有额外reader、store顺序被改动，或child到parent消费区间含不属于表达式的效果
- **THEN** 保留既有拒绝，不复制child表达式、不重排元素、不发布一半父子组合，不丢失失败区域的生产者与来源

#### Scenario: 预算或取消中止组合

- **WHEN** 嵌套组合的结构证明或正文构建受本次共享预算/取消停止
- **THEN** 准确报告停止原因与位置，不发布未闭合组合或半份Java正文，不绕过深度上限，不把空体或结构记录计为恢复成功
