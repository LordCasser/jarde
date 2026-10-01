## Why

[InnerClasses family 巡查](../../evidence/java-syntax-2026-10-01/inner-enum-args-patrol/README.md)确认账本挂起项的根因：枚举常量折叠的构造 descriptor 白名单只有四种固定形，任何用户参数形态（`(byte, String)`、`(NumString)`、`(byte, NumString)` 等——`TestInnerEnums` 的"全部构造实参"）直接拒绝折叠，退化为逐字段呈现 + `valueOf` 拒绝。两个判别探针证明根因同源（任意实参），且 `Refs(Simple.A)` 形态的实参是单条 `getstatic`——按限定名拼写即忠实等价，**无需跨类机制**。

## What Changes

- 折叠的构造 grammar 扩展：`(Ljava/lang/String;I<arg>)...` 形的任意 descriptor，用户实参数 ≤3；每实参为 {int 族字面量（iconst/bipush/sipush/ldc int，按参数类型拼窄化 `(byte) 1`/`(short) …`）、String 字面量（ldc）、静态字段引用（单条 getstatic，拼 `Owner.name`）、null（aconst_null）}。
- ctor 体纪律沿用现有证明：super(name, ordinal[, 委托链]) + 每用户参恰一次存 own final 字段、无其它语句；常量步骤纪律沿用（new 本 enum、dup、ldc 名、iconst 序数、实参、invokespecial、putstatic own 常量字段）。
- N3$Simple/N3$Refs/N1$Numbers 全量折叠（`A((byte) 1, "x");`、`A(Simple.A);`、`ONE((byte) 1, NumString.ONE);`）；N0 基线与其余四固定形逐字不变；N2$Operation（匿名体）不在本片。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：自定义参数枚举构造的常量列表可折叠（有界实参语法），嵌套枚举交叉引用按限定名呈现。

## Impact

仅根 crate `src/enum_constants.rs` 折叠证明与呈现及测试；无跨类读取、无新机制。既有四固定形、map-suffix、varargs 折叠与全部 enum 相关已验收证书零回退。
