## ADDED Requirements

### Requirement: A proved pure transfer inside a closed Boolean decision graph preserves its value

Java 8 字节码把闭合的无环布尔测试图中的一条边经无效果前向转接送往共享 1/0 producer，并由唯一同槽 Phi 写入一个静态 `Z` 字段时，恢复器 SHALL 仅在真实物理边、节点归属、测试值、producer 常量与字段消费全部可证明时输出结构化 Java。输出 MUST 在每条路径与原 class 保持相同字段值、惰性 RHS 调用次数和顺序，并给每个已认领的真实指令来源。转接存在额外入口、效果、异常边、回边或不明消费者时 MUST 原子引用完整局部图，不得把可编译的错误条件当作恢复。

#### Scenario: Ternary arm transfers to a shared true producer

- **WHEN** `javac --release 8 -g:none` 编译 `result = (gate ? extra : left) || other || rhs()`，BCI 8 的纯 `goto` 把 `extra=true` 送往 BCI 25 的真 producer，BCI 29 为假 producer，BCI 30 唯一 `putstatic Z`
- **THEN** 结构化完整类的全部 32 组 `gate/extra/left/other/rhsValue` 输出 SHALL 与原 class 的 `result/calls` 逐字相同；BCI 8 和另外 15 个已解码起点 SHALL 在 source map 中可追；JADX 1.5.6 的错误结果不作为验收依据

#### Scenario: A gateway is not physically closed

- **WHEN** 转接块有额外正常前驱、独立效果、可抛异常边或回边，或共享 producer/唯一 Phi 的证明失败
- **THEN** 恢复 MUST 不输出伪字段赋值，MUST 保留全部已证明拥有的已解码来源，并且预算/取消停止时不得发布伪完整方法
