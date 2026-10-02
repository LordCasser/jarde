## Why

[do-while return 巡查](../../evidence/java-syntax-2026-10-02/do-while-return-patrol/README.md)实锤独立缺陷族：do-while(false) 体内的 return 边不被区域树拥有——无副作用形态产出**静默错编**（L5：引注区含 return、剥引注后类仍可编译、重编 `i=10`≠原类 `early`），有副作用形态整方法拒绝（D1）。静默错编属最危险类缺陷（下游使用者无从察觉），本片承担恢复与失败闭合双义务。

## What Changes

- **恢复**：do-while(false) 降低体的语句集（含条件 return/break 出边）由该区域拥有，按 `do { … } while (false);` 呈现；return 边与循环 break 边按 dj 切片的语义归类各自证明。
- **失败闭合（新不变量）**：区域证明失败时整方法拒绝——呈现层不得产出"引注区包含控制流改变语句且剥离引注后仍可编译"的部分体（在引注落点增加该判定：引注区内含 return/throw/break/continue 出边时升级为方法级拒绝）。
- L5.dblJumpDoWhilePlain 恢复（`early`）或至少安全拒绝；D1.retInDoWhile 恢复（`early:5`）；既有 do-while、dj 双跳转、return-in-if/for 形态逐字不变。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：do-while(false) 体含 return 可恢复呈现；引注失败闭合（控制流语句不进部分可编译体）。

## Impact

`crates/jarde-java`（region 归属 + 引注落点的失败闭合判定）及测试；dj 边归类复用，无新机制。既有 do-while/dj/return 形态零回退。
