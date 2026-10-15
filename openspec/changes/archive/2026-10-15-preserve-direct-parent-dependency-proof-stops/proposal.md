# Proposal

## Why

真实 DT-29 公共 class-source 在直接父类 accessor/field 证明发生 ClassHeaders 预算拒绝后，仍交付 Complete、CLI exit0。两条已取证路径需要保留原停止，并在首次停止处结束恢复，避免把被中断的证明写成正常缺证。

## What Changes

- 保留直接父类 accessor/field 证明的原 Error 或依赖报告所携带的预算、取消、期限停止及全部原诊断。
- 停止后保留本次 analysis、已获准用量和可靠成员前缀，不继续正文、后继证明或可选证据工作；依原准入规则发布停止报告。
- 保持宽预算父字段 owner cast、物理 helper 调用及普通缺失、歧义、不适用的保守拒绝；用公共库/CLI、真实收费中的停止和完整源码编译执行对照验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：新增直接父类依赖证明停止在恢复和类汇总中可见的要求。

## Impact

范围是根门面两条消费者、仅由它们使用的直接父类关系读取及必要回归；复用现有 typed proof 交接与 stopped-before-recovery 构造，不新增依赖或通用框架，不改全局预算/合并策略。

实施前提为 `c8718aeff6cc08b686c7b3c3495bfd0bb9efa2b1` 已存在的最窄交接；建项时该提交的接口/snapshot/double-brace 自身 CI 仍在运行；现自身 CI38692921405 已成功、两片已验收归档，不能据此冒称本片通过。当前只有18次普通预算观察、两条只观察 trace 与宽预算完整源码对照；本片尚未实施。reference-overload、CLI 最终文档输出预算、控制流恢复覆盖等均不在本片范围，不将 method-only 入口冒称本次公共反例。
