# Proposal

## Why

`Engine::class_view` 的 `BodyRef::Method` 按 raw name/descriptor 首取成员；同一完整成员表含重复签名且首条无 Code 时，会返回确定的 `NotDeclared`，而同输入的 `BodyRef::Name` 已正确返回 `Ambiguous`。已有 `PhysicalMethodId` 不含 member ordinal，不能证明它唯一选中了重复记录，原 driver/callee 修复也不经过这个门面分支。

## What Changes

- 保留 foreign-owner 校验优先级，让两种 body 引用在确认 owner 后使用一致的候选收集与唯一性判定；完整重复匹配返回既有 `OperationOutcome::Ambiguous` 及全部物理候选。
- 在判断有无 Code、准备类或解码方法体之前处理歧义，不把首条 abstract/native 声明当作唯一目标；原成员列举及每条 index/raw identity 继续保留。
- 补公共 API 的重复成员排列、名称/身份一致性、唯一声明和 foreign-owner 控制，保留原部分成员表、预算和逐 body 结果规则。
- 复用已有候选与报告类型，收敛本地重复选择逻辑，不扩展公开身份或引入选择框架。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `analysis-contracts`：细化任务目标选择的既有唯一性要求，使 class-view 的已知成员身份不能绕过重复物理签名的歧义判定。

## Impact

主要涉及 `src/facade.rs` 的 body reference 解析及公开 class-view 回归。前置为现有同次 Header/member table、`BodyRef`、候选项与 `OperationOutcome`；不新增依赖、公开 carrier、ordinal 请求字段、全类合法性检查或 reader/JVM 选择器。

这是独立于 `reject-ambiguous-no-code-method-selection` 的门面入口修复；该片的 direct/prepared driver 和 callee 证据不覆盖本入口。私有公共复现已得到唯一/foreign-owner 两项控制通过，以及首个 abstract+Code 重复组合的预期失败；其余矩阵仍待实现后实际执行。当前只有规划及失败证据，不代表生产修复、主线测试或 CI 已通过。
