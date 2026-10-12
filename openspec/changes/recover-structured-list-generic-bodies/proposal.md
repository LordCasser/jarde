## Why

固定 JADX CF11 `TestNestedLoops` 的循环已经恢复，但合法 `List<String>` 方法参数仍被降成 raw List。同次诊断显示完整结构化正文未进入泛型候选；双JDK探针还证明，只恢复签名会因 `add((Object)s1)` 编译失败。本片把结构化正文、参数用途和调用参数投影一起完成，使完整生成类恢复泛型并保持行为。

前置局部引用写兼容片已完成本地、确切产品自身CI和最终已推送main审计，产品基底 `5f5fb2c605d61e18f392bb3abe6bdae6e10bf41e`，文档检查点 `f12668ed0dc022fee42a69bc9723427eece33d31`。本片尚未实现。

## What Changes

- 在既有泛型方法投影中接纳完整、无handler的结构化void正文，闭合物理Code/SSA/effects、参数身份与实际源码用途；不以“无参数重赋值”替代调用可赋值性证明。
- 恢复确切 `List<S>` 参数与有限平台调用契约；支持嵌套foreach、条件、合法add/对象remove/size/clear/iterator用途，覆盖String和另一不同元素类型，避免用固定类名或源码名字识别。
- 复用保留AST、精确调用键、presentation-cast移除和事务投影。仅对证明合法的泛型E参数去掉合成descriptor包装，保留物理checkcast、其他调用绑定、执行顺序和来源。
- 固定用例及近邻控制真实编译，原/JADX/Jarde双JDK完整类对照；报告值、状态、异常、来源与拒绝。本片不扩大总体CF11/71/612完成数。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 完整结构化 `List<S>` void方法的参数与调用投影必须作为同一有界源级决定，不能发布泛型签名与擦除实参矛盾的源码。

## Impact

主要影响 `crates/jarde-java/src/report.rs` 的既有同次候选/保留AST接口和 `src/class_source.rs` 的泛型投影事务及有限平台契约，必要永久API/fixture。不新增通用泛型solver、类型层级/identity服务、第二AST、独立pass、外部依赖或隐式JDK扫描；未知层级、wildcard capture、参数重赋值、raw别名escape、handler与不完整恢复不在本MVP证明内。5GiB机器余量/target1GiB/一秒守卫/default stack/最终clean保持；root独占共享工具链与Git，Luna私有实现。
