## ADDED Requirements

### Requirement: Recovery recursion is bounded and a stop is published

恢复调用 MUST 返回 `RecoveryReport`，MUST NOT 因递归耗尽进程栈而让进程以 signal 终止。恢复层 SHALL 只在能证明递归会完成时继续加深；每次递归下降前 MUST 检查一个显式的深度界，界内递归照常完成，超过界 MUST 在继续加深之前结束为已发布停止：`outcome = Stopped`、非 `Complete` 的 `execution`（Partial 或 Cancelled）、以及点名停止原因与位置的诊断，全部复用既有停止词汇与平面。停止 MUST 保留本次运行的真实 usage 与停止位置，MUST NOT 提交部分构建的文本/段表（`content = not_produced`，与既有停止语义一致），MUST NOT 以空产物冒充成功，也 MUST NOT 只记录日志后返回成功。结构性平面（representation/quality/content/execution）MUST NOT 被当作本要求的证据：本要求的证据是入口返回的报告与其中的停止平面。

#### Scenario: A request that used to abort now answers

- **WHEN** 对固定复现（默认预算、公开入口、standalone class）发起恢复，且修正前同一输入使进程以 `stack overflow, aborting` 结束
- **THEN** 调用返回报告：`outcome` 为 Stopped、`execution` 非 Complete、诊断点名递归界与位置；CLI 以既有「执行未完整」状态（exit 4）退出且 stdout 为报告。MUST NOT 出现 signal 终止、空 stdout 或无报告的失败（验收 A13、A14）

#### Scenario: The bound is checked before the descent

- **WHEN** 下一次递归会超过显式界
- **THEN** 在该次递归开始之前结束并发布停止，位置与原因可定位；不能在递归内栈耗尽后再补救，也不能先写出部分产物再撤回

#### Scenario: Re-entering a state inside the bound

- **WHEN** 递归在界内重新进入本次运行已经访问过的状态（同一 SSA 值、同一块或同一表达式节点的重复进入），说明该递归不能自行完成
- **THEN** 结果 MUST 是已发布停止，或该层能把它局部化时按既有 refusal 语义处理：被拒区域保留 bytecode 与 origin、不得标记结构化，其余工作如实报告；两种结果都 MUST NOT 继续加深直到栈溢出，诊断 MUST 说明是重新进入而不是输入过深（验收 A13）

#### Scenario: In-bound recovery is unchanged

- **WHEN** 已提交语料与受控 fixture 的递归在界内完成
- **THEN** 结果与修正前逐字段一致（仅 elapsed_millis 可不同），不因新增界升级拒绝、降级 quality、改变 content 或改变停止/取消语义（验收 A13）

#### Scenario: The stop is the existing stop shape

- **WHEN** 一个运行因超界停止
- **THEN** 其 `execution`、诊断形态、`content = not_produced` 与 text/段表为空的性质与其他既有停止一致（仅 code 与位置不同）；不得新增 `StopReason` 变体、预算维度或报告平面
