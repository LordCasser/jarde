## Why

隔离后的 DT-29 接口 fixture 已由原始/JADX/Jarde 完整源码通过 Java 8 重编、`-Xverify:all` 和同一 Runner，确认普通接口 cast、运行时 `ClassCastException` 与 overload 选择在该切片中一致。独立 private-field fixture 只有根类、静态嵌套 `A/B` 和一个 setter：`B extends A`，一次写入父类 public 字段和 private 字段。Jarde 仍拒绝父类字段 owner、未能证明 private accessor 的 `B→A` 实参转换，并为 Java 8 private 写入 helper 留下无返回语句的方法体，导致整组重建源码编译失败。固定 JADX `TestFieldCast` 的多类、泛型和多字段组合另存为待扩证据，不是本 change 的验收输入。

## What Changes

- 恢复已证明的子类接收者对父类声明字段的写入，保持 class file 中字段 symbolic owner。
- 对唯一 Java 8 private-field accessor 证明 receiver 到父类参数的转换，并恢复完整字段写入及返回值消费。
- 以单 setter、单 accessor、反射式外部 Runner 验证三方完整源码编译和字段状态。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：增加固定 Java 8 子类父字段写入及 private accessor 的可验证恢复契约。

## Impact

前置条件为现有字段引用、方法调用、明确的父类关系和单方法恢复事实。实现只覆盖 fixture 证明的精确字段 owner 与唯一 javac private-write accessor；不将 class 名称或 synthetic 标记单独当作授权。

非目标：普通接口 cast（该切片已追平）、任意成员解析/字段隐藏、泛型 D、复杂类族、字符串拼接、多个 accessor/多分支写入、任意 synthetic bridge、无证据的 class hierarchy 推断和通用源码可编译性声明。
