## Why

单臂循环候选已完成双 JDK、default/all 八份完整生成类的重编运行对照，但 PlainOneArmLoops 的三个 `for` 方法均缺物理 `goto@20` 来源。现有 ForHeader 已证明唯一更新块和回跳，来源 helper 却排除了 ForHeader；需补全这一已证明的来源，才能关闭上一片全 BCI 验收。

## What Changes

- 复用既有自然循环、ForHeader、末尾 Straight 与物理 transfer 证明，将唯一隐式 for latch 绑定到整个循环语句的派生来源。
- 单臂循环 join 验证读取同一证明，保留已恢复正文、块所有权和停止语义。
- 以现有完整 fixture 验收准确 BCI、物理方法和语句范围，并复跑双 JDK 完整类及相邻来源对照。
- 前置条件：单臂产品 ca43ac74767c4609284f13b251de9d6c349c67a4、已冻结 CLI 和 fresh candidate-whole-classes-root-v3 的实际观测。实现与独立验收尚未完成。
- 非目标：扩大 for 识别、修改 iterator/foreach/递增拼写、修复 CF07 if 汇合与嵌套 else 来源；这些独立缺口继续记录，不宣称整单元完成。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 已证明的 for 隐式回跳必须保留准确物理来源，投影及单臂组合不得丢失它。

## Impact

主要影响 jarde-java 的 region 私有证明与既有循环来源永久测试；Builder 既有 gateway_origins 到 OriginSet 的折叠可直接消费。无新公共 IR、Frame 字段、pass、依赖或核心/适配器跨层接口。保留现有预算/取消计费及保守拒绝；Rust 构建遵循用户授权的机器 5 GiB、本仓 target 1 GiB 守卫与完成清理。
