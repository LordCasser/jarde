# c075 自身 CI 的原始计费失败

产品 c075d1a15ab9e29482ee2eb006bc152e5c57980c 的 run 38838653033 实际 failure。原始 API 与 466118B ZIP 全部保留；ZIP 实际45个安全唯一成员，不假定提供方固定布局。stable 首seed在 P5 bulk 两项计费快照失败：flat-mixed AnalysisSteps1549对1548，旧direct/shared总20304对20290。其它打印维度相同。后继pass2/JDK比较/strict等均skipped，不算执行通过。MSRV/supply-chain/fuzz三个job成功不能替代失败的stable。

实际workspace汇总253条（252非空加1条零测试），1941通过、2失败、55忽略；新七项prefix测试未执行。独立失败审计283项检查/0错误，接受的是失败事实，不是产品门通过。

后继修复仅在计费SSA proof之前使用固定六opcode候选筛选，不改旧P5表/断言，不退款。八个旧P5 class中12个六指令int方法均不匹配完整候选；仅首opcode筛选不足以排除NestedEval。独立分类器原报告另附。

capture.tar.gz 为修正元数据的v2原包：953124B、SHA256 9118dccf44589c7deaa57365637634377f5b923b1e8f74ab8df9b72b79b63d0d，159 payload加manifest共160成员。v1缓存原包保留，v2修正把253全部summary误称非空的元数据，并增补独审；原raw未删除。最终产品成功须另验后继a89自身CI。
