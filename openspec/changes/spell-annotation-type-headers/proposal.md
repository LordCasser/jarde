## Why

合法 Java 8 `@interface Basic` 的 class 文件把 `java/lang/annotation/Annotation` 放在接口表。Jarde 已从 flags 选中 `@interface`，却又走普通接口的 `extends` 发射分支，生成 `@interface Basic extends java.lang.annotation.Annotation`；完整类仅此一处就被 javac 拒绝。原类与 JADX 的完整反射执行相同，独立重放证据在 `../../evidence/java-syntax-2026-09-22/annotation-default-boundaries/header-minimal/basic/`。

## What Changes

- 对已确认 canonical Java 注解类型的声明，发射没有显式 `extends` 的 `@interface` 头；类视图仍保留物理接口表事实。
- 保留普通接口的真实 `extends` 和普通类的 `implements`，不让注解语法分支改变它们。
- 用完整 Basic 类/runner 编译、反射运行以及已有注解默认值测试验收；嵌套注解默认值与 enum 常量发射作为独立 change，不并入此头部修复。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 合法注解类型的 class-source 声明 SHALL 使用可编译的 Java `@interface` 头，保留 class 文件的原始声明事实。

## Impact

仅 `src/class_source.rs` 的类头拼写和相应测试/永久 fixture；不改 reader、成员恢复、公共报告形状或依赖。当前嵌套注解值与 enum 输出债务另行规划。
