## ADDED Requirements

### Requirement: Proved prefixed named catch retries preserve normal completion

Java恢复 SHALL 支持已证明闭合的普通具名catch重试：每次尝试前有不受该catch保护的效果前缀，handler的一臂更新状态后回到精确尝试入口，另一臂经有限条件汇合正常退出。恢复 SHALL 保留每次效果、异常作用范围、重试及正常完成顺序，每个物理块 SHALL 仅有一个结构位置拥有。

#### Scenario: Complete original with a prefixed retry is recovered

- **WHEN** 完整原件每次先记录尝试，再执行受保护的流操作，首次失败更新标志并重试，后续失败执行条件分支后正常退出，且具名保护区与全部路径均获证
- **THEN** 目标方法 SHALL 形成可重编的结构化完整类；已声明空路径输入的builder、stdout、异常及实际check SHALL 与原件同runtime一致，不得递归重复发布同一个try正文

#### Scenario: Prefix and handler failures are outside the protected attempt

- **WHEN** 独立有限控制实际触发正常完成、一次失败后成功、再次失败后退出，以及前缀或catch内部效果抛出异常
- **THEN** 前缀 SHALL 每次尝试恰好执行一次，状态更新 SHALL 只出现在重试路径，正常退出后效果 SHALL 恰好一次；前缀和handler自身抛出的异常 MUST NOT 被扩大的catch再次接管

#### Scenario: A normal catch diamond remains complete

- **WHEN** 非重试分支包含有可观察效果的两臂条件，随后在正常出口前汇合
- **THEN** 两臂各自的效果、条件求值及汇合后效果 SHALL 保留；未选择臂和重试后的正常尾部 MUST NOT 被执行或复制

#### Scenario: Foreign ownership or exception rows refuse the candidate

- **WHEN** 保护区/handler/捕获类型发生漂移，存在共享handler、额外异常行、外来入口/出口、未解释转移、未闭合分支或未知克隆路径
- **THEN** 本能力 SHALL 拒绝该候选并保留可定位降级，MUST NOT 因跳过重复块或把异常边改写为普通边而声明完整恢复

#### Scenario: Evidence and stop reporting remain atomic

- **WHEN** 同一输入以default/all恢复，或候选证明与组装遭遇预算耗尽、预取消或晚期停止
- **THEN** 正文、质量、身份及物理来源 SHALL 不因证据选择改变；停止 SHALL 保留准确原因与已消耗用量，MUST NOT 留下半份循环、重复owner或在取消后继续试探发布
