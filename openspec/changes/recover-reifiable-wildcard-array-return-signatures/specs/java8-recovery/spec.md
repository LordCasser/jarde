## ADDED Requirements

### Requirement: 无界wildcard数组返回声明 SHALL 使用同次完整正文证明

对无形式参数、完整单return数组创建正文，系统 SHALL 在原始Signature具有可具体化的全无界wildcard叶类型且数组rank/擦除与实际创建完全一致时保留参数化返回声明。系统 MUST 保留原始数组创建、initializer、调用/构造/写入次序和次数、物理来源；声明投影 SHALL NOT 改写正文以制造兼容。

#### Scenario: 完整集合二维数组

- **WHEN** 原始方法返回`Collection<?>[][]`，正文完整创建实际`Collection[][]`，元素是已证明兼容的`ArrayList[]`与`HashSet[]`，无形式参数
- **THEN** 输出完整泛型返回声明，无该Signature拒绝标记；所有原始成员的全部生成源码在两真实JDK隔离重编并验证运行，exit/stdout/stderr与原始家族一致

#### Scenario: 非可具体化或类型不符

- **WHEN** 原始Signature含具体类型实参、有界wildcard、类型变量、不同rank/叶擦除，或存在本片未证明的泛型形式参数/throws
- **THEN** 不凭数组擦除兼容猜测泛型声明，保留明确拒绝；正文、物理Signature和原始成员不被删除

#### Scenario: 缺失完整返回证明

- **WHEN** 数组创建并非完整单return正文，SSA/物理return或来源不同，initializer拒绝、额外效果未闭合或计划停止
- **THEN** 不生成成功参数化声明，不借相同文本、别的方法或局部结构记录代替同次完整证明

#### Scenario: 预算和取消

- **WHEN** 本次返回候选证明或声明投影受共享预算/取消中止
- **THEN** 保留实际停止原因/位置，不发布半份泛型声明；不把擦除声明或完整运行结果计为泛型恢复成功
