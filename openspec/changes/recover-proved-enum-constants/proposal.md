## Why

Java 8 的两常量 enum 已能被识别出 `enum` 类头，却仍把常量写成普通字段，暴露 JVM 注入的构造参数及 `$VALUES` 等编译器成员，导致完整类无法编译。`enum-declaration/` 的原 class 与 JADX 全类可执行，Jarde 只在常量声明处出现 javac 错误；带用户 `static` 逻辑的边界说明不能简单删除整个 `<clinit>`。

## What Changes

- 在同一类的字段、构造器和 `<clinit>` 中证明有限的 Java 8 枚举初始化模式后，把常量按物理字段顺序写成带源参数的 enum 常量列表。
- 只消费已证明属于该模式的隐式 name/ordinal、`$VALUES` 初始化及标准辅助方法，保留其后的用户静态初始化、用户字段和方法；公开报告继续保留每个原始成员和各自恢复结果。
- 证明不全时不猜测常量参数、不按名字吞掉成员，维持保守来源。复杂常量类体、控制流初始化和跨类合并另案。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 对已证明的有限枚举类输出可编译、行为等价的 Java enum 常量与用户成员。

## Impact

影响类源码装配与必要的类级结构证明，复用已有 reader、prepared class 和成员恢复；无需新 crate、依赖或目标代码执行。当前 `present-proved-java-structure` 的枚举场景要求在**尚无类级证明**时不合并 `<clinit>` 常量赋值，本 change 补充有证明后的窄例外；实施前应协调这两份尚活跃的 delta，而不能让相反断言并存。与正在进行的复合赋值/注解修复隔离，串行占用 Cargo 窗口。
