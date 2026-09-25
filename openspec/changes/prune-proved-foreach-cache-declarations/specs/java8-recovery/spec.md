## ADDED Requirements

### Requirement: Proved array enhanced for omits consumed cache declarations

当完整类源码把已证明的计数数组循环投影为 Java 8 增强 `for`，且原长度缓存和归纳下标的所有用途都已由该投影认领时，源码 SHALL 不再保留这两个变量的独立无初值声明。投影 MUST 保持原有执行行为、成员身份与被折叠指令的来源 BCI。

#### Scenario: Direct and safely wrapped element read

- **WHEN** 数组循环已满足现有直接绑定或安全包装读取增强 `for` 的全部证明，长度缓存和归纳下标仅服务于该循环
- **THEN** 完整类源码 SHALL 写增强 `for`，且不含仅属于这两个缓存的前置空声明；原 class 与重编后的类在值、调用次数、副作用和异常类别上 SHALL 相同，来源仍能定位原长度计算、初值与更新 BCI

#### Scenario: Later use shares a JVM slot

- **WHEN** 增强 `for` 之后另一词法变量复用原缓存或下标的 JVM slot，并在后继代码中被读取
- **THEN** 清理 SHALL 仅在同次局部身份计划能区分前后变量时删除缓存声明；后继声明及用途 MUST 保留；若身份无法区分，SHALL 保持现有投影结果，不得按 slot/name 删除。这一场景不要求本 change 修复基线已有的同槽异类型源码错误

### Requirement: Cache declaration cleanup refuses unproved deletion

声明清理 SHALL 仅随相同的增强 `for` 成功证明原子发生；仅有同名文本、同一 slot 或看似未使用不足以删除。投影未成立、用途逃逸、来源不能合并或预算/取消停止时 MUST 保留原声明与既有循环/拒绝结果，MUST NOT 发布半个清理后的正文。

#### Scenario: Counted loop is not eligible

- **WHEN** 下标在循环体或循环后逃逸、数组身份不一致、元素读取顺序不安全，或任何已有增强 `for` 证明失败
- **THEN** 类源码 SHALL 保留原计数循环与其仍需的声明，不得仅因相似变量名删除

#### Scenario: Evidence and stop behavior

- **WHEN** 对同一已证明循环分别请求 essential/all 证据，或在清理证明/输出期间耗尽预算或取消
- **THEN** 两个成功请求的 Java 正文 SHALL 相同且来源覆盖被折叠 BCI；停止请求 SHALL 沿用现有停止契约，不得发布只删一项缓存声明的部分结果
