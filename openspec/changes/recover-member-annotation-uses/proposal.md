## Why

Java 8 字段、方法和参数上的 `@Deprecated` 属性均真实存在，而 Jarde 输出的完整两类源码可编译却全部省略这些使用；反射从 `true / true / 1` 变成 `false / false / 0`，普通方法结果仍为 `5`。类级注解读取已另列 `recover-class-annotation-uses`，本项沿用其最小解析路径处理成员归属和参数位置。

## What Changes

- 对字段/方法自身的运行时可见、不可见声明注解，以及方法的参数注解属性作按需内容读取；只用物理属性事实，不从 `Deprecated` 标记或名称推断。
- 在字段、方法声明前与每个参数类型前写完整可拼写的 Java 8 注解使用；参数按 descriptor 的**参数位置**对齐，不把 JVM slot 当位置。
- 对值不可拼写、参数个数与 descriptor 不符、重复类型及受损属性明确拒绝，不静默输出部分注解或把未完成读作缺省。
- 用冻结的完整两类原/JADX/Jarde 源码在 Java 8 编译与 `-Xverify:all` 反射四行对照验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：类源码在成员声明及参数位置保留已证明的声明注解使用与来源；失去值或位置证明时明确降级。

## Impact

实施需复用 `recover-class-annotation-uses` 验收后的读树及拼写，扩展 `jarde-reader` 的参数注解属性解析、`src/class_source.rs` 的字段/方法签名拼写与 `src/facade.rs` 的同次成员读取交接。方法体 IR、AST、目标代码执行、任意外部注解类型解析及 type-use 注解不在范围内。类级注解 change 未验收前不启动共享代码实施；规划和独立证据可先完成。
