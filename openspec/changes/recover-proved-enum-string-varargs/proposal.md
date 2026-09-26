## Why

DT-11 的 int 构造实参已恢复，但冻结的 `StringVarargs` 仍因 enum 构造器尾参是 `String[]` 而整组拒绝；Jarde 的完整类源码不能以 Java 8 重编。JADX `TestEnums4` 和三方冻结对照表明字符串 varargs 是独立的可验收语法形态；其数组分配、元素写入和构造调用均可从现有同次 Code 事实中限定证明。

## What Changes

- 在既有 enum 常量组证明中增加一条封闭的 `String...` 构造实参形态：每个常量恰有一次新建 `String[]`，按索引填入已证明的字符串 literal，随后把该数组唯一传入准确构造器；零元素数组也作为真实分配证明。
- 证明构造器的隐藏 name/ordinal 参数、`ACC_VARARGS`、源 Signature `([Ljava/lang/String;)V`、`Enum` super 调用和唯一 `String[]` 字段赋值后，输出合法的 `String...` 参数及常量实参；保留物理 `<clinit>`/构造器报告和来源。
- 用冻结原 class、JADX 与修后 Jarde 完整 Java 8 重编和 `-Xverify:all` 运行验收；数组别名、额外读写、非 literal 元素、其他数组类型、构造器重载及预算停止继续原子拒绝。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 扩展有界 enum 常量和构造器源码投影，覆盖准确证明的字符串 varargs 数组初始化与零元素常量。

## Impact

复用 `src/enum_constants.rs` 的同次 raw Code/BCI、构造器及 use census，`src/class_source.rs` 的 enum 构造器 Signature/源码装配，以及 `src/facade.rs` 的现有 class-source 提交边界。无需新 crate、读取器、外部依赖或宿主 API。首切片以前述已合并的普通 enum/int 参数证书为前提，不扩展匿名常量体、任意数组表达式或 enum 构造器重载。
