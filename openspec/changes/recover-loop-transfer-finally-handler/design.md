# Design

## Context

参见proposal的Why。原FX两腿finContinue/finBreak各61B Code，max_stack2/max_locals7、35个物理指令；只有BCI31的goto目标不同（continue到53，break到59）。三条catch-all rows为[22,28)->45、[34,39)->45、[45,47)->45。正常清理是28与39的iinc(local1,+100)；异常出口45 astore6、47 iinc1,+100、50 aload6、52 athrow。10为loop header、53为更新、59为最终返回；这些是实际class解码事实，不等同尚未取得的SSA证据。canonical原始Exception边与按throw_sites判定实际可触发边必须分别记录；现exception_edge_takeable的结果不构成本片认领证书。handler的保守phi/caught来源、save/reload/throw全部消费者仍须核对，三份同slot清理也不能机械要求共享同一SSA ID。

当前正常Region覆盖0/loop{10,16,28,34,53}/59；45因只在normal-flow之外可达而fallback。现named-catch/fragmented/loop-catch拒绝catch-all是正常边界，不能把它们泛化来接管本片。输入class/Code相同不证明phi、alias或handler可达性；第一项任务已经实际分析两腿并保存原表，完整矩阵见baseline包support/actual-matrix-independent-review-v1.json。CallReturn产品464543594自身CI38913819742已终态独审通过，正常归档后main b12b0d9e已交付；本片已实际rebase到它，4942行源与probe收据逐项不变。

## Goals / Non-Goals

**Goals:** 针对这对有限形态消除未证明handler，保持正常循环结构、全物理来源、原错误传播与证书/正文原子提交。新增单一私有证明，不增加通用语法或IR构造入口。

**Non-Goals:** 任意可抛body/handler、资源/monitor、多层finally、不同清理、任意循环及父计划全覆盖；不改原Java/driver/oracle、不把固定main或JADX输出当语义证明。

## Decisions

1. **先固定输入和真实IR，再接线。** 独立fixture从FX原两方法完整切片，仅外类名变化，保留原FX整类作回归；两真实编译器、单一未改driver固化多输入/溢出/null输出。临时私有probe直接读取该次P2发布的canonical/frames/SSA/effects与Region；不构造Facts、不从摘要重建，保存完整矩阵及输入/tool/source pins。

2. **有限admission后paid proof。** 静态大小/形态hint只能决定是否尝试，不能认领handler、修改scope或发布证书。proof检查所有操作/slot/值/phi/uses、完整边/row/实际covering handlers、局部更新值与转移目标、异常对象保存到同一值重抛。扫描/比较/增长与末尾commit检查按既有Ledger收费；失败及Stop不留下ownership，固定上限拒绝不冒充共享停止。

3. **异常出口不能仅靠normal-flow消失。** 原保护区看似只有局部/条件/int加法，但尚须核实际effects和完整表。采用现有Finally/转移结构保留try-finally语义是首选；若真实表证明全部受保护物理操作不可抛且无可达异常边，可用同一窄证书完整解释不可达handler并保留其来源。该优化必须同时闭合handler值链和三份清理，不能泛化“无正常前驱即不可达”。若证据不支持这两个等价表示，保持fallback并收窄/修订规划，不能直接mark visited。

4. **复用既有所有权和发射事务。** 仅接入本组guard/region证明和现有Builder/source-map接口；正常branch的continue/break目标必须分开核对，数组准备/元素load保持try外，tail索引更新一次。拒绝/停止时回滚本候选visited/scope且不发布部分正文。具体载体选择由任务1真实IR决定，但不得改变上述行为与接口边界。

5. **不用外部库替代值流证明。** 当前Rust分层已提供class reader、IR、Ledger、证书和AST/来源发射；标准容器与现有工具足够，无新增维护/许可依赖。JADX1.5.6仅固定对照：其完整FX目前因未定义th重编失败；不能修文本后把它说成原样成功，也不能以其失败证明jarde正确。

6. **验收分开来源和执行平面。** 公库与CLI断言Java/Structured/空fallback、完整方法及所有BCI真实方法identity；最终CLI两独立整类原样重编、未改driver rebuilt-only双JVM逐字对照。原body没有可观察同步throw，handler异常身份依赖静态证明；额外可抛变体是拒绝控制，不冒充原class动态覆盖。另对原FX/FY/FO所有方法和JAR/WAR真实加载根留完整结果。

## Risks / Trade-offs

- [物理handler保留与正常CFG不一致] → 实际IR矩阵先行，证书必须解释全部物理行/边/值链，不能用显示名/slot代替值。
- [等价正常表示掩盖可达异常] → no-throw必须逐操作effects证据；新增可抛操作、改变row或保存别值的真实class近邻不得被本规则认领。
- [继续与break共享大部分字节导致误合并] → 分别核31目标及循环/出口关系，独立多输入oracle。
- [新证明改变旧账单或错误优先级] → 保留旧P5与全部finally/loop兼容门，真实late预算/charge取消/期限检查，默认stack与5GiB free/1GiB target/exact clean。
- [固定样例成功被当成完整恢复] → 全text不删方法、不混原classpath；静态/运行/结构/CI分别记录，exact-head自身双seed通过后才归档。

## Migration Plan

完成实际基线和IR后先实现private proof，再最小接入公共恢复并做最终整类/双JVM对照；等待CallReturn主线交付后rebase并确认实际源码闭合。新产品提交冻结CLI/source/Git blob/mode并运行自己的CI。通过后追加本delta、正常归档、两版strict及main绑定；父计划未完成部分保留。

## Actual evidence decision

任务1取得的canonical.throw_sites仅5/19/52、handlers均空；全部三个物理保护区逐指令nonthrowing，保守canonical异常边仍三条。选择本组固定形态的不可触发handler证书，不把它呈现为活的catch/finally运行路径。正常28/39清理与continue/break不改，原始handler图/phi/来源不删。

后续实现以最小私有投影为界：先在原region walk/ownership/P3字节完整性检查完成后，对完整35操作、全部row/effect/throw-site及handler SSA和正常树作有界paid proof，保持原Fallback45待最终产物确认。Builder仍按既有树构建；只有正常AST完整、唯一For主锚BCI13（Region header10、init8、update53、gateway56）与唯一handler fallback45/47/50/52完全对应时，准备全部来源和ownership核对并在末次poll后原子移除该已证fallback、把四BCI派生到真实Loop语句；公共报告明确补充已证handler45 RegionDetails及原finally规则。若AST/来源检查不满足，保留原fallback/质量与树，不发布证书；真实Stop仍不交付部分产物。不增加公开Recovered字段或Region/AST变体，不伪造节点/空语句/网关来源，也不把该证书带入通用named-catch。
