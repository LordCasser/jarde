# Proposal

## Why

`c814267719ebc6d8fe14535ebee1b85e41d444db` 的真实公共 `class-source` 请求中，同快照层级前置证明与 double-brace 后置投影各自发生了真实 ClassHeaders 预算拒绝，最终却仍为 class Complete、CLI exit0；后者甚至保留正常正文，证明 content 成功不能代替请求完成。需要让两条依赖证明消费路径保留原执行停止，区分普通缺证与请求中断。

## What Changes

- snapshot hierarchy 前置证明停止时保留本次 raw analysis 与可靠前缀，在正文提交前返回既有 Stopped/not_produced，不继续后续正文、证明或投影收费。
- double-brace 后置依赖证明停止时保留已提交 main、兄弟成员及可靠 raw/source 结果，将原执行停止汇入 class aggregate；终止后继 anonymous-interface 等投影，不清空已提交正文。
- 同时承接原 `Err` 与 resolver 非 Complete report 中承载预算/取消/期限的共享请求停止两条渠道，保留原维度、usage 和诊断；完整读取后的缺失、歧义、不适用仍是普通保守拒绝。
- 保存六次普通 CLI 对照、两条已定位消费路径的原始 trace、真实输入与两个独立审计副本；实施时转为永久公共回归，不让产品 CI 依赖 GDB。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：明确 class-source 的 snapshot 前置证明与 double-brace 后置投影的停止、已提交内容保留及请求汇总契约。

## Impact

预期修改根门面 `src/facade.rs` 的两条 proof 交接、其必要停止构造入口和公共测试。复用既有 Error、ExecutionReport、诊断与报告初始化；不新增 crate、依赖、全局 sticky Budget、通用 pipeline 或层级服务。

前提是现有同快照 widening/double-brace 证明及物理身份；狭义 `preserve-interface-super-proof-stops` 仍只有计划、尚未完成实现，本片拟复用它的最窄停止构造或等价入口，实施前须核定真实可用状态。两片不互相冒领完成；本片不修 reference-overload、direct-parent accessor/field 三个未动态证明的包装债务，不扩 class-view、CLI output_bytes 债务或当前四工程 closure/既有 c814 CI 门槛。

目前六次 CLI 是缺陷/对照观察，不是修复验收；取消、期限和其他预算原因尚未在本家族实测。最终实现必须绑定自身完整源码、适用项目门禁与自身产品 commit CI，不能以 c814 的既有局部/矩阵通过证据或其他提交 CI 替代。证据与来源见 `results/README.md`。
