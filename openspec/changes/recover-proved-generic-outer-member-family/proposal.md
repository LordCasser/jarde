# Recover a proved generic outer member family

## Why

[DT-19 的固定对照](../../evidence/java-syntax-2026-09-27/dt19-outer-generic-member/report.md)中，`Outer<T>.Inner` 与 `T id(T)` 的原始和 JADX 完整源码可由同一个 `Outer<String>.Inner` consumer 以 Java 8 重编并验证运行。Jarde 已证明唯一 child 关系与捕获，但因现有家族调用门只接受非泛型头而拒绝嵌套源码；child 的 `T` 也没有已证明的外层作用域。

## What Changes

- 把现有单根单直接成员的家族关系、捕获和构造证明扩到一个泛型外层类与无自有类型参数的非静态命名成员。
- 在这个已证明的词法家族内，把外层 `Signature` 的类型变量作用域传给 child 声明及方法签名，并为根返回类型选定 `Outer<T>.Inner` 的源码路径；物理 descriptor 与擦除仍是准入前提。
- 根与 child 的完整源码、构造调用、隐藏捕获构件和来源一次提交；不成立时保留两个物理报告及明确拒绝。
- 固定原/JADX/Jarde 的全源码 Java 8 重编、`-Xverify:all` 与 API consumer，并覆盖错关系/错作用域/额外使用/停止负例。

## Impact

只修复 DT-19 的一个直接成员首片。多 child、多层泛型路径、成员自有类型参数、接口实现、无 debug 局部泛型流和 Smali `TestGenericsInFullInnerCls` 留在 DT-19 后续扩验；本变更不增加通用泛型推断器。
