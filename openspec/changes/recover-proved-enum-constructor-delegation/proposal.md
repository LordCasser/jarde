## Why

Java 8 枚举允许 `ZERO` 调用无源参数构造器、再以 `this(0)` 委托到 `int` 构造器，同时另一个常量 `ONE(1)` 直接调用后者。[独立三方对照](../../evidence/java-syntax-2026-09-25/enum-constructor-delegation/analysis.md)中原 class 与 JADX 完整源码均可重编、运行并保留两个反射构造器；Jarde 尚不能重编 enum。现有普通枚举 change 明确只证明单个保存整数参数的构造器，不能凭它推出此委托链。

## What Changes

- 在基础变更交付的同次两常量候选与隐式成员校验接缝上，按精确物理构造器身份和调用边证明一个无源参数构造器只以常量 `0` 委托到唯一 `int` 构造器，另一个常量直接传 `1`；整组证明通过后投影源级 `ZERO, ONE(1)` 与两个构造器。
- 保留终端构造器中用户调用与字段写入的原顺序和次数，移除的仅是已证 JVM 注入的 name/ordinal 传递及 `Enum` super 调用；原始物理成员、恢复结果和来源仍在报告中。
- 在委托目标、实参、额外效果、辅助方法或来源不完整时拒绝整组投影，不把 `ZERO` 猜成 `ZERO(0)` 或吞掉无参重载。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 对已证明的 enum 构造器委托输出可编译、行为与反射构造器形状一致的 Java 8 源码。

## Impact

前置为 [recover-proved-enum-constants](../recover-proved-enum-constants/tasks.md) 的单构造器两常量计划完成并独立验收；本变更才扩展到双构造器委托，不修改正在实现的基础 change。影响现有 class-source 的类级证明、构造器声明/正文投影与来源测试，不新增公开 IR、reader schema、crate 或依赖。任意委托链、重载集合、复杂常量参数、常量专属类体和跨类内联不在本变更范围。
