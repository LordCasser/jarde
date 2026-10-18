## ADDED Requirements

### Requirement: A proved if-arm entry transfer retains its nested-loop origin

已恢复if的非空臂若以前导唯一无条件跳转块进入紧邻的嵌套循环，且完整物理控制流和唯一归属证明该关系，系统 SHALL 在真实if源码span上保留该跳转的派生物理来源。变换 MUST 只增加真实来源，保持Java文本、方法/类身份、物理事实及既有拒绝不变，不把该要求推广为任意goto或所有结构的完整BCI映射保证。

#### Scenario: Original Raung entry edge is traceable

- **WHEN** 请求真实TestLoopRestore2原class或原JAR中 `test([I[I)V` 的来源映射，其十条物理指令包含从if臂进入循环的goto
- **THEN** 已恢复源码 SHALL 保留十条指令的可靠来源，入口goto作为if的派生origin；每个span均为有效UTF8边界并指向精确原方法/类，不制造独立Java跳转语句

#### Scenario: Incomplete or different entry proof adds no guessed origin

- **WHEN** 前导块含其它指令/效果、存在额外或异常出口、目标不是紧邻循环入口、物理身份或控制流证据不足
- **THEN** 本次规则 MUST 不附加猜测来源，保持既有源码与拒绝，不挪用其它块、循环出口或上下文clone的来源

#### Scenario: Evidence selection and real stops retain their contract

- **WHEN** 同一输入分别请求default/all证据，或证明、来源构建、后置映射物化遭遇真实预算/取消停止
- **THEN** 成功请求的Java文本与精确来源 SHALL 一致；停止 MUST 原样传播并保留实际用量，正文提交前不发布半成品，正文提交后映射停止沿用可靠完整节点前缀及显式Partial/None，不改变既有发布协议

#### Scenario: Comparison preserves the upstream evidence boundary

- **WHEN** 验收该原始Raung形态的完整原始/JADX/Jarde输出
- **THEN** 生成源码 SHALL 原样交两编译器，所有失败保留；不执行上游未提供oracle的目标方法，不以编译成功声称运行等价或JADX精确while文本通过，亦不关闭整个CF08或71/612账本
