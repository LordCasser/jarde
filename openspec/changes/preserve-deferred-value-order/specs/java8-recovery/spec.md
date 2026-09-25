## ADDED Requirements

### Requirement: Deferred values retain their original evaluation order

系统 SHALL 在呈现可恢复值时保持原生产者、独立语句和消费者的执行顺序、一次求值和异常优先级。存在消费者本身 MUST NOT 被视为允许跨越独立语句重算该值的证明。

#### Scenario: A construction retains its place before an independent call

- **WHEN** 已支持的直线字节码先构造对象，再执行独立调用，最后消费该对象
- **THEN** 实际恢复的完整正文 SHALL 可重编译并保留一次构造、对象身份及原执行轨迹；构造器先抛错时后续调用不发生，后续调用抛错时构造已发生

#### Scenario: Calls and mutable reads preserve already computed values

- **WHEN** 调用结果、字段读取或数组读取在独立调用/写入之前已经求得，之后才返回或传递，类型和作用域均可表达
- **THEN** 系统 SHALL 保存原求得值而不是在独立语句之后重新调用或重新读取；完整类执行的值、计数与顺序和原class一致

#### Scenario: Checks keep their original exception priority

- **WHEN** cast、数组读取/长度/分配、实例字段访问、整数除法/取余或构造可能在中间独立语句前抛出异常
- **THEN** 恢复代码 SHALL 在相同顺序执行该检查或构造，不先执行原本未发生的后续效果，也不以该效果的异常替代原异常

#### Scenario: Nested expression evaluation remains readable and ordered

- **WHEN** 多个生产者都属于同一个已有支持的表达式，Java操作数顺序与原字节码相同，没有需要跨过的独立语句
- **THEN** 系统 SHALL 保留现有内联表达式和一次求值，不无条件给每个操作数增加局部声明

#### Scenario: Saved values respect lexical scopes and execution counts

- **WHEN** 需要保存的值出现在分支内直线区域或已有支持的测试前缀
- **THEN** 保存及消费 SHALL 留在原执行路径，局部名字在消费处可见且不与已有名字冲突；不能把循环内每轮求值移到外部

#### Scenario: Unproved regions do not publish reordered normal statements

- **WHEN** 类型、词法可见性、循环/异常范围或值合并不能按当前规则证明
- **THEN** 系统 SHALL 明确引用必要生产者与失败消费者，保持已呈现独立语句可见，不发布错序或引用未声明名字的正常表达式

### Requirement: Saved value presentation remains bounded and source faithful

保存值的决策、声明、引用和证据 SHALL 遵循既有预算、取消和产物提交契约，不伪造原始局部槽或物理指令。

#### Scenario: Source evidence identifies both producer and consumer

- **WHEN** 同一方法以默认及完整证据请求恢复
- **THEN** 正文 SHALL 逐字相同，默认不构建来源表，完整来源关联真实生产者/消费者BCI与成员；合成名字不冒称debug名称

#### Scenario: An incomplete binding does not escape publication

- **WHEN** 初始化表达式、类型证明、声明计费或产物提交失败
- **THEN** 系统 MUST NOT 发布引用未声明名字的语句、重复生产者效果或未支付的文本/来源；停止状态与已提交正文遵循原契约

#### Scenario: Long dependency chains remain bounded

- **WHEN** 值依赖链、候选绑定或名称冲突数量达到预算/深度限制，或收到取消
- **THEN** 系统 SHALL 有界停止，不无限重扫、递归溢出或把未完成顺序证明当作安全内联
