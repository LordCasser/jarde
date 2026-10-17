# Proposal

## Why

最终CallReturn CLI已恢复FX.finReturn，原完整FX两编译器重编/固定main运行均通过，但finContinue与finBreak仍把catch-all handler45作为未覆盖bytecode注释交付。固定main没有触发该异常路径，不能用运行匹配替代其原始异常范围和值流证明。

## What Changes

- 固化原FX两腿finContinue/finBreak的61B Code、三条catch-all rows、唯一正常transfer目标差异及最终CLI红基线；先取得实际canonical/SSA/effect与Region，不从CLI摘要重建表。
- 对这组int数组循环、局部加法及三份相同局部+100清理建立有界私有证明，完整核对正常continue/break、非transfer体、异常保存/清理/原值重抛和try外数组读取。只有证明成功才消除handler未覆盖；不能简单把该块标为visited。
- 明确检查受保护操作是否全部nonthrowing以及真实异常边/SSA来源。若拟利用不可抛异常结论，必须独立证明所有物理操作effects、完整异常范围与handler值链，并保留来源，不能仅凭没有正常前驱或固定样例没抛异常省略handler。
- 增加真实class近邻拒绝与证明期停止、公库/CLI完整平面及全部物理BCI来源；最终完整类原样重编和未改driver rebuilt-only双JVM对照，另保留原FX/FY/FO和JADX完整源码观察。
- 前置：依赖CallReturn产品464543594的既有能力；其自身CI、正常归档及main b12b0d9e交付已完成，本片已rebase，不借其CI验收新片。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 已证明的循环continue/break清理及catch-all重抛handler能完整交付Java，并保持真实异常边界、值流、停止和来源。

## Impact

候选改动限jarde-java的guard/region既有证书与接线、对应测试、真实fixture census/指纹及OpenSpec证据；不新增依赖、公开IR/AST种类、通用handler框架或循环注册机制。原named-catch拒绝catch-all的契约继续成立。任意可抛调用、monitor、资源、不同清理或跨范围handler不在本片；不声称任意finally/JAR/WAR或父计划已完成，不调整旧P5计费基线。
