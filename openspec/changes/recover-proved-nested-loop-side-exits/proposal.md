## Why

JADX 固定用例 TestNestedLoops4 的完整源码可编译运行，当前 Jarde 因中间循环的退出路径不在自然环集合内，将内层循环入口截成空分支，随后可靠拒绝整个方法。真实 IR 已定位此缺口，需要证明 Java 正文中的嵌套循环、break 和早返回归属，补齐已有循环恢复，而非只放宽最终拒绝门。

## What Changes

- 保持 natural-loop 集合不变，在既有循环证明中准入精确单入口、包含一层内嵌循环的退出路径，并核其准确 break gateway 和 terminal return。
- 让已获证明的正文 scope、覆盖集合、物理来源和外层 prefixed-arm 接续共同消费同一完整归属结果；拒绝额外入口、出口、异常边、重复所有者与不明嵌套。
- 复用现有 Frame、Region、loop transfer/return、完整边扫描及事务回滚，不新增公开 IR、pass、CFG 改写或任意可达子图恢复器。
- 以固定上游整类和可实际到达早返回的受控输入验证源码、JADX、Jarde；同时验证全部物理来源、预算与可靠拒绝。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 已证明的嵌套循环退出路径能够成为正确的 Java 循环正文，并被外层条件接续准确接受。

## Impact

主要范围为 jarde-java 的 region.rs 循环 scope、ownership 和 arm 接续证明，必要的永久真实 class/API 回归与整类对照。Builder/emitter 仅在实际发现既有 Loop/If/LoopBreak 消费不完整时作最小修订并记录原因；不引入依赖或改变 Reader、SSA、NormalFlowView、Query、CLI 公共协议。前置产品 8bef1b76 自身 CI 和 b922a0ec 干净主线已接受。CF16 finally、通用多出口循环、跨层任意共享子图及整 CF11/71 单元完成声明不在本片范围。
