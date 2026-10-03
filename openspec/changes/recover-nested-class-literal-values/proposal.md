## Why

[嵌套类字面量巡查](../../evidence/java-syntax-2026-10-04/nested-class-literal-patrol/README.md)以单变量判别钉死：`Nested.class`（CP 名 `Outer$Nested`）在表达式位不在可证子集内 → 整方法拒绝 → 完整类不可编；顶层类字面量五形（实参/receiver/拼接/链式/存局部）全部健康。反射入口 `Foo.Bar.class`（logger、enum 元数据、内部 Builder、注解处理）为超高频形态；A9.main（17 处引注）与 A11 反射读注解三方法同因。

## What Changes

- 类字面量值的证明准入接受**嵌套 CP 名**：判据沿用既有类型名可拼性事实（与 `recover-nested-type-source-spelling` 同源），不再要求名为顶层形或等于当前类。
- 呈现沿用既有类字面量通道与 nested-spelling 的拼写规则（折叠域内简单名、分离域按该片既有规则），不新增呈现函数。
- A12 两形与 A11 三形（反射读注解）恢复且行为一致；顶层类字面量五形与全部既有类字面量测试逐字不变；不可拼名（本地类 `1$Local`、匿名 `X$1`）保持拒绝。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：嵌套类的类字面量可呈现，反射入口方法完整恢复。

## Impact

`crates/jarde-java`（类字面量准入判据）及测试；复用既有可拼性事实与呈现通道，无新机制。既有类字面量/nested-spelling/反射相关切片零回退。
