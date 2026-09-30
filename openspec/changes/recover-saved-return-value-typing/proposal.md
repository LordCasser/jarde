## Why

[CF-17 巡查账本](../../evidence/java-syntax-2026-09-30/cf17-twrcatch-patrol/README.md)登记：TWR saved-return 呈现不可编译——`T2.voidBodyReturnInside`（`try (T2 r = new T2()) { touch(r); return "in"; }`）输出 `Object local1 = "in"; return local1;`，方法返回位是 `String`，`return local1`（按 `Object` 声明）不可编译。字节码：`12: ldc "in"; 14: astore_1`（保存值）、清理后 `19: aload_1; 20: areturn`。既有 `p3_java_recovery.rs:1389` 钉死 `Object local1 = null;` 为 null 保存值的期望——null 无窄型合理；但非 null 值（字面量/调用结果）有自身类型，声明应从值生产者细化。与 Test2 已修的 `array_of_value`（元素类型从 `aaload` 细化）同族：`written_type` 通道（build.rs 24316）已按 `value_type`/`array_of_value` 细化，问题在 saved-return 声明决策未走或未命中该通道。

## What Changes

- TWR saved-return 局部的声明类型按保存值细化：字面量（ldc String→String、iconst→int）、调用结果（descriptor 返回类型）、构造点（new 类型）沿用 `written_type` 既有细化；细化失败时回退现拼写（保持既有行为，含 null → Object）。
- 既有期望 `Object local1 = null;`（null 值）不变；`T2.voidBodyReturnInside` 输出 `String local1 = "in"; return local1;` 且整类可 `javac --release 8` 编译。
- 以 T2 固定类、null 对照、多种保存值类型（String/int/构造/调用返回/数组）验收；TWR 家族与其余 saved-return 证书（Test5/7/9、Tf 家族）零回退。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：TWR saved-return 局部声明按保存值的生产者类型拼写，使 try 内 return 形态可重编。

## Impact

仅 `crates/jarde-java` 私有 build.rs 声明类型决策及测试；不新增机制（复用 `written_type` 细化通道）。细化失败回退保证既有输出不受影响。
