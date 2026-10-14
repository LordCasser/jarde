# 提案：恢复前向条件分支携带的栈值

## Why

`recover-committed-local-multireads` 的门控实验已证伪其核心前提：已提交局部多读不会进入目标计数门；NI 类问题由跨块拼接回退引起，BI 循环例也不能通过宽泛 phi 过滤安全解决。该 change 的原实施仍阻断，不能据此扩大 `prepare_deferred_bindings` 的消费者规则。

admission 的完整 13 方法类曾在包含 `effects += value == null ? 0 : 1` 的条件控制流中被拒；独立 final-matrix-v1 审计记录了 admission 的 Jarde 整类编译失败。root 已从真实 classfile、CFG 与 SSA 逐项确认：同值退休栈 phi 的两条输入 use 与一个真实 reader 一起进入计数，导致原有 consumer gate 拒绝。有限前向证明已接入并通过 focused、完整类矩阵与时序对照；全工作区验证和最终交付仍开放，不能将本 change 标为完成。

## What Changes

依据 root 对真实 admission class 的 SSA/字节码诊断，实现一个极窄的恢复入口：仅处理物理 SSA 类型为 `Value::Int` 的整数携带值及已有 Folded 条件值恢复所证明的单个前向 diamond；只有同一 `Stack(slot)` 上被精确证明退休的同值 phi、恰有两个 phi Value 输入、且恰有一个真实物理 reader 时，才允许忽略这两条等值 phi 输入记录对 reader census 的重复计数。

这不是一般的 phi-use 过滤，也不放宽 consumer 总数、执行顺序、循环、handler、alias、表达式拼接或活 phi 规则。若真实诊断不能证明上述结构，或实现必须改变更宽的通用门，则先重新切片提案，不实施近似修补。

## Scope and Invariants

- 对象/StringBuilder receiver 继续由既有 construction/concat 完整链证明与原拒绝路径处理；不准入本整数 carry。
- 候选在完整 final-consumer/join/dependency 检查成功前保持 provisional，不改变全局 consumer census。

- 不改 admission 或 baseline 的 Java fixture、Runner、manifest 和基线源字节；不删减完整类的 13 个物理方法。
- 不硬编码方法名、BCI、slot 数字或编译器偶然命名；候选通过 classfile、控制流、SSA Value/Slot 与真实指令 reader 身份绑定。
- boolean-loop / BI 控制的当前冻结恢复或拒绝状态、正文、来源与循环行为保持逐字不变；不以历史拒绝快照覆盖当前事实。
- 候选仅限单一向前 diamond；禁止 handler 覆盖、回边/循环、额外 reader、不同值输入、live/mixed phi、alias 展开或多阶段 phi 链。
- 所有 source-map/origin、表达式求值次序、budget/cancel 原子提交行为继续受检。

## Validation

完成后按既有 frozen matrix 原协议重跑：双 JDK、debug/none、default/all 的完整 16 个 Jarde cells；原 baseline 45 行与 admission 33 行 oracle；保留编译器/JADX 实际成功或失败，不把失败改判为成功。还需做前向副作用顺序对照、BI 当前冻结正文与来源逐字不变、真实 budget/cancel 中断测试。全量 acceptance 必须使用完整 13 方法 admission 类及原 Runner。

root 已完成实际 SSA 诊断与最小范围裁定（见 design 的实测节）。生产接入、证明测试、完整 16 个 Jarde cells、45/33 行 oracle、前向时序及 BI 当前状态对照均已实测通过；全工作区验证、最终独立审查和自身 CI/clean-main 交付仍待完成。

## Capabilities

### Modified Capabilities
- `java8-recovery`：当且仅当既有证明足以唯一界定同值栈 phi 的有限前向使用时，恢复对应的单 reader 条件表达式并保留物理执行顺序；具体行为由 delta spec 约束。
