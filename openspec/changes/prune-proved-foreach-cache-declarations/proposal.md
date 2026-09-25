## Why

已证明的数组增强 `for` 把长度缓存和下标更新折叠进循环头后，完整类源码仍在方法顶部写出不再使用的 `int local3; int local4;`。结果虽可编译且执行等价，却让恢复出的结构显得不完整；按名字清理又会误删同槽复用或拒绝路径真正需要的声明。

## What Changes

- 只在现有数组增强 `for` 证明成功时，识别同一次投影完全消费的长度缓存与归纳下标的前置无初值声明，并与循环投影原子删除。
- 被删除声明的原 BCI 仍由增强 `for` 来源覆盖；失败、索引逃逸、无法区分同槽前后变量、预算耗尽或取消时不作声明清理，保留现有循环投影/拒绝结果。
- 用原/JADX/Jarde Java 8 对照复核已存在的数组正例和独立负例，检查文本、完整类执行与 source map。
- 不引入全局未使用局部删除 pass，不改变已验收的数组/Iterable 增强 `for` 等价门槛，也不把泛型元素类型或同槽异类型局部的身份修复混进来；后者已有独立反例。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已证明的数组增强 `for` 不再保留仅供原计数循环使用的失用前置声明，同时维持来源与拒绝路径。

## Impact

集中在 `jarde-java` 的声明规划与数组循环投影提交接缝，以及现有数组增强 `for` 测试/夹具。无新依赖或公开 API。现状定位见[独立分析](../../evidence/java-syntax-2026-09-24/foreach-cache-declarations/analysis.md)与[已验收数组投影](../project-proved-enhanced-for-loops/verification-root-array.md)。
