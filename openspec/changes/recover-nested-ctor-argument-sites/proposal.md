## Why

[嵌套构造巡查](../../evidence/java-syntax-2026-10-02/nested-ctor-argument-patrol/README.md)确认：构造实参位是另一完整构造（`readCause(new Exception("top", new Exception("inner")))`——异常包装链/包装对象的高频形态）被构造站点走查误拒——外层序列扫描把实参位先完成的内嵌构造（`new; dup; …; invokespecial`）当作外层调用点，接收者检查报 "allocation did not produce"（X2 判别：单参作方法实参 ✓、throw 位 ✓、仅嵌套于构造实参位 ✗）。

## What Changes

- `init.rs` 构造站点走查接受实参位嵌套构造站点：外层实参扫描遇完整内嵌 `new; dup; …; invokespecial` 时递归证明（单用途、值恰为外层该位实参、ctor/类与内嵌一致），跳过其区间续扫外层序列；内嵌站点呈现为构造实参表达式（复用构造呈现，无第三份拼写）。
- X2.nested/X1.main 恢复且行为一致（`outer`/`inner` 回归）；单参作方法实参、throw 位、receiver 构造等既有形态逐字不变；内嵌非完整站点（缺 ctor、双用途、跨块、栈残留）保持拒绝。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：构造实参位的完整嵌套构造可呈现为实参表达式，嵌套构造链完整恢复。

## Impact

仅 `crates/jarde-java` 私有 `init.rs` 构造站点走查与对应呈现及测试；无新机制（递归复用同走查）。既有构造切片（receiver、lambda、匿名、varargs ctor）零回退。
