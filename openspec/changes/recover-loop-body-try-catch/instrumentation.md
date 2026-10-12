# 循环体 catch 的同次 IR 分析

当前 `ca45c0e` 的问题分成两个落点：普通 catch 认证不接纳 try 内的终止 throw 分支；foreach 的区域已经恢复，但 try 外写入的数组局部被词法呈现门拒绝。先把这两个事实分开，才能判断要补什么证明。

取证来自 [当前完整类基线](results/baseline-cli-v4-root-v1/README.md) 和 [同次 IR 原始流](results/same-run-ir-root-v3/README.md)。debug/none、Java8/23 共四个 lane；每个 lane 在同一个方法分析 payload 上取得 Code/canonical/normal-flow/SSA，再运行现有恢复。四方法共 16 次，physical class digest/length/name/descriptor 与原 CLI 配对。插桩使用最小 MethodFacts（参数槽、访问标志），没有模拟 CLI 的 debug-name 元数据；它证明实际物理/SSA 门，不能冒充 CLI 完整来源请求。

## Q-i：plainFor 在哪项拒绝

`DV.plainFor([I)I` 的 row0 是 `[10,32) → handler35`，具名 `IllegalArgumentException`。handler35 无普通前驱，只有一个后继 join39。header4 支配 join39；join 在自然环里且不是 header。声明行与 canonical 行完整匹配。

自然环成员实际为 `{4,10,26,35,39}`，包含 handler35，但不包含终止分支 block16。try 范围重叠 block10、16、26，block16 被 header4 支配，末指令为 throw25；new16/ldc20/constructor22/throw25 都保留 row0 的实际 throw-site。

因此 full-table-range 的 `loop member` 在 block16 为 false；`plain_paths_reach_join` 在 block10、16 为 false，block26 为 true。block10 的正常分支可以走到终止 throw，而该 helper 要求所有普通路径先到 join。现有 `proved_loop_catch_joins` 返回空集合。`whileLoop` 同样落在这两道门。这里不能把“普通自然环不含 terminal 分支”等同于“Java 循环体不拥有该分支”。不过，也不能放行任意 terminal；它必须归属于完整保护行，并由现有区域行走准确放在 try 内。

`DF.loopCatch` 相同：header10、handler45、join50，保护行 `[22,42)`，终止 block27 不在自然环 `{10,16,37,45,50}`，throw36 的普通路径不会到 join50。

## Q-ii：为什么被说成不可归约

`DV` 的普通 SCC 是 `{4,10,26,39}`，外部普通入口为 `0→4` 和 `35→39`。后者来自异常根 handler 的普通续接，所以初始多入口检查拒绝；原异常边包括 `10→35`、`26→35`。terminal block16 自己的异常边也保留在完整 dump 中。`DF` 的 SCC `{10,16,37,50}` 则有 `0→10`、`45→50`；其 row0 throw-site 在 SCC 外 block27，故 SCC 邻接异常边列表为空，这不代表方法没有异常边。

实测仅向 `irreducible_blocks` 传入假设的 handler→join 豁免，DV `(35,39)` / DF `(45,50)` 的不可归约集合就变空。原 CFG 没有删边，`try` 行走仍需认领 handler 与 join。这个实验定位了缺失的认证，不构成放行许可。实现优先补现有 `proved_loop_catch_joins` 的 terminal-body 所有权条件，不新增第二 CFG 或全局循环修复 pass。

## Q-iii：foreach 是另一道门

`DV.forEach` 的 row0 为 `[22,27)→30`，catch Exception；header10、join35，自然环 `{10,16,35,30}`。现有 catch join 认证实际得到 `(30,35)`，传入后不可归约集合为空；当前 CLI 也已经有结构化循环。canonical row 的 protected block16 在此没有范围内实际 throw-site，不能由 canonical 保守 Exception edge 断言 `iaload19` 在 try 内；完整物理范围仍是 `[22,27)`。

随后 `build` 的 local2 拒绝：`aload0@2 → astore2@3` 生成数组引用 Value15/局部 Value16；读出为 BCI4（arraylength 前）和 BCI16（iaload 前）。源路径 `[0]`，try 路径 `[1,0]`；引用类型 `int[]`、同 block、producer 在 store 前、非 quoted、store type 都已证明，但 `presented_reference_store_value` 要求 producer 在 protected try body，实测为 false。因而 `all_reads_reach_presented_writes=false`，local2 被解释性拒绝。首个 enumswitch warning 是探测信息，不能拿它冒充实际失败阶段。

要在原任务 2.3 补 try 外已呈现写的实际 SSA/词法支配证明：所有读须到达同一已呈现写，类型和作用域闭合，不能猜测赋值或新增 blanket reference 放行。普通 catch join 修复不能关闭这一项。

## 仍需验证的方案边界

无限 retry/break 代表已按 JADX `TestTryCatchInLoop` 算法准备五个事件 oracle，当前整类采集仍待独立接受；不能让有限 for 成功替代它。它可能涉及 handler 直接回到 body/header 的循环所有权，须由实际 Code/CFG 决定，当前尚未定方案。

设计需同时检查 terminal throw 的异常派发、完整范围/行重叠/普通 handler 入口、循环测试与更新不被吞进 try、已有 try emitter 对 terminal 分支的认领、foreach producer/consumer 的全路径与停止计费。保持原 CFG、来源和 exception row，不用 hypothetical 结果绕过这些条件。`design.md` 尚未完成，任务 1.2 仍开放。
