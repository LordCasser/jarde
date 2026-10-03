## Context

[巡查证据](../../evidence/java-syntax-2026-10-03/assert-stmt-patrol/README.md)：A1 降低形状（见 results/A1.txt）。**第一个取证义务**：javap A1 固定守卫块与 `<clinit>` 行的精确形状（有无消息两形：`new AssertionError()` vs `new AssertionError(msg)`、msg 的求值序）；读使用点 if 守卫的呈现位与 clinit 行/字段声明的隐藏挂点（init-block-return 片的语境过滤先例）。

## Goals / Non-Goals

**Goals:** 两形 assert 回写；合成物消隐；两态行为一致。**Non-Goals:** 守卫内额外语句（手工字节码）；多类共享静态断言开关（非法形态）；`-ea` 语义改变（只改呈现）。

## Decisions

1. **模式判据（结构）**：字段名 `$assertionsDisabled` ∧ ACC_SYNTHETIC ∧ `<clinit>` 单行初始化 `X.class.desiredAssertionStatus() ? 0 : 1` ∧ 使用点恰为守卫形（if !field { if !cond throw AssertionError[(msg)] } 且无 else/额外语句）→ 回写；msg 求值序保持（抛出前求值）。
2. **消隐语境**：字段+clinit 行在全部使用点被回写后隐藏（有任一未回写守卫则字段保留）；沿 init-block 片的语境过滤机械。
3. **验收锚定**：A1（两形+嵌套类独立字段）+ 变体（msg 为方法调用副作用、嵌套类与外围同名字段）；负例（守卫含额外语句保持现呈现、无 assert 类零变化）。

## Risks / Trade-offs

- **msg 副作用求值序** → 回写后 `assert cond : msg;` 的 msg 仅在断言失败时求值（与原一致）；非平凡 msg（多语句拼接降低）保守不回写。
