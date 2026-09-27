# Recover a proved generic null return

## Why

DT-16 的冻结 Java 8 类 `NullResult` 含 `public <T extends Number> T value() { return null; }`。原 class 与固定 JADX 输出均能与 `Integer value = result.<Integer>value()` 一起重编、验证运行，Jarde 当前只输出擦除后的 `Number value()`，使完整 API consumer 编译失败。三方证据与可重放脚本见 [DT-16 报告](../../evidence/java-syntax-2026-09-27/dt16-generic-null-return/report.md)。

## What Changes

- 在现有同轮 AST/Code/SSA 方法返回候选中证明无参数实例方法精确直接返回 `null`，并把这个局部事实交给现有 class-source 泛型方法声明门。
- 复用 reader 的完整 `Signature` 解析/擦除和现有成员来源、预算、绑定拒绝条件，仅对已证单一方法自有类型变量 `T` 的返回位置恢复 `<T extends Number> T`。
- 冻结完整源码 Java 8 编译、`-Xverify:all` 和反射结果，并以非 `null`、隐藏效果、错误擦除、同名调用及预算/取消负例验证拒绝不产生泛型半头。

## Impact

仅扩张 DT-16 的一个已证正例。DT-17 通配符、DT-18 参数化参数、成员泛型类型、泛型覆盖、任意正文与多方法调用保持各自验收边界；独立方法恢复与物理报告不改变。
