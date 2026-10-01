## Why

[判别矩阵](../../evidence/java-syntax-2026-10-01/inner-enum-args-patrol/nested-bodies-matrix/README.md)确认：已合入的 `recover-proved-enum-constant-bodies` 闭合了**顶层名**枚举的常量体折叠；**嵌套名**（枚举二进制名含 `$`，如 `Holder2$OpAbs`、`TestEnumsInterface` 的 `TestCls.Operation`）仍逐字段降级，判别变量与抽象/接口实现无关（四个矩阵格钉死）。方向：子类名派生/匹配在父名含 `$` 时错位。

## What Changes

- 定位并修复常量体折叠中依赖"父名无 `$`"假设的子类派生/匹配点（子类名回切父名、InnerClasses 关系解析或常量名提取的 `$` 段切分），使嵌套枚举的常量体折叠与顶层同语义。
- `p.Holder2$OpAbs` 与 `N2$Operation` 全量折叠（常量体含覆盖方法文本）；`demo.Op`/`p.OpIface` 顶层行为逐字不变。
- 不放宽既有义务证明（ctor 委托体/成员集/structured 体方法）——仅修名派生。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：嵌套枚举（名含 `$`）的常量专属匿名体折叠，与顶层同语义。

## Impact

根 crate `src/enum_constants.rs`/`src/class_source.rs` 的名字派生点及测试；矩阵四格为正反 fixture。既有顶层常量体与全部 enum 证书零回退。
