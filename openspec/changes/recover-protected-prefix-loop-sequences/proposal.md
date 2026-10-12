## Why

固定 CF11 完整类的 `handled(List,List)` 在外层 try 中初始化 iterator，随后进入 foreach；当前只恢复前置 block0 和 handler50，循环 [7,16] 与出口47 被留为 fallback，导致完整类无法编译。root同次Walker跟踪确认，try 调用方未消费内部 continuation；补充实验调用region_at(7)又实际返回ExceptionEdge(7, ordinal0)，尚未进入loop body。canonical CFG已有自然循环，但循环header/body还缺少当前named catch范围的精确归属，不需要重做图分析。

## What Changes

- 在普通单range try的既有区域入口，按同一个保护frame接回已证明的前缀和循环，并作为一个完整 try body 发布。
- 在该有限试探的Frame中携带由当前Catches绑定的named row事实，让header和body使用同一精确异常归属；不全局继承own_try，也不把任意覆盖当前块的named row当成本Try的clause。
- 对range-end纯跳转复用物理闭合证明，明确它属于try后的continuation；不让循环、handler或效果块被错误搬出保护范围。
- 永久正反例验证完整所有权、handler/异常边、bridge条件、停止和物理来源；双JDK、debug/none、default/all重编译未手修完整类。
- 与 `recover-debug-reference-lifetimes` 分别实现和审查，最终用两项代码共同满足原完整类17场景及保护范围边界运行门。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 恢复普通单range catch保护的前置语句加已识别循环，保留完整异常覆盖和唯一块所有权。

## Impact

主要落点为 `crates/jarde-java/src/region.rs` 的现有try组合及range-end证书，复用已有Try、Loop、Sequence、Frame、来源与预算机制；不新增依赖、pass、solver或公共结构。JADX的RegionMaker连续遍历与ProcessTryCatchRegions区域包装可作算法参考，但不复制其允许重复块的策略。

前置为当前产品 `7e93eb30647338dd8a6ddfc9994392f23457aa3e`、完整基线及两次已归档同次trace。补充子调用证明header拒绝；Frame::loop_body清空own_try是已读源码中的后续缺口，该路径尚未实际到达，必须由候选测试验证。非目标包括任意复合try区域、多个循环、nested try/finally、retry/fragmented特化的扩大、通用异常分析、泛型handler投影和修补生成Java。完成本项不等于完成整CF11或71/612。
