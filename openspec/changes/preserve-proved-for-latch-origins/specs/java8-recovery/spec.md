## ADDED Requirements

### Requirement: A proved for latch retains its physical source

恢复 SHALL 为已证明更新和唯一回跳的 for 循环保留隐式末尾 transfer 的准确物理 BCI 与方法身份，并将它作为整个循环语句的派生来源。投影为 for 或把循环组合到单臂分支 MUST NOT 丢失该来源、改变已有正文或放宽结构证明。来源 MUST 由物理控制流与更新证据决定，MUST NOT 通过文本、指令范围填充或猜测缺失 BCI 生成。

#### Scenario: A for projected inside one branch retains its latch

- **WHEN** 已证明单臂分支内的计数循环具有独占更新块，末尾 goto 回到同一自然循环 header，且生成 for 语句
- **THEN** goto 的物理 BCI SHALL 映射到完整 for 语句范围及正确物理方法；PlainOneArmLoops 的 prefixWhile、takenArm、loopAndTail 在 Java 8/23 的 default/all 输出均 MUST 包含 BCI 20，已有所有来源保持有效

#### Scenario: Source retention does not change the executable class

- **WHEN** 双 JDK 的原形与普通单臂对照分别以 default/all 生成完整类
- **THEN** 八份 Jarde 完整源码 SHALL 原样重编且 exit/stdout/stderr 与原 class 逐字一致；正文与 source map 的 profile 恒同性、每个物理成员身份及全部 BCI SHALL 独立验证，运行成功 MUST NOT 代替来源完整性

#### Scenario: A transfer without the exact proof is not invented

- **WHEN** 回跳不是唯一 latch、更新块或更新 BCI 不匹配、terminal 并非准确回到该 header 的 goto/goto_w，或存在不被证明允许的其它边
- **THEN** 恢复 MUST NOT 为其补造循环来源或放宽既有结构接受条件；原有可靠表示、拒绝范围和物理来源 SHALL 保留

#### Scenario: Existing loop controls retain their own origins

- **WHEN** 普通 while、iterator 原形与相邻 CF07 对照被重新恢复
- **THEN** 已有准确来源 SHALL 保持有效，iterator goto@42 SHALL 保持完整；本修复 MUST NOT 用 for 来源规则填补独立 if 汇合或嵌套 else 缺口

#### Scenario: A stopped source proof publishes no partial artifact

- **WHEN** 来源证明达到本 run 的预算、取消或递归停止条件
- **THEN** 恢复 SHALL 传播既有停止，MUST NOT 继续生成或发表部分正文、部分来源，MUST NOT 新增不计费扫描
