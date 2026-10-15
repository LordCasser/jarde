# Proposal

## Why

接口 `super` 证明的预算停止会被门面转换为空证明，随后真实公共恢复入口交付 `Fallback/ExplanationOnly`，却声明 `execution=Complete`、`stop=None`。私有组合上的确定性复现已经给出宽预算 1 passed 与 `dependency_depth=0` 的行为断言 1 failed；需要让原停止穿过证明、恢复报告和 presentation 边界，兑现既有接口源码合法性契约。

## What Changes

- 接口 `super` 证明遭遇预算、取消或期限停止后，保留原原因及可用位置，返回既有恢复停止报告；保留本次 raw analysis，停止之后不再启动正文恢复或证据物化。
- 将执行停止与完整证明后的普通保守拒绝分开：缺失、歧义、抽象遮蔽或已证冗余继续按既有来源契约拒绝，不被改成预算停止。
- 用真实公共入口的深度预算反例、宽预算对照、证明收费中取消、期限及早停止原因控制验收；保留原合法调用、拒绝来源与完整生成源对照。
- 保存可接手的最小复现、关键原始流和来源 manifest。当前只有反例证据，尚无修复；实现、最终产品源码门禁及产品自身 CI 均待后续执行。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：明确接口 `super` 证明的执行停止必须在恢复报告与 presentation 可见，保留预算原因、raw analysis 和未提交产物语义，不能用正常 fallback 隐藏停止。

## Impact

预期涉及 `src/facade.rs` 的接口证明交接、`jarde-java` 的最小停止报告入口与必要测试。复用现有 `Error`、`StopReason`、执行平面及停止构造；不新增依赖、通用 pipeline、全局 sticky Budget 或类型/层级服务。设计须先核对非 counted 的 `DependencyDepth` 与期限如何无损映射，不能把合法公共请求简单 `?` 成无恢复报告的裸错误。

前提是既有物理绑定、只读 IR 交接和接口源码合法性证明链；P0–P5 归档及 primary `644882d` 的 CI 不构成本修复的成功证据。真实复现基线为私有 HEAD `f014028…` 上的 13 文件候选与 build `b973da39…`，不是 primary 644 的公共入口实跑；详见 `results/repro-v1/README.md`。

本片不修复 snapshot/direct-parent/reference 证明包装、class-view、CLI 最终文档共用 output budget 或已立项 emitter 四文件修复，不重开它们的证据，也不向当前四工程/644 交付追加门槛。相邻包装只保留已记录静态债务，不能据此宣称同样的公共反例已经实测。
