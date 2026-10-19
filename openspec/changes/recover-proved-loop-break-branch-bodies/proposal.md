## Why

完整原件 TestBreakInLoop4 的三方基线已确认真实缺口：Jarde 保留了 for 循环，却把 break 前的条件赋值块 17/23/29 留为 uncovered，生成整类无法重编。实际 IR 表明这些块构成自然循环之外、通向同一循环出口的条件 DAG；不能用空分支或单条 break 替代。

## What Changes

- 在已证明的头测循环中，恢复单入口、无环、全部路径到同一 break 目标的外侧分支体，保持条件和赋值的原执行顺序。
- 保留自然循环成员及已有 for 初始化/更新证明，使用已有 If、Straight、LoopBreak 表达完整语句，保留物理转移来源。
- 对外部入口、额外出口、回边、异常边归属不明、已占用 owner、预算或取消失败保持明确拒绝或原停止原因。
- 以冻结完整 Loop4 原件的 debug/no-debug、class/JAR、default/all API、完整重编及隔离运行完成验收；Loop5 仅作为后续命名 catch 组合的依赖候选，不据此宣称通过。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 增加有闭合证据的循环 break 分支体恢复及拒绝、物理来源和停止约束。

## Impact

主要影响 jarde-java 的区域证明和其定向测试，不改变 Reader 原始事实、SSA、自然循环发现或公共 API，不增加依赖。前置为 main 已验收产品 618b5e20、基线检查点 ef2ece345，以及四原件实际 IR 探针；证据入口为 `openspec/evidence/java-syntax-2026-10-19/cf09-loop4-loop5-original-root-v1/README.md`。不处理 Loop3/6、任意不可约图、switch/return 型外侧子图或无界一般循环；不扩大 CF09、71 单元或 612 文件的完成声明。
