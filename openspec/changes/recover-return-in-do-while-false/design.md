## Context

[巡查证据](../../evidence/java-syntax-2026-10-02/do-while-return-patrol/README.md)：L5（静默面）/D1（拒绝面）双 fixture。dj 切片刚建 `loop_break_transfer`/`loop_continue_bridge`/`loop_side_routes` 边归类与"到达自家 header 的边停止"归属。**第一个取证义务**：javap L5.dblJumpDoWhilePlain 与 D1.retInDoWhile 的块/边差异（副作用语句的有无如何改变区域计费与引注落点），读引注（方法级 fallback）的触发条件——失败闭合判定应落在"引注区内容分类"处而非每个区域特判。

## Goals / Non-Goals

**Goals:** 两面恢复；失败闭合不变量（引注区含控制流改变语句→方法级拒绝）全局生效并测试钉死。**Non-Goals:** do-while(false) 之外的引注内容审计（continue 目标外逃等更广闭合——按失败闭合不变量的自然外延处理，出现回归另片）；throw 形态（同判定覆盖，测试一形）；jsr/ret。

## Decisions

1. **恢复路径**：do-while(false) 区域拥有体语句集（含条件 return 出边），return 边归类为方法退出边（非循环边），呈现 `do { … } while (false);`；break 边既有归类复用。
2. **失败闭合**：引注区内容包含控制流改变出边（return/throw/break/continue 的目标不在已证呈现内）时，该方法升级为整方法拒绝（现部分体路径改为拒绝）——全局判定，一次实现；L5 形修前可编译静默错编 → 修后恢复或拒绝，二者皆通过行为差测试（不允许"可编译且行为不同"）。
3. **验收锚定**：L5 整类（`early`）、D1 整类（`early:5`/`d3`/`f2`）重编行为一致；retInPlainDo/retInIf 等既有 return 形态逐字不变；负例（闭合判定触发形——构造引注区含 return 且不可恢复的补丁类——保持整方法拒绝）。

## Risks / Trade-offs

- **闭合判定过宽**（既有合法部分体被拒）→ 判定严格限定"控制流改变出边"；全量既有 diff 断言守护（若既有测试出现合理部分体被拒，如实报告并以具体案例回到 root）。
- **恢复路径与 dj 归类交叠** → return 边不是循环边（目标=方法出口），与 break/continue 归类正交；测试双向。
