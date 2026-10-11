## Why

JADX 的 `TestLoopCondition5` 要求倒序查找呈现为 `for`；当前冻结 Jarde 已恢复正确行为和完整回边来源，却仍输出计算初值加 `while`。限制来自既有 ForHeader 的常量初值门和循环外读取门，应复用同次 SSA、返回叶证书及 Builder，补齐这一有明确上游验收标准的呈现差距。

## What Changes

- 用现有初始化语句承载已证明的计算初值，不增加 `arg - 1` 专用模板或表达式恢复机制。
- 在同一循环上下文中提前计算并复用已有 terminal-return 集合，仅允许其中的循环变量读取，外部写入和其他外部读取继续拒绝。
- 保留 ForHeader 存在时末尾 return/latch If 的准确派生回边来源，不改自然环或 Region 所有权。
- 先用真实 IR 确认求值、类型、作用域与异常上下文，再实现；对独立副作用、共享值、后续读取和预算停止进行对抗验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 将已有安全初始化语句纳入已证明计数循环的 for header，并保留已证明返回叶及完整物理来源。

## Impact

范围为 jarde-java 既有循环证明、初始化语句消费和来源证明，永久真实输入回归及原/JADX/Jarde 整类对照。无新公开 IR、Frame、pass、依赖、CLI 或通用初始化分析器；不扩大其他循环、外部写入或异常恢复。当前 stack 产品自身 CI 与干净交付仍是生产应用前置；本片排在已规划 CF12 同类常量 gate 后，不借窄片成功宣称整个 CF07 或 71 单元完成。
