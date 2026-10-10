## Why

CF12 原上游完整 `TestSwitchWithFallThroughCase$TestCls.class` 中，case1 的条件路径可进入 case2 或公共出口。Jarde 现有单 successor fallthrough 证明不覆盖该形状，实际恢复报 117 的 Loop、arm overlap 和 uncovered，完整类无法重编；JADX 完整类则与原始运行相同。

## What Changes

- 在既有 switch fallthrough 证明中检查有限无环条件路径，只有全部路径归于已证 join、唯一相邻 case 或准确物理 return/throw 才准入。
- 核完整 canonical 边、clone path 与入边所有权，先探测再复用现有 case frame/标签排序；增加必要的 switch-break 证据叶，使同一 case 的局部 break 与 fallthrough 不丢失。
- 真实 class 永久测试、对抗边界、预算/取消及原/JADX/Jarde 全类对照验收，保留来源与失败 raw。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 对准确证明的条件 switch fallthrough 恢复已有 Java case 结构。

## Impact

主要为 jarde-java 的 `region.rs::switch_fallthroughs` 与现有 switch tests。先独立接受 CF12 基线和真实 IR，产品按串行冻结/CI验收顺序交付。仅增加必要的 Region switch-break 叶及其私有控制目标上下文，消费为已有 AST Break；不扩 JVM IR/SSA/frame 类型域，无通用图框架或依赖；类型恢复、嵌套常量名与 computed-init for 属于其他片。此片不代表整 CF12 或 71 单元已完成。
