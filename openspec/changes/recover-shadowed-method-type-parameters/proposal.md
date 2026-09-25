## Why

Java 8 允许方法自己的 `<T>` 遮蔽所属类的 `T`。当前 Jarde 将这类签名作为重复变量拒绝，使已恢复的直接参数返回方法退化为 `Object`/`CharSequence`，原本有效的强类型调用方不能重编；JADX 1.5.6 虽恢复无界版本，却在类 `T extends Number`、方法 `T extends CharSequence` 的合法版本上也丢掉方法泛型。

## What Changes

- 在现有类/方法 `Signature` 解析与擦除证明中按词法优先级解析方法变量，再解析已证类变量；继续拒绝同一作用域内部重名、未绑定、循环界与物理擦除不符。
- 让已支持的静态直接参数返回方法保留合法的同名方法变量及其实际界，原始强类型调用方能在 Java 8 下对 Jarde 完整类重编、执行。
- 以无界及异界两个正例和独立错签名控制做原/JADX/Jarde 对照；不扩大到非静态正文、嵌套类继承外层变量或泛型调用站点绑定。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 在已证类级作用域下恢复方法自有同名类型变量的静态直接返回声明。

## Impact

影响 `crates/jarde-reader/src/signature.rs` 的类型变量作用域/擦除证明及 `src/class_source.rs` 现有泛型方法投影门，定向 reader/Jarde/CLI 测试与 [冻结证据](../../evidence/java-syntax-2026-09-25/method-type-variable-shadowing/analysis.md)。不增加 parser、AST、运行依赖或通用语法 pass；物理 descriptor、`Exceptions` 和 BCI 来源保持原样。
