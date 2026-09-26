## ADDED Requirements

### Requirement: 已证明的结构化 finally 受保护正文

在正常与异常出口的清理等价、完成流和异常表范围均已证明时，若受保护正文含可完整恢复的 Java 分支与抛出，系统 SHALL 将其写入单个 `try/finally`。恢复后的完整类 MUST 保持每条路径的原返回值、原异常对象、清理次数、效果顺序和清理异常覆盖优先级。若正文或边界不能完整证明，系统 MUST 保留整段物理来源并拒绝该结构，不得仅因 catch-all 表项存在而推断 `finally`。

#### Scenario: Branch throws or returns before one cleanup

- **WHEN** 受保护正文的一臂先产生效果并抛出异常，另一臂求值并保存返回值，两个出口都流经已证明的清理
- **THEN** 完整 Java 类 SHALL 在异常路径重抛同一对象，在正常路径返回清理前求出的值；每条路径的清理恰执行一次

#### Scenario: Cleanup call overrides either pending completion

- **WHEN** 仅由清理正文中的调用抛出新异常，且先前待完成的是返回或受保护正文的异常
- **THEN** 完整 Java 类 SHALL 抛出该新异常并保留先前效果，清理调用不得因新异常重入而再执行一次

#### Scenario: Protected tail shares a physical block with cleanup

- **WHEN** 正常臂的值保存、正常清理与实际返回位于同一个物理基本块，而异常表仅保护值保存之前的半开区间
- **THEN** 源码 SHALL 把返回写在该正常臂的有效局部作用域内，清理只由 `finally` 写一次；不得漏掉分支、把清理放进 `try` 正文或在分支外读取不可见局部

#### Scenario: Unproved body and edge remain source bearing

- **WHEN** 正文含外部入口、额外异常处理器或未呈现的指令，清理副本不等价，保护范围包含清理自身，返回值或原异常身份不唯一，或预算/取消阻止完成证明
- **THEN** 系统 MUST NOT 输出部分恢复的 `try/finally`；保守结果 SHALL 保留两份清理、分支、返回、重抛与异常表的可追溯来源，并遵守既有停止状态

#### Scenario: Evidence modes and neighboring shapes

- **WHEN** 对已证明正文请求默认或完整证据，或处理既有直线 `finally`、资源、monitor、命名 catch 与显式覆盖型清理反例
- **THEN** 两种证据模式 SHALL 有相同 Java 正文且来源涵盖实际指令；既有正例行为保持，未证明及显式覆盖型反例继续拒绝
