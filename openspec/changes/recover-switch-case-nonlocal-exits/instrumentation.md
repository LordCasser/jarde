# Q-i / Q-ii：实际同次恢复探针

## 输入与执行

产品来源81b0684d5、文档head3deab286e；正式605 CLI红基线见results/baseline-root-v1。root审查Luna私有日志补丁后，只临时修改region.rs与build.rs，实际守卫构建诊断CLI，并对同一八个class输入各执行all模式。每个请求的整个非usage JSON与正式CLI逐字结构相同。26份原始流pin复核；默认栈、5GiB/target1GiB/每秒守卫通过，构建峰值372076204B，exact cargo clean成功，最终两源SHA精确恢复且target不存在。完整canonical/SSA/Region、探针源码和恢复收据见results/first-refusal-root-v1；这是诊断构建，不能作为产品或产品CI。

## Q-i：switch 与 if 的差别

双编译器具有相同结构事实，以下BCI来自本次真实输入。

1. **SN两个case-return。** switch指令24在canonical块16，case3到44，default到49。循环体scope仅`[10,16,49]`，出口集合有正常尾60（有尾形为63）和return叶44，break目标为正常尾。postdom与forward join均None。case3的arm实际为空Straight，44未claim，最终单独UncoveredBlocks；local1在44的读取落在fallback path `[3]`。相反if控制的return在循环树内正常呈现。源码`loop_terminal_returns`只从双successor comparison接纳单入口、无出口、decoded Return叶；switch未通过其comparison入口，因而已有返回叶片证明没有进入switch来源。这不是保存值或SSA类型先失败。
2. **SN.labBreak / SL.labeledBreak / LD.lab。** switch24的postdom58正是外层循环break目标；scope为`[10,16,44,47]`，case5到44再goto58，default47回10。`switch_loop_join`实际返回ContinueTarget、candidates空、shared_tail=false，但`switch_continue`为None（postdom不是continue），当前消费把join置None后返回SwitchShape。拒绝发生于arm walk之前。不能直接删掉该拒绝：原ContinueTarget的无candidate回退并没有独立遍历并证明所有路径闭合，必须证明到现有精确loop target的路径再接纳无局部join。
3. **SL.mixed。** case1/2/3/default入口52/55/58/63；case1到68，case2到71，case3直接return，default63到68，68到71，71回10。postdom/forward join均None。现scope不含return58；第一arm实际claim52/68/71，第二arm经55又遇71并产生子Loop fallback，随后SwitchArmsOverlap。正确候选局部join是68，continue目标71应保持独立；不能把两者合并或把正常经过68→71的路径都输出成continue。须先证明返回叶，再证明局部join/非局部完成和每个arm的边界。

## Q-ii：与 local-scope 的关系

本次build.rs实际日志确认，两族都经过同一个`declarations`后置门：局部访问跨region，且其path在`RegionPaths.fallbacks`中，因而产生`local crosses a quoted fallback region`。SN的local1同时有入口定义、循环default更新、正常尾读取和未呈现return44读取；fallback确实属于该值切片，不能跳过守卫或按诊断字符串消音。

本族异常表均为空；上游分别是返回叶scope、switch join准入及arm尾所有权。已有`preserve-local-scope-across-exception-regions`的作用域拒绝规则应保持，产品修复独立放在Region已有证明与消费接缝，无须合并两个change或新增声明系统。没有证据要求公共AST/Region变体、通用solver或新的pass。

## 验收边界

本次只关闭真实红基线和首拒取证。后续仍须证明外来入口/错目标/异常边/跨case/重复所有权拒绝，真实晚停止无部分发布，完整四类源码重编与独立驱动行为、SM正文零回退、fresh workspace和精确产品自身CI。JADX错误输出保留为反例，不作为语义oracle。
