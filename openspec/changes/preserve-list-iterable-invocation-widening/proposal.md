## Why

调用方把 `java.util.List` 实参传给 `java.lang.Iterable` 形参是 Java 8 的合法上溯，但当前 Jarde 因缺少该引用关系证据而引用掉整个调用；最小完整类仍能编译，却不再执行被调用方法。独立探针没有 foreach、重载或循环体，已把缺口限定在调用实参类型判定。

## What Changes

- 在 Java 8 调用实参中，接受从精确呈现类型 `java.util.List` 到目标类型 `java.lang.Iterable` 的安全上溯，使物理调用仍可重编译并执行。
- 保留实参只求值一次、调用 BCI 与实参来源；不因这个关系扩展成一般类层级或泛型兼容性求解。
- 其它未证明的引用转换继续拒绝。数组、重载选择和 foreach 投影保持各自已有边界。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：补充一个可证明的平台引用上溯调用实参场景。

## Impact

影响 `jarde-java` 既有调用实参呈现与 `java8-recovery` 行为契约，不新增 crate、依赖、CLI 选项或公开 API。步长索引循环因局部跨引用区域无法呈现，是另一项局部生存期/Region 恢复债务，记录在 CF-10 证据中，不属于本 change。
