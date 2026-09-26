## Why

Java 8 的 `package-info.class` 承载包注解，源文件语法却是“注解 + `package` 声明”，不是普通接口声明。当前 Jarde 把标准产物写成非法的 `interface package-info`；同一冻结样本的原版与 JADX 完整源码可以重编运行，Jarde 完整源码无法重编。

## What Changes

- 在 class-source 的既有类声明/注解装配路径中，为准确证明的 Java 8 `package-info` 物理类输出包注解和 `package p;`，不生成虚构的接口体。
- 只在类名、包、flags、空成员表、注解与来源均完整且可写时原子发布；不完整或非标准形状保留物理报告和明确拒绝。
- 用冻结 class、JADX 与 Jarde 的完整源码进行 Java 8 重编和 `-Xverify:all` 对照，并验证包注解在运行时可见。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：准确证明的 `package-info` 物理类应恢复为可编译的包声明与包注解；证据不足时不得把任意空接口猜作包声明。

## Impact

影响 `class-source` 类级声明/注解文本装配及对应来源映射，不改变 classfile 读者、物理身份、单方法恢复、公开查询模型或 CLI 协议。首切片只覆盖有包名的 Java 8 标准 `package-info` 和已能完整拼写的包注解；一般 import 优化与外部注解类型解析不在本项。
