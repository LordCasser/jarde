# Recover a proved parameterized null return

## Why

DT-18 的固定 `List<String> empty(){return null;}` 在原 class 与 JADX 完整源码中保留返回 `Signature`；Jarde 的完整源码可重编运行，但返回反射类型退化为 raw `List`。[三方对照](../../evidence/java-syntax-2026-09-27/dt18-parameterized-members/report.md)同时确认未使用参数化字段与参数直接返回已通过，差距精确落在 null 返回候选的消费边界。

## What Changes

- 在 DT-16 的精确无效果 `NullLiteral` 同轮候选通过独立验收之后，让现有普通方法参数化声明门消费该候选，只准入顶级普通类的公有无参实例 `List<String>` 返回。
- 继续使用 reader 的方法 `Signature` 作用域、完整解析与 descriptor 擦除，现有正文候选、同名调用绑定拒绝、来源和原子投影；不增加第二种 null 扫描。
- 固定完整源码 Java 8 重编、`-Xverify:all`、反射及 raw 负控制；验证错误签名、正文效果和预算停止时不发布泛型半头。

## Impact

仅修复 DT-18 的一个已证子形态。嵌套参数化类型、字段读取、局部变量泛型推断、任意参数化返回表达式和 DT-16 方法自有 `T` 仍按各自验收边界处理。
