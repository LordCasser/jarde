## Why

Java 8 类的 `RuntimeVisibleAnnotations` 已在原 class 中，却只作为属性壳读取；Jarde 类源码静默省略 `@Deprecated` 和 `@Retention(RUNTIME)`。完整源码虽能编译，反射分别从 `true`、`@Retention(RUNTIME)` 变为 `false`、`null`，因此必须把类声明上的实际注解值纳入忠实恢复边界。

## What Changes

- 在现有按需属性读取中解析**类级** `RuntimeVisibleAnnotations` / `RuntimeInvisibleAnnotations`，复用已存在的 `element_value` 树、深度限制、常量池和预算规则。
- 在类源码声明前按可证明的属性内顺序写出可拼写的注解使用，含无参和具名值；值无法忠实拼写时保留明确拒绝标记，不输出半个注解。
- 用三份完整 Java 8 类的原始/JADX/Jarde 编译、全校验和反射对照验收，并固定原始属性及损坏/不可拼写边界。
- 不恢复字段、方法、参数、类型使用或注解继承关系；这些位置各有自己的属性与来源归属，独立巡查。不从 `Deprecated` 标记属性或类名推断注解。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：类源码仅在读取并验证类级注解属性后呈现对应注解使用；反射可观察值和原始属性归属必须保留。

## Impact

触及 `jarde-reader` 的属性内容解析、`src/class_source.rs` 的类级装配、`src/facade.rs` 同次类读取到装配的交接及定向 Rust/Java 测试。不改变方法 IR、恢复 AST、CLI 参数或宿主适配边界；类源码 JSON 若增加注解拼写/拒绝记录，以现有报告结构承载。
