## ADDED Requirements

### Requirement: A proved short-circuit Boolean graph may supply one static call argument

当 Java 8 方法的闭合无环短路测试图将两个精确 `1/0` 生产者汇入唯一栈 Phi，且真实 `invokestatic` 的 Methodref 描述符为 `(Z)V` 并且唯一读取该 Phi 作为唯一参数时，恢复结果 SHALL 发射 Java 8 可重编的惰性布尔表达式与恰好一次目标调用；每条路径的字段值、短路 RHS 次数及目标调用次数和顺序 SHALL 与原 class 一致。任一图入口、异常边、Phi use、参数绑定、目标或预算无法证明时 MUST 原子拒绝并保留全部已解码来源。

#### Scenario: Mixed expression passed to one Boolean sink

- **WHEN** Java 8 编译 `sink((a && b()) || c())`，BCI 1/7/13 测试、16/20 生产 `1/0`，BCI 21 是唯一 `invokestatic sink:(Z)V`
- **THEN** 完整恢复类在八组 `a/bValue/cValue` 下 SHALL 与原 class 有相同 `result`、`bCalls`、`cCalls` 和 `sinkCalls`，且每条路径 `sinkCalls=1`

#### Scenario: A call's signature or parameter binding is not proved

- **WHEN** 调用有接收者、更多或非 `Z` 参数、真实 Methodref 不明、Phi 有第二 use、测试/producer 有外部或异常边，或请求在证明/发射预算处停止
- **THEN** 恢复 MUST 不输出猜测的调用或部分短路表达式，拒绝结果 SHALL 保留测试、producer、消费和后缀的真实来源；已有字段/直接返回正例不受影响
