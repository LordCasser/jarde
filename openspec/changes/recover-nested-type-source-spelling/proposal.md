## Why

[嵌套拼写巡查](../../evidence/java-syntax-2026-10-02/nested-name-spelling-patrol/README.md)实锤：嵌套类型引用以常量池二进制名（`V1$Op`）直拼在源码语法位（参数类型、枚举常量限定）——`$` 非源码语法（须 `Op`/`V1.Op`），javac 拒编 → 含此类引用的类不可重编。单类与 jar 输入一致；包级类简单名正常（仅嵌套形态缺陷）。

## What Changes

- 类型引用的源码拼写转换：常量池名含 `$` 且为嵌套形态时，自嵌套（`Outer` = 当前类）呈现简单名 `Inner`，否则点分 `Outer.Inner`；作用于既有类型拼写出口（声明/参数/局部/调用限定/new/instanceof/cast 等源码语法位统一经过处）。
- V1 家族整类重编通过、行为一致（`Op.MUL` 限定与 `Op` 参数类型）；包级简单名、平台限定名（`java.util.List` 等）、匿名/局部类合成名（`C1$1` 呈现既有约定）逐字不变。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：嵌套类型引用按源码语法（简单名/点分）拼写，含此类引用的类可重编。

## Impact

仅 `crates/jarde-java` 既有类型拼写出口（单点）及测试；不动证明层。既有合成名约定（匿名类 `X$1` 家族呈现、enum 折叠文本）零回退。
