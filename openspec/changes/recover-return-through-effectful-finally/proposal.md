## Why

2026-10-05 patrol发现单return穿副作用finally的源码恢复缺口。return表达式必须恰好求值一次，先保存结果，再执行finally；FO的`try{return bump();}finally{i=100;}`以`1/100`区分次序。历史矩阵留在[原巡查](../../evidence/java-syntax-2026-10-05/return-effectful-finally-patrol/README.md)，其中“finally返回值形已恢复”不能直接作为当前整类可编译的结论。

此前产品`30afa7c9ab2818f41959f9ea6a8ca6f896714dc1`与冻结CLI `05e0272e…b89d`已完成五个独立叶片：FO静态前置求值、FY.loopless条件return副本、FY.noCall局部iinc循环、FX.finReturn调用结果循环，以及FX.finContinue/finBreak残留handler45。最终两编译器来源FY/FX/FO共6完整类、32方法均Java/Structured且无fallback；六份生成源码原样重编通过，12次rebuilt-only main输出逐字等原oracle。详细事实和限制见[instrumentation.md](instrumentation.md)，不由历史诊断同名推断共同原因。

父计划负责汇总这些可证叶片、完整物理来源、对照与回归验收，并继续跟踪已经观察到的其它未证明正向形，不能仅凭固定FY/FX/FO成功收窄原有单return穿副作用finally范围。EX.finThrow、EX.nestedFin已由两个有界叶片在组合产品4c6852中恢复并验收；FinallyCompletion.tryThrowsFinallyRuns是仅异常完成的相邻正向欠账，由后续独立变更承接；EX.finReturn及FinallyCompletion的finally自身return/throw则是独立拒绝边界。这些范围外方法仍有fallback，FinallyCompletion完整源码仍missing return；实际红项继续登记，不把固定FY/FX/FO成功扩称任意finally或任意JAR/WAR完成。

## What Changes

- 复用既有Finally/SavedReturn与私有continuation，按实际CFG、SSA身份、effects、保护区间和出口证明有界叶片；不另建通用temp框架。
- 将历史Q-i定位到loopless真实首拒与融合尾块；将Q-ii分解为noCall的iinc/handler phi与finReturn的保存调用结果/多调用cleanup，保存各自证明和拒绝边界。
- 按固定FY/FX/FO的所有物理方法与BCI核来源，并对完整生成类原样重编/运行。固定main覆盖的观察与独立driver观察分别记账。
- 对照保持行为、求值顺序、来源和拒绝边界。continue/break这次授权改善会删除旧fallback，旧“逐字节不变”不能约束它们保持失败文本；前一CallReturn版本至当前的28个非目标方法记录须逐字不变，未来以当前30afa为新的完整对照基线。
- 本父已按实际Facts/SSA分别恢复EX.finThrow的try分支主动抛错与EX.nestedFin的嵌套落穿清理；FinallyCompletion.tryThrowsFinallyRuns的仅异常完成另立有界后续变更，保持红项跟踪；finally内显式return/throw等未证明非局部完成继续保留可定位拒绝。不把未改变的fallback称为成功恢复。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：可证明的return穿副作用finally恢复与父级完整源码验收，保留求值序、副作用次序、物理来源与拒绝边界。

## Impact

实现位于`jarde-java`既有guard、region/Builder与来源呈现层，公库和CLI只沿现有入口消费。五个已归档叶片及新增finThrow/nestedFin分别维护永久测试、真实双编译器fixture、停止/负控制及自身CI。父级不重复实现已交付叶片，也不修改原Java、driver、oracle、生成Java或旧计费基线。

## Final product evidence

组合产品 `4c6852ac8c4ab9ce88f0716f4d2bd702e15684c5` 的同一冻结4811eea2 CLI已复验FY/FX/FO全部32方法与固定oracle，完成nested432及finThrow144个重建case。产品自身CI39042611463的4 jobs / 52 steps及两seed各3691/0/98已由root独立接受；最终本地workspace与父文档闭合见tasks及instrumentation，不以本节取代收据。完整EX原样编译已成功，finReturn仍为未改变的fallback；FinallyCompletion三处红项未隐藏。
