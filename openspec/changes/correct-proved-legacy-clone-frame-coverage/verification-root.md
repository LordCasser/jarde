# Root 验收

当前5/6；完整workspace v4 JVM243/Java351、最终Clippy及原CLI观察已接受。父片产品 e041b7635 的自身 CI 38116415118 在 overdeep lambda helper stack overflow/SIGABRT，2.4继续未完成，等待独立 bound-arithmetic-render-stack-frames 修复后的新产品验收。

## 历史验证记录

实际 v2 格式化、jarde-jvm 全库 243/0/0、jarde-java 全库 350/0/0，5GiB free/target1GiB 守卫未触发，source pins 恒同，cargo clean 成功。新九项 canonical 测试覆盖共享普通分支、nested Return、handler Exception、明确 dead seed、断开/自环/错路径/未覆盖 Return 以及新增存储/工作预算和收费后的取消。ordinary 图只复用原邻接，新增闭包在 clones>0 执行。

真实双 JDK 已验证的完整 class 经 Reader 六阶段全部 Completed，实际 CanonicalCFG/Frame/SSA 可取得，两套克隆来源保留；region 因跨调用路径 join 拒绝且没有消费泄漏。Jarde verification 保持 NotPerformed。不是扩大 Java 恢复或完整 verifier 声明。

私有 v1/v2 静态问题、第一轮 Java test-only 类型比较编译失败和最终通过均保留。前端错误属于独立 change，没有将 ReturnAddress/Frame 改造混入父片。证据：[真实回归](results/jvm-java-validation-root-v1/root-acceptance.json)、[实际 source/raw](results/jvm-java-validation-root-v1/observed-attempt-v2/execution.json)。最终 workspace/lint、CLI 对照、自身 CI、提交和干净交付仍待父片闭合，任务2.4不勾选。

完整workspace第一轮到P2golden在两测试失败，实际九API捕获逐字段确认只有新增coverage IrItems/AnalysisSteps变化：45–48各+12/+32，shared+10/+27，49–52完全不变。只更新20个数值位置（report/expect各两维×五legacy），所有请求/limits/来源/阶段/诊断/quality/其余usage恒同。原永久golden测试文件字节恢复，永久9/0/0已实际通过。失败/捕获/old与new JSON/strict差异/永久结果见 [计费核对](results/golden-usage-reconciliation-root-v1/root-acceptance.json)。完整workspace需从头重跑，未借前20批执行。

Final fmt及CI同范围workspace/all-targets/all-features Clippy实际exit0，29项既有allowlist不增减，-D warnings保持；完整WS v4 source inventory与当前pins相同，守卫未触发，clean exit0且target不存在。证据compound/results/final-validation-root-v1/final-clippy-v4。自身产品CI及最后主线干净交付仍待；本地通过不替代CI。
