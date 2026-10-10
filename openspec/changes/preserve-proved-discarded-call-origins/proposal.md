## Why

CF12真实上游TestSwitch类型候选已恢复调用正文，但全物理来源测试仍缺append后pop82/92/105。现有丢弃计划证明并跳过这些pop，成功调用语句却没有携带其来源；该债务需独立关闭，才能完成局部类型片的完整来源验收。

## What Changes

- 对既有DiscardedEvaluations已证的调用返回值丢弃，在成功调用语句完整跨度保留pop的derived物理来源。
- 保持调用正文、参数与求值次序、既有primary来源、拒绝quote和未证pop/pop2行为。
- root验证真实class正反例、预算/取消、default/all来源及原/JADX/Jarde完整运行；保留失败raw，独立精确产品CI和clean交付。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 成功调用语句保留既有证据证明的返回值丢弃来源。

## Impact

限jarde-java成功调用语句来源消费与真实测试；无新IR、Frame、pass、依赖或证明计划。前提为return-arm片精确自身CI及e84600a65 clean交付已接受。局部类型、条件fallthrough和一般pop2恢复不在本片；71单元/612测试分母不变。按5GiB余量/target1GiB一秒守卫运行并清理。
