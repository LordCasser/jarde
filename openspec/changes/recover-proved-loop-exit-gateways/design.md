## Context

见 [proposal.md](proposal.md)及[CF-08 最小回放](../../evidence/java-syntax-2026-09-27/cf08-endless-loops/report.md)。`EndlessInts.find` 的自然循环块是 BCI 2/10/18；头部失败走 BCI 7 `goto 24`，体内命中走 BCI 15 `goto 24`，BCI 24 才是共同的循环后继。当前 `header_tested_loop` 把 BCI 7 作为 `Region::Loop.exit` 和 `LoopTarget.break_target`；体内 BCI 15 到 24 的真实边因目标不是 7 而不能获得 `LoopBreak` 归属，成为 `UncoveredBlocks`。现有 `loop_exit_bridge`、`Frame.loop_body`、`LoopTarget`、区域覆盖检查与来源记录均可复用。

## Goals / Non-Goals

**Goals:** 对头部纯转接出口和一个体内纯转接出口证明同一最终后继；只在证明闭合时把体内网关纳入循环体走访，写成指向该循环的退出，并保持头部失败边及循环后的代码各一次。保持预算、来源与拒绝原子性。

**Non-Goals:** 多入口/不可约循环、带效果的出口块、异常区、共享出口的跨循环重排、`TestNotIndexedLoop` 的外层 `if` 与循环后 slot 2 汇合、CF-07 的返回叶，或强制把 `while` 改写成 `for`。其它 CF-08 形态继续巡查。

## Decisions

1. **证明物理网关与最终目标，再决定 Java 出口。** 只在头部失败边到达一个无效果、不可抛、仅含直接 `goto` 的正常出口网关，且体内比较分支到达另一个满足同样约束的网关时，核对两个网关的唯一正常后继为同一块。网关的普通入边必须分别且仅来自已证明的头部/体内分支；排除异常、子例程、额外正常入口和未解码指令。沿 canonical CFG 与 SSA 已有事实核对指令、目标和使用，不按 BCI 邻近或后支配点猜测。可借鉴 JADX `LoopRegionMaker` 分开收集 loop condition exit 与额外 exit edge 的调查顺序，但不用它的块复制或启发式插 break 代替物理证明。
2. **让 LoopTarget 表示实际 break 目标，保留原始正常出口。** `Region::Loop.exit` 仍持有头部真实失败后继 BCI 7；候选通过后，既有 `LoopTarget.break_target` 指向共同后继 BCI 24。只把已证明的体内网关 BCI 15 放进本次 `Frame.loop_body` 可走访范围，使原有 `loop_exit_bridge`/`Region::LoopBreak` 路径认领它；循环体覆盖必须实际访问网关一次，外层继续从 BCI 7 到 BCI 24。若实现发现现有 Region 表示无法同时保留这两个身份，应先收紧拒绝并回报设计，而不是改写全局 CFG 或放宽词法计划。
3. **发布前检查两条路径和来源。** 体内分支的 taken/fall-through 极性保持原样；循环体、头部正常出口与共同后继不重叠拥有。来源须包含 BCI 4、7、12、15、18、21、24/25 的对应条件、转移、更新及返回；任何候选网关未被结构实际消费或出现额外所有者，整方法继续引用。枚举与核边按现有预算计费，取消直接传播。
4. **不增依赖和中间控制流层。** 当前 reader/JVM IR 已提供 decoded transfer、normal/exception edge、SSA 与区域事实；第三方 CFG 库不能代替本项目的预算、物理来源和 Region 所有权合同。新增私有有界证书或参数即可，无需新 crate、公共 pass、AST 或 Region 枚举。

## Risks / Trade-offs

- [把共享后继误当某个循环出口] → 同时证明两条网关的全部入边、直接目标与所属循环；多入口/嵌套或异常路径拒绝。
- [只把网关加入 scope 却没发出 break] → 覆盖与最终结构消费同时核对，保留 `@bytecode` 和原因，不发布遗漏出口的完整方法。
- [头部失败边被误写成体内 break 或循环后继被双重执行] → `Region::Loop.exit` 与 `LoopTarget.break_target` 分别保留物理网关/最终目标，以六个输入的三方验证运行和 BCI 来源检查验收。
- [负例或低预算导致半个循环已发表] → 证书在修改访问集合前完成，沿现有 stop/atomic fallback 路径处理不完整证明。
