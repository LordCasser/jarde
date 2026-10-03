## Why

[assert 巡查](../../evidence/java-syntax-2026-10-03/assert-stmt-patrol/README.md)确认：assert 降低恢复完整且行为一致（`-ea`/无 `-ea` 两态），但 javac 合成物三层显式（`$assertionsDisabled` 字段、`<clinit>` desiredAssertionStatus 行、`if (!…)` 守卫）——源码形状 `assert cond : msg;` 未呈现。与 val$/access$ 同族合成识别缺口，可编可运行故属源码忠实度片。

## What Changes

- 合成模式识别回写：字段（名字 `$assertionsDisabled` + ACC_SYNTHETIC + `<clinit>` 初始化形 `desiredAssertionStatus() ? 0 : 1`）与 clinit 行隐藏；使用点 `if (!X.$assertionsDisabled) { if (!cond) throw new AssertionError[(msg)]; }` 折叠为 `assert cond [: msg];`。
- A1 家族呈现 assert 语句、合成物消隐；无 assert 类与识别失败形态（守卫内额外语句/字段被引用）保持现呈现逐字不变；`-ea`/无 `-ea` 行为两态一致。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：assert 降低按 `assert` 语句呈现，javac 合成守卫物消隐。

## Impact

呈现/装配层（模式识别+回写）及测试；无证明层改动。既有 `<clinit>`/字段呈现零回退。

> **2026-10-04 root 账本核对**：本片与既有 [project-proved-assert-statements](../project-proved-assert-statements/)（2026-09-26 立，范围高度重叠）构成立项重复——本片立项前未先查既有 change，属流程失误（与同日 bridge 域先查后立、避免重复的做法相反）。功能已交付并验收，不回滚；既有 change 的残留范围已在其 tasks.md 收窄记录（预算/取消测试、定向测试粒度、root 验收）。教训已记入 handoff.md：立新 spec 前必须先 `ls openspec/changes | grep -i <域>` 并读同域 change 的 proposal 与 tasks 状态。
