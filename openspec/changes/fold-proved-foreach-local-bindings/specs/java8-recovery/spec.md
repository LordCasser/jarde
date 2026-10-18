## MODIFIED Requirements

### Requirement: Typed List foreach preserves the existing body binding

系统 SHALL 在同一物理方法的完整源码证明已确定 iterable 为精确 `List<E>`、逐元素转换与 E 完全匹配时，将受支持的 Object foreach 与正文首句元素 cast 呈现为 typed E foreach。系统 MUST 保持逐元素检查先于原目标写入和任何用户效果。默认保留已有元素合成名字、原目标声明作用域、正文绑定位置及其余语句；仅在唯一元素绑定、全部局部读写和词法可见性完整证明原目标只属于该循环时，系统 SHALL 将已有原目标名字提升为头变量，删除对应首句和无其它用途的外提空声明，不改变其余引用的变量含义。

#### Scenario: Nested declaration and assignment bindings

- **WHEN** 完整输入包含两层受证 foreach，外层正文首句给循环外空声明变量赋值，内层正文首句声明变量，二者均从唯一元素作相同精确 E 转换
- **THEN** 完整类源码 SHALL 同时呈现两个 E foreach；全部使用范围闭合时，两头分别使用已有外层与内层目标名字且不再保留中间复制或无用空声明。任一绑定缺证时仅保留该绑定的已可靠 typed 头和原声明/赋值，不为另一层的美化删掉它

#### Scenario: Empty iteration and externally observed local

- **WHEN** iterable 为空，或循环前/后可观察原赋值目标，或正文内有同名但不同作用域的变量
- **THEN** 系统 MUST 保持零次迭代、最后成功元素绑定及循环退出后的可观察值和变量解析关系；有初始值、循环外用途或无法区分的同名绑定时 MUST 保留原目标声明及正文写入，不把这些变量重新绑定为头变量。无外部用途的空声明仅能在完整作用域证明后删除

#### Scenario: Typed foreach is the only source-body projection

- **WHEN** 受支持方法的泛型参数与唯一逐元素转换已完整证明，但正文没有其它调用实参转换或包装需要改写
- **THEN** 系统 SHALL 仍完成 typed foreach，并在绑定用途也完整证明时提升已有元素名字；两者均不能依赖无关正文改写偶然触发

#### Scenario: Element conversion fails

- **WHEN** null iterable、null 元素、污染列表中的错误类型元素或 iterator/hasNext/next 抛异常
- **THEN** 完整生成 Java 的异常类型、已完成目标写入、列表状态和事件顺序 SHALL 同原 class；失败转换之前不得写原目标，正文效果不得提前执行，移除中间复制不得改变循环外可见的最后值

#### Scenario: Source or binding evidence is incomplete

- **WHEN** 泛型源码证明未完成、源码元素与物理 cast 不等、源不是受支持的直接参数、首句转换不唯一或位于其它异常范围、或原 next/cast 结果在受证绑定之外还有消费
- **THEN** 系统 MUST 保留已有可靠泛型/循环/正文表示，不猜测元素类型、不吞掉 cast、不发布部分 typed 投影。仅变量用途或可见性证明不足时 MUST 保留已证明的 typed 头和原正文绑定，不能因此降级可靠元素类型

#### Scenario: Evidence scope and transactional stopping

- **WHEN** 相同输入使用 default/all/no-rule-details，或新投影遇到预算、取消或期限停止
- **THEN** 完成产物的完整类/方法投影 SHALL 一致，原始方法恢复文本与其自身来源映射 SHALL 保留；停止 MUST 原样传播，当前事务不得发布半完成的声明删除、头变量替换或正文，方法 map 不得冒称属于投影后的类文本

#### Scenario: Visible declaration and all uses belong to the loop

- **WHEN** 元素来自唯一首句绑定，原目标只有该物理写入，全部读取都位于该循环正文，原声明在真实词法祖先中可见且没有其它同名绑定
- **THEN** 系统 SHALL 使用原目标已有名字呈现头变量，保留正文其它表达式、语句、求值次数及访问对象；有调试名时保留该名，没有调试名时保留已有确定性局部名，不猜原始源码名

#### Scenario: Sibling scopes and reused locals do not authorize a fold

- **WHEN** 相似声明位于另一个 if/else、catch/finally 或 switch 分支，局部槽位在别处复用，有额外写入、循环外读取、未闭合值合流，或存在无法完整区分的捕获/遮蔽
- **THEN** 系统 MUST 不借相同名字、槽号或指令来源把不同词法绑定合并；缺证的可选折叠保持原 typed 头与正文绑定，不删另一个分支的声明

#### Scenario: Removed bindings retain physical origins

- **WHEN** 唯一首句及其无用空声明被提升到已证明的 foreach 头
- **THEN** 投影 SHALL 保留这些被折叠节点的同一物理方法来源；原方法 text 及其 source map 仍逐字保持，只在其自身文本坐标中验证全部物理指令，不伪造投影全文映射
