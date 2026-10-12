## Context

见proposal.md。root同次trace记录protected range[0,47)、handler50、join59。`region_at(0, protected_frame)`返回Straight[0]和next7；view中自然环7的blocks[7,16]、latch16、exit47，非irreducible。try最终body0/catch50，末尾59恢复后未owned[7,16,47]。trace从未调用region_at(7)，不能把它预期返回Loop当成观察。

当前`try_level`普通路径只调用一次region_at，body_next仅尝试Switch专用的range-end证书；`split`把run的首个region放入body，其余放到Try后的tails。因此仅增加一次walk仍不足以保留catch语义。`build::try_region`已可通过arm递归呈现Sequence；normal_exit_bci只是Try来源的derived anchor，不是控制流证明。

补充[同次子调用诊断](../../evidence/java-syntax-2026-10-13/outer-try-foreach-child-probe-root-v1/README.md)真实调用region_at(7, protected_frame)：返回Fallback(ExceptionEdge BCI7/ordinal0)、next=None、visited无新增，六个Walker状态恢复，原uncovered仍[7,16,47]。loop body frame未到达。现有header异常门只接受finally等证书；Frame::loop_body也清空own_try。后者是源码事实，不能称为该诊断的运行观察。

## Goals / Non-Goals

**Goals:** 在普通单range路径证明并组合一个Straight前缀和一个已识别Loop，保留完整保护范围、唯一所有权和纯transfer的词法落点。使用同次canonical/view/SSA及现有frame，目标由字节码形状决定，不按fixture或方法名匹配。

**Non-Goals:** 不改normal-flow建图，不新pass/solver/public carrier；不普遍把任意structured run收进try，不扩大多loop、nested try、透明finally、retry或fragmented规则，不改handler泛型投影。本项与LVT引用身份修复分别审查，共同接受完整类。

## Decisions

1. **只在已有ordinary入口接回明确的候选。** 排除inner/transparent/retry/fragmented特化；只有原run为非空Straight前缀、next为同保护range内的未visited自然loop入口且frame允许时，才在protected_frame基础上绑定下述scope、尝试region_at(next)。这是一段前缀加一个loop的有限组合，不增加任意continuation遍历。子结果必须是完整、已识别Loop（或仅包装此形态的现有Sequence），有进展、无重复ownership；全部body指令仍在range内，保留既有fused lead例外。若实际loop reader拒绝，保留拒绝并分析真实原因，不猜测扩大loop规则。

   **精确named row事实是必要的最小补充。** own_try只存canonical节点，fused block及同起点不同结束range可共享该节点，无法据此无歧义反推当前Catches.sites。仅在此有限候选创建一个可选的private Frame named_catch_rows事实：从当前[shape.lead.1, shape.protected_end)及Named sites生成(type, canonical handler)多重集合；一次有界扫描实际handler rows/映射，一一匹配，保存准确ordinal集合。缺失、重复、catch-all、额外/不同范围row均不能借当前clause。表、SSA和canonical不可变，集合用标准库共享不可变容器随frame传播，避免每次frame.clone复制整集合；不新增pass、solver或全局索引。

   header独立异常门仅在这个事实存在且所有takeable边通过原物理edge accounting时放行；loop body才可继承该事实及原own_try。arm继承同scope，protected/nested try清除它；本候选遇到nested Try/Guard或额外活动row则拒绝，不引入scope stack。原generic loop及finally/resource/monitor/retry/fragmented路径不获得该事实。原edges_accounted_by_catches在有此事实时另核ordinal归属，仍保留takeability、真实handler映射、block/range末端、纯transfer、call/subroutine拒绝。前缀和完整loop的所有takeable物理异常边都须归该集合，不能只检查header后便把遗漏边隐藏在Sequence中。新增扫描及集合构造逐项poll/charge，停止不发布部分scope。

2. **完整结果作为一个body，bridge单独留作兄弟。** 合格的prefix+Loop通过sequence_region进入Try.body，不经过把Loop拆成tail的普通split。出口必须等于exclusive protected_end，并经过原certified_try_exit物理证明。成功后只把纯bridge region放入Try后的tails，normal_exit_bci取其BCI用于来源；catch和join沿旧路径恢复。原单Switch证书及它的body/tail行为保持。现有certified变量仅属于fragmented证明，不能借它表示新分支成功。

3. **扩展语法形态门，保留物理闭合门。** 证书只新增明确Straight+Loop（及等价单Sequence包装）形态，不改为任意is_structured集合。保持单Transfer instruction、准确range-end BCI、唯一join后继、无其他row保护/handler角色、非boundary/switch/loop/control target、bridge未visited、所有前驱owned、body指令范围和closed_transfer_edges。后者核canonical异常/call边；只有view predecessors不够。识别Loop还必须结构完整，不能包装fallback。

4. **尝试失败恢复所有权，中止不发布部分结果。** 沿附近现有组合分支的checkpoint，核visited、unclosed_tail_at、catch_joins、fragmented等可能被子walk修改的状态；失败或Stop恢复尝试前状态，计费不回退。guard对own_monitor的进出以及region_at的depth复位沿现有机制，审查时确认无漏状态。不新增通用checkpoint框架。新增复制/遍历/证明在既有维度预付或逐项poll/charge，禁止未计费二次全图扫描。

5. **参考JADX的组合层，不照搬可变图策略。** 已读本地JADX的`RegionMaker.makeRegion/traverse`：每次traverse返回next，并在region stack的exit处停；`ProcessTryCatchRegions.wrapBlocks`把选定子容器包成完整try region并调整parents。Jarde的等价落点是已有Sequence和same-frame子walk，保持canonical不可变、唯一ownership。JADX允许重复块并添加DUPLICATED警告的策略不适合本项目不变量，无需复制源码或新增依赖。

## Risks / Trade-offs

- header已观察ExceptionEdge拒绝，body丢owner尚属未到达路径 → 候选必须实际核两层和完整class，不把自然loop事实当成reader成功，不降低完整类门。
- 错把另一named row归入当前Try → row/type/handler/range一一绑定；加入同handler不同range、额外活动named row、catch-all及未设置scope的循环负控制。
- 有goto但异常范围不同 → 原物理rows/edges和range检查继续拒绝，测试构造竞争入口及效果反例。
- 子walk改变全局状态而失败 → 核实际可变字段与已有checkpoint，永久反例对比失败前后，保留Stop传播。
- body正确但被split搬到try外 → 树级测试断言Try.body为prefix+Loop Sequence，全部protected blocks只owned一次；运行抛异常的前缀、条件、取值和body。
- 只验证当前handler的body异常 → 在未改写完整类上增加独立harness，对iterator()/hasNext()/next()/output.add分别触发RuntimeException及Error；原17场景仍原样保留。

## Migration Plan

root固化同次trace和设计后，Luna仅私有准备patch、永久测试及harness；root审查后独占共享源码/Cargo/JDK/JADX/CLI。至少5GiB可用、target1GiB上限、1秒进程组守卫、default Rust stack、finally exact cargo clean。与引用身份修复的实际最终代码一起接受完整class/runtime，再走fmt/CIClippy/strict、fresh双seed workspace、精确产品推送/自身CI/真实HEAD cleanmain；失败保留原始证据。
