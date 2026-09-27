# Recover a proved direct parameterized superclass

## Why

[DT-21 的单层对照](../../evidence/java-syntax-2026-09-27/dt21-parameterized-parent/report.md)中，原始和 JADX 的 `Child extends Parent<String>` 完整源码都能 Java 8 重编并在反射中保留 `Parent<String>`；Jarde 可编译却写成 raw `extends Parent`。classfile 自身的 `Signature` 明示泛型实参，父类物理名也准确匹配，差距在类声明投影门。

## What Changes

- 对没有自有类型参数的普通 child，允许现有 class `Signature` 投影一个直接、非嵌套、唯一选定的参数化父类，首片父类是同一选定环境中的 `Parent<T>`，实参为 `String`。
- 证明父定义具有相符的单个形参，child 的 `Signature` 擦除与物理 `super_class` 相同；统一决定整个 class 头，不从调用或局部变量猜类型，也不改动现有构造恢复。
- 固定原/JADX/Jarde 完整源码 Java 8 重编、`-Xverify:all` 和泛型父类反射；缺失/错误/歧义父定义、错 arity/擦除和停止时保留 raw 头及物理来源。

## Impact

修复 DT-21 的单层父类声明首片。`C2<B> extends C1<B>`、多层类型变量代换、继承调用和 bridge 属于 [相邻审计](../../evidence/java-syntax-2026-09-27/dt21-inherited-generic-call/report.md) 的后续切片；不能以本次通过宣称整个 DT-21 追平。
