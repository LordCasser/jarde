## Why

固定 JADX `TestNestedLoops2` 在当前产品的循环和方法体均已完整恢复，但 `test(List<String>)` 仍因参数用途证明缺少 primitive 返回和 indexed get 两部分而降为 raw List。这是已实际重放的 CF11 质量缺口；优先复用现有同次证据和泛型事务，使声明恢复与完整正文一致。

## What Changes

- 为完整结构化、结果仍为原物理 void 或精确 primitive 的方法复用现有参数读写用途证明，恢复受证的 `List<E>` 参数声明。
- 对精确 `List.get(int)` 的结果、真实元素转换及源级调用逐项闭合；保留物理 CHECKCAST 的效果和时序，E 仍由合法物理 Signature 选定。
- 复用现有 staging/final 重验证、opaque AST 和 raw/report/map 平面；缺证保留已可靠恢复的循环与擦除声明，真实停止原样传播。
- 前置条件：主线产品096a764dc及其CI39138950945已接受，当前文档主线3f888afb1干净并与远端一致；本片尚未实现。原始完整上游、两版JDK和JADX/Jarde基线及边界控制须在集成补丁前由root实际接受；私有实现可与这部分采集并行。
- 非目标：引用、数组、类型变量返回的一般完整体证明；任意List成员/别名/泛型求解；移除真实get转换；循环重构或变量美化；整CF11、71单元/612测试完成声明。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 完整 indexed List 方法的受证泛型参数声明、物理转换保留及缺证/停止输出边界。

## Impact

涉及 `crates/jarde-java/src/report.rs` 的现有候选、`src/class_source.rs` 的声明准入和 `src/facade.rs` 的List用途闭包/事务；保留现有crate分层和方法级恢复入口。回归复用公开class-source与泛型List测试，补实际完整源码双编译器/双VM对照；不新增依赖、IR、pass、请求载体或运行时平台数据库，不复制JADX代码。
