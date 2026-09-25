## ADDED Requirements

### Requirement: Proven array traversal is presented as Java 8 enhanced for

当已恢复循环的数组遍历、元素绑定和所有可观察转移均被完整证明与 Java 8 增强 `for` 等价时，系统 SHALL 呈现 `for (元素类型 元素名 : 数组表达式)`。呈现 SHALL 保持数组表达式求值次数、元素访问顺序与类型、循环体、异常和 `break`／`continue` 的效果；系统 MUST NOT 因源码形态投影删除另有用途的索引或数组操作。等价性不成立或证据不足时，系统 SHALL 保留已可用的普通循环或既有保守引用，MUST NOT 生成看似合法但语义改变的增强 `for`。

#### Scenario: Primitive and reference arrays

- **WHEN** `int[]` 或 `Object[]` 的循环从索引零开始，逐次加一，以同一数组的长度为上界，并在每轮绑定同一数组的当前元素，索引及长度缓存没有其它可观察用途
- **THEN** 完整类 SHALL 输出 Java 8 增强 `for`；重编译后的空数组、多元素数组和数组元素相关调用与原 class 的值、副作用及异常顺序一致

#### Scenario: Container expression runs once

- **WHEN** 数组由有副作用、返回 null 或抛异常的表达式取得，原循环只求值该表达式一次
- **THEN** 增强 `for` 的数组表达式 SHALL 只求值一次；null 与抛错路径 SHALL 保留已发生的调用及其异常类型

#### Scenario: Index remains observable

- **WHEN** 索引在元素读取、循环测试和单次递增以外仍有消费，包括退出后使用或循环体内使用
- **THEN** 系统 MUST NOT 输出会移除该消费的增强 `for`；若该循环已恢复为普通 Java，其重编译执行 SHALL 保持原 class 的结果，否则 SHALL 保留可定位的保守引用与未恢复状态

#### Scenario: Length and element refer to different arrays

- **WHEN** 循环以数组 A 的长度作界，但读取数组 B 的元素
- **THEN** 系统 MUST NOT 将循环折叠为遍历 A 或 B 的增强 `for`；若输出普通 Java，B 较短时 SHALL 在原位置抛数组越界异常，否则 SHALL 保留可定位的保守引用与未恢复状态

#### Scenario: Transfer or element conversion cannot be proved

- **WHEN** `continue`、标签、异常处理边界、元素赋值转换或额外作用使增强 `for` 的执行路径不能被完整证明
- **THEN** 系统 SHALL 保持原有普通循环或保守引用，且输出的来源和拒绝原因 SHALL 可定位相关字节码

### Requirement: Iterable enhanced for requires source-valid type and iterator semantics

系统 SHALL 只在容器表达式可被 Java 8 编译为 `Iterable` 增强 `for`，且 `iterator()`、`hasNext()`、`next()`、元素转换与所有退出边被证明等价时呈现该语法；系统 MUST NOT 仅因方法名相同或推断类型可改写就把手写迭代循环当成增强 `for`。证据不足时 SHALL 保留可执行的迭代循环。

#### Scenario: Iterable with a proved element binding

- **WHEN** 容器的源类型和元素绑定／必要的循环体内显式 cast 可完整证明为合法 Java 8 增强 `for`，其迭代器没有额外消费者、更新或副作用
- **THEN** 呈现的完整类 SHALL 重编译，且原 class 与呈现类在空序列、多元素、null 容器及 `iterator()`／`hasNext()`／`next()` 抛错路径上的值、调用次数和异常顺序一致

#### Scenario: Raw Iterable keeps the element cast

- **WHEN** 容器只能以原始 `Iterable` 源类型呈现，但原循环的 `next()` 结果随后有可证明的元素强制转换
- **THEN** 系统 SHALL 仅在可保留该转换的位置与异常效果时，使用合法的 `Object` 增强 `for` 元素绑定并在循环体保留该转换；MUST NOT 把原始 `Iterable` 的元素直接声明成较窄类型

#### Scenario: Method named iterator without Iterable

- **WHEN** 容器只提供名为 `iterator()` 的方法但其源类型不实现 `Iterable`
- **THEN** 系统 MUST NOT 输出该容器的增强 `for`；保留普通迭代循环或保守引用

#### Scenario: Next is protected by a different handler

- **WHEN** 原循环的 `next()` 位于 `try` 保护范围，而 `hasNext()`／循环测试不在同一保护范围
- **THEN** 系统 MUST NOT 把 `next()` 移到增强 `for` 的隐式取值位置；只有能保持原异常处理效果的普通循环才可作为可执行源码，否则 SHALL 保留可定位的保守引用

#### Scenario: Body effect occurs before next

- **WHEN** 每轮循环在调用 `next()` 之前还执行可观察的调用、写入或可能抛错的操作
- **THEN** 系统 MUST NOT 以增强 `for` 提前调用 `next()`，并 SHALL 保留该操作与取值、转换的原有顺序

#### Scenario: Cast and body effect have distinct failure order

- **WHEN** `next()` 的结果先经过显式元素 cast，随后才有体内副作用，且 cast 可能抛 `ClassCastException`
- **THEN** 任何增强 `for` 投影 SHALL 保持 cast 先于副作用；错误元素路径的调用次数和异常类型 SHALL 与原 class 相同

### Requirement: Enhanced for projection preserves bounded evidence

增强 `for` 投影 SHALL 遵守既有预算、取消、来源与报告契约，且默认／完整来源请求的正文 SHALL 一致。不能完成证明时 MUST NOT 部分隐藏原循环指令或伪造已认领的来源。

#### Scenario: Projection is refused within a budget

- **WHEN** 候选循环的证明超出工作、深度或取消界限
- **THEN** 系统 SHALL 按既有停止或保守输出契约返回，保留必要的原始指令来源，不发布半折叠的增强 `for`

#### Scenario: Source selection changes metadata only

- **WHEN** 同一已证明循环分别请求默认与完整来源，且预算足够
- **THEN** Java 正文 SHALL 相同；完整来源 SHALL 包含被投影的数组捕获、长度、索引、元素读取和更新的真实位置，默认请求无需发布来源表
