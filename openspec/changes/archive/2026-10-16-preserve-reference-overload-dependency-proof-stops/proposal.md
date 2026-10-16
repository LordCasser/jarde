# Proposal

## Why

EM11 的真实 class-source 请求在 reference-overload 证明首次拒付 Header 预算后，仍进入后续 snapshot 证明并再次拒付；最终 Partial 掩盖了首停之后仍有工作的事实。源码还存在将该消费者的原错误吞为空证明、将停止报告解释为无事实的边界，需要按已有 typed proof 停止契约收口。

## What Changes

- 让 reference-overload 前置证明及其必需依赖读取保留共享请求的原预算、取消、期限停止，在正文与后继证明开始前结束，保留 raw analysis、真实用量及已获准前缀。
- 将原 Error、原 Report 与完整读取后的普通拒绝分别验收，保留普通缺失/歧义、匹配条件及平台缺失判据。
- 复用最窄的既有测试观测，新增可在旧生产上实际失败的首停回归；最终冻结 CLI 对未改 EM11 整组生成源码原样重编并与原始 Runner 五行结果比较，取得该修复自己的 CI。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：新增 reference-overload 前置证明的可观察停止要求，覆盖必需 owner/source 与平台缺失检查，不改变现有正常证明范围。

## Impact

前置阶段为已验收产品 aadf71、归档/main f30df5d9；4905产品输入与冻结 CLI c2db4721…相同。立项基线与边界见 results/repro-v1；旧生产反例与后续实际实施状态以 results/old-native-v1 和 tasks.md 为准。

修改 src/facade.rs 的局部消费者与永久库/CLI回归，沿用既有报告和工厂，无新依赖/API/框架。非目标：不改全局 Budget、合并/发布准入、其它证明消费者、层级终端策略、平台契约或通用重载解析；不以当前观察、旧 CI 或结构 quality 宣称已修复、可编译或语义等价。
