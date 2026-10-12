## ADDED Requirements

### Requirement: Debug names preserve proven distinct reference lifetimes

当一个非参数、非资源局部槽有不同确切引用类型的多段生命周期，定义、现有读取、引用别名存活和正常控制流均已证明彼此独立，而且调试局部记录与这些身份一一对应时，恢复 MUST 保留不同变量身份及各自引用类型。同名调试记录 MUST NOT 把身份合并成导致 Java 类型冲突的一个变量；名称冲突允许使用现有确定性后缀。初始化写入可位于调试范围开始之前，其归属 MUST 来自值所有权，MUST NOT 按最近偏移猜测。 对没有局部读取的身份，MUST 另以唯一已证明的真实Store、下一decoded instruction与row.start严格相等、row包含于该身份范围证明初始化邻接；MUST NOT 要求编译器生成多余aload，也MUST NOT 仅按row排序为无读取身份附名。

记录空缺、重叠、额外或不能一一匹配、读取所有权不明、旧引用跨越下一次定义、未支持控制流或类型证据不足时，恢复 MUST 沿既有保守路径处理，MUST NOT 仅凭名称或调试表宣称拆分成功。原始检查转换、raw 别名、调用目标、异常、集合状态和副作用位置 MUST 保留。没有调试信息时 MUST 保持已有身份证明和确定性名称行为。

#### Scenario: Equal debug names belong to different loop scopes

- **WHEN** 两个顺序循环复用同一个局部槽与调试名称，分别承载 Integer 和 String，所有读取及定义满足独立生命周期证明
- **THEN** 恢复 MUST 为两段生命周期发射类型正确的独立变量，MUST NOT 把 String 赋给 Integer；两个名称允许确定性后缀，完整输入中的其他方法必须保留

#### Scenario: Raw alias heap pollution retains its observable exception

- **WHEN** 通过 raw List 别名向 List<String> 写入 Integer，随后 String 检查转换抛出 ClassCastException
- **THEN** 恢复 MUST 保留 raw 别名和真实转换，MUST NOT 为泛型语法删除别名或转换；完整生成类能编译后，其异常、集合内容和副作用计数 MUST 同原类

#### Scenario: Ambiguous debug evidence cannot override value ownership

- **WHEN** 调试记录与定义身份无法一一匹配，或者旧引用经过 load/store/dup/checkcast/phi 跨越下一次定义边界
- **THEN** 恢复 MUST 拒绝该新拆分决定，保留既有可靠处理与物理来源；MUST NOT 用调试偏移覆盖实际值流

#### Scenario: Evidence selection and interruption preserve one decision

- **WHEN** default、all 或不选择调试证据的请求恢复相同方法，或者新增记录匹配工作遇到预算耗尽或取消
- **THEN** 完整请求 MUST 得到相同正文和物理身份；中止 MUST 发布既有停止，无部分变量身份、名称或来源

#### Scenario: Independent outer try refusal remains visible

- **WHEN** 同一完整类中的外层 try/foreach 方法仍未被结构化恢复
- **THEN** 对局部生命周期的改善 MUST NOT 删除或修补该方法，完整类编译失败和未运行状态必须如实记录；MUST NOT 因一个方法的类型错误消失宣称整个语法单元完成

#### Scenario: An unused loop binding still has a distinct reference identity

- **WHEN** 第二个顺序foreach的String变量只有初始化astore而没有aload读取，其SSA身份、alias存活和CFG均已证明，且唯一Store的下一decoded instruction恰为对应LVT row起点、row在身份范围内
- **THEN** 恢复 MUST 保留该身份及String初始化/checkcast效果，允许现有确定性名称后缀；非邻接row、跨身份范围或多个初始化owner MUST 拒绝新增附名，不改变无LVT行为
