## ADDED Requirements

### Requirement: Typed List foreach preserves the existing body binding

系统 SHALL 在同一物理方法的完整源码证明已确定 iterable 为精确 `List<E>`、逐元素转换与 E 完全匹配时，将受支持的 Object foreach 与正文首句元素 cast 呈现为 typed E foreach。系统 MUST 保留已有元素合成名字、原目标变量的声明作用域、正文赋值或声明位置及其余语句；逐元素检查不得晚于原目标写入或任何用户效果。

#### Scenario: Nested declaration and assignment bindings

- **WHEN** 完整输入包含两层受证 foreach，外层正文首句给循环外已声明变量赋值，内层正文首句声明变量，二者均从唯一元素作相同精确 E 转换
- **THEN** 完整类源码 SHALL 同时呈现两个 E foreach，正文首句保留原声明或赋值并读取各自已类型化元素；不得为美化头变量而删掉或移动原目标声明

#### Scenario: Empty iteration and externally observed local

- **WHEN** iterable 为空，或循环前/后可观察原赋值目标，或正文内有同名但不同作用域的变量
- **THEN** 系统 MUST 保留原目标在零次迭代、最后一次成功元素绑定及循环退出后的值与变量解析关系；不借头变量类型改写重新绑定原目标

#### Scenario: Typed foreach is the only source-body projection

- **WHEN** 受支持方法的泛型参数与唯一逐元素转换已完整证明，但正文没有其它调用实参转换或包装需要改写
- **THEN** 系统 SHALL 仍完成typed foreach，不能依赖无关正文改写偶然触发类型化循环头

#### Scenario: Element conversion fails

- **WHEN** null iterable、null 元素、污染列表中的错误类型元素或 iterator/hasNext/next 抛异常
- **THEN** 完整生成 Java 的异常类型、已完成目标写入、列表状态和事件顺序 SHALL 同原 class；失败转换之前不得写原目标，正文效果不得提前执行

#### Scenario: Source or binding evidence is incomplete

- **WHEN** 泛型源码证明未完成、源码元素与物理 cast 不等、源不是受支持的直接参数、首句转换不唯一或位于其它异常范围、或原next/cast结果在受证绑定之外还有消费
- **THEN** 系统 MUST 保留已有可靠泛型/循环/正文表示，不猜测元素类型、不吞掉 cast、不发布部分 typed 投影

#### Scenario: Evidence scope and transactional stopping

- **WHEN** 相同输入使用 default/all/no-rule-details，或新投影遇到预算、取消或期限停止
- **THEN** 完成产物的完整类/方法投影 SHALL 一致，原始方法恢复文本与其自身来源映射 SHALL 保留；停止 MUST 原样传播，当前事务不得发布半完成的typed声明或正文，方法 map 不得冒称属于投影后的类文本
