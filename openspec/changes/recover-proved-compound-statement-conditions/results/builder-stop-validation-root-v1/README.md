真实阶段测试失败记录：v1编译失败7项(E0277 ValueRenderFailure无Debug、E0308 RegionPath必须Vec)；只改测试后v2编译成功但0/1。实际prepare success usage52，limit51 Stop@59及状态哨兵回滚通过；emission prepared52/total52且没有条件工作，是测试截断regions导致declarations早期wholequote，不是新emitter成功。root要求完整recovered.regions与真实path[0]修正，不能清除incomplete绕过验证。两轮target均clean；2.5尚未接受。

root v3实际unit1/0通过：prepare usage52/limit51、emission prepared52/total68/limit67，两次Stop为AnalysisSteps/written0@59，试验状态回滚。public整目标7/1，新增预算测试误断言written0；真实Stop usage516/limit516/written633@7，生产停止保留实际output_bytes。没有改Stop API；失败完整保留，不能把此轮整体验收写为通过。取消测试是prepare后、进入emitter前触发，不是异步中途注入。

v4原私有精确pin未format，root fmt --check实际exit1，未执行library测试；完整记录保留于format-v4-failed。root随后格式化，新一轮full library尚在执行。

root实际修订后public三目标2/8/2通过：success517、limit516、stop usage516/written633@7（写入量同Budget.output_bytes），binding/text/map全无partial，取消和既有scope/boundary均通过；targetclean。完整库348/0/0包含v4精准Builder51@59/67@59，用量与状态回滚通过，记录在edge-negative-validation-root-v1/observed-lib-v2。此时尚未完成全部旧loop/value回归与productCI，不关闭2.5。
