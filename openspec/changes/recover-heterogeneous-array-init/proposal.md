## Why

EM-18 的已冻结 `CT.cov` 与新建家族对照显示：字节码明确创建父类/接口分量数组，却因元素呈现为子类而在初始化器处拒绝。2026-10-09 基线也发现，生成 Java 编译、运行退出码为零仍可能丢失 main 的输出，因此恢复成功必须由完整源码及运行双流共同确认。

## What Changes

- 对既有 fresh-array 初始化器，使用可证明的单向赋值兼容关系接受元素，保留分量类型、物理写入次序及元素表达式。
- 复用当前平台、数组与快照 header 层级事实；为具体 reference-array store 提取站点证明，支持自有子类、接口及等秩引用数组关系。没有证明的合法 Java 输入仍明确记录为未覆盖。
- 冻结 CT、数值、接口、集合、异常、嵌套数组、自有类族的双 javac 完整对照，以及非法方向、未知层级、位置错配和效果次序负例。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：初始化器元素按本次事实可证明的赋值兼容关系恢复，完整行为对照为验收条件。

## Impact

- `crates/jarde-java/src/build.rs` 的初始化器呈现与既有 reference widening predicates；不改变调用参数的 overload 固定逻辑。
- `src/facade.rs` 的既有快照层级证据生产者；沿用 Runtime 环境选择、预算及 header walk，不新增层级服务、pass、crate 或外部依赖。
- `crates/jarde-java/src/report.rs` 现有站点证明的错误注释与相关测试、fixture、EM-18 账本。

## Prerequisites and Non-Goals

依赖现有 SSA、reader、ArrayInitializers 的 fresh-allocation/唯一消费者/次序证明及完整类源组装。method-only 路径继续遵循其现有证据可用性。

构造元素的结构证明组合另见 `compose-constructed-reference-array-elements`，EM-18 整单元仍保持未完成。

不计算 LUB，不重建泛型推断，不解析任意外部 classpath，不无条件接受 subtype 名字，不添加元素 cast，不放宽一般数组赋值，也不将 CT 外围问题混入此项。不得仅因退出码为零关闭 EM-18。
