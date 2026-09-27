## Context

隔离类 `EffectfulExits` 的 SHA-256 为 `2e27cffb9361cbd78a689b404457b68d6bc31f1739362659ce71c0e276c0bae8`。`pick` 的头比较 BCI 2–5：越界到 8，执行 `cost(7)` BCI 10、`istore_1` 13、`goto 35` 14；继续到 17，数组读与 `istore_1` 20，比较 BCI 23；命中到 26 `goto 35`；未命中到 29 `iinc`、32 `goto 2`；共同后继 35 读取结果并返回。`javap` 已确认 35 的正常入边来自 14/26；实现须从 Jarde 的 SSA 显式证明该点的局部值/φ，而不能把静态推断当成证书。

`loop_exit_bridge` 只接受独占单条无效果 `goto`；效果块 8–14 不符合。`header_tested_loop` 把 8 作为循环的正常出口，此时体内 `break` 若落到 8 就多执行 `cost`，若直达 35 就不符合现有 `break_target`。现有 `LoopForm` 仅有 While/DoWhile，builder 要求 `tests.first()`；`Frame.loop_body` 将头部设为边界，`loop_body_sequence` 在第一次头部访问就停止。JADX `LoopRegionMaker.makeEndlessLoop` 可参考其搜集公共出口的调查顺序；Jarde 不复制块或借用其启发式插 break。

## Goals / Non-Goals

**Goals:** 证明每次头比较都在循环体内原位求值，失败臂先执行一次 `cost` 再退出，命中臂直接退出；两臂只在 BCI 35 合流，局部结果在 Java 源码中合法且语义相同。所有区域及指令各自拥有一次，预算和取消保持原子停止。

**Non-Goals:** 额外入口/回边、异常 handler、嵌套循环或 switch 的共享出口、任意带效果转接块、JADX 的复制块机制、`TestNotIndexedLoop` 的外层 `if`/slot 2 汇合、普遍 `for` 推断。

## Decisions

1. **先证明表示可行，再扩展准入。** 用当前 Region/Frame/Builder 追踪首访 header、回边、两臂 `LoopBreak` 和唯一后续块；先写一个只读或拒绝式试验/定向测试。如果 walker 不能在单次有限改动中只放行首次头部、回边停止，或无法让头部分支成为 body 首个 `If`，本 change 停在拒绝并记录阻碍，不发布部分实现。不能通过放宽 `loop_exit_bridge` 或重排效果绕过这一门槛。
2. **证书按真实边和 SSA 建立。** 仅收单入口自然循环、一个纯回边、头部纯比较、头部效果臂独占正常入口/出口、体内独占纯 `goto` 出口，以及二者相同且唯一的正常目标。逐块检查完整 Code、canonical 正常及异常边、SSA 的 slot 1 定义与 BCI 35 的合流/唯一呈现消费；任何未解码指令、额外消费者或 handler 均拒绝。证明在修改 walker scope 之前完成，边/块/SSA 扫描按现有预算收费并 poll 取消。
3. **必要的最小表示是 Region 级 endless。** 证书闭合后，允许 `Region::Loop` 带明确 Endless form 和空头测试，body 从头部首访开始并包括效果臂与体内转移臂；回边到头部只停止当前迭代，不能再次展开。两臂各用既有 `LoopBreak` 指向 35。Builder 将 Endless 呈现为 `StmtKind::While { cond: true }`，不新增 AST；Header 的物理来源由体内 `If` 持有，循环语句只取派生锚点，`Region::blocks` 不重复认领 header。不要把已有 While/DoWhile 的空 tests 合法化。
4. **闭合后验收效果与来源。** BCI 5 的条件极性必须使越界臂执行 8/10/13/14，命中臂经 17/20/23/26 跳过它；29/32 只在未退出时发生。结果局部声明必须在循环外可用。来源覆盖 BCI 5、8/10/13/14、17/20/23/26、29/32、35–40，隐藏转移只作为相应结构的派生来源。`EffectfulExits` 三组输入的原 class/JADX/Jarde 完整源码必须以 Java 8 重编并经验证运行同为 `8:1 / 3:0 / 8:1`，且无 `@bytecode`。已有纯双网关、单出口及其他循环回归不能改变。

## Risks / Trade-offs

- [在体内 break 后错误执行头部效果块] → 两条出口分别拥有独立 `LoopBreak`，共同目标只在循环外访问一次；计数 runner 必须发现多执行。
- [header 首访/回边重复或漏读] → 首访放行与回边停止分开证明，并由区域覆盖检查拒绝重复归属。
- [slot 1 φ 或声明无法呈现] → 实际 SSA 检查和完整 Java 8 编译是发布门槛；不能只看 CFG/JADX 文本。
- [预算/异常边留下半个循环] → 先有界证明再改 scope，停止与拒绝保留物理 BCI，不输出声称完整的方法。
