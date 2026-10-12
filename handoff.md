# Jarde 接续入口

持续目标 active，用户要求在当前聊天继续。固定 JADX **71 语法单元 / 612 测试文件**的账本，先完成大颗粒 MVP，再探索双方未覆盖的形态。组件测试或局部方法通过不能增加整体单元完成数。

## 当前主线交付

2026-10-13：产品 `7e93eb30647338dd8a6ddfc9994392f23457aa3e` 已提交推送 main；[自身 CI 38297425449](https://github.com/LordCasser/jarde/actions/runs/38297425449) 的 4 jobs / 52 steps 全部 completed/success。root 实际捕获后独立核两 seed 各 96 批 / 364 targets / 3525 passed / 0 failed / 98 ignored，逐 target、逐具名结果及 4854 个来源与本地一致。[CI 归档](openspec/changes/recover-ordered-field-concat-operands/results/own-product-ci-7e93-root-v1/README.md)保留实际捕获、v4 verifier、接受结果和两次私有 verifier 适配失败；生产代码未因 verifier 适配而修改。

循环 catch 父项 **8/8**，字段拼接子项 **6/6**。已推送检查点 `6bfc44833580b3068a0dab6144f59d5c5bb44691` 的[真实主线审计](openspec/changes/recover-ordered-field-concat-operands/results/clean-main-checkpoint-root-v1/README.md)已接受：only main / 14 aux detached、clean、main ancestor、无 target / 4854 来源恒同。父 3.3 / 子 2.3 按该证据关闭；关闭文档提交后 root 再对实际最终 HEAD 外置审计，结果路径 `/private/tmp/jarde-loop-field-clean-main-final-root-v1/execution.json`。检查点不证明未来 HEAD；接续时应核外置结果的 head 与对应 Git 提交。

- [局部门禁](openspec/changes/recover-ordered-field-concat-operands/results/reader-census-fix-and-final-local-root-v1/README.md)：focused-v12 八命令全部 0，705 passed / 0 failed / 4 ignored，4854 来源前后恒同，fmt / 当前 CI Clippy / strict / CLI build / finally exact clean 通过。receipt SHA `887bff1cb4999f13d03e9d3e6b411a032c46ce26151b0f566ea78448a7869401`。
- [完整工作区](openspec/changes/recover-ordered-field-concat-operands/results/full-workspace-accepted-root-v9/README.md)：195 命令全 0，两 seed 合计 7050 passed / 0 failed / 196 ignored，资源守卫、默认 stack、最终 clean / target absent 已独立接受。
- [最终整类矩阵](openspec/changes/recover-ordered-field-concat-operands/results/wrap-load-regression-fix-and-final-local-root-v1/README.md)：父项 40 命令 / 12 全类运行、字段 24 命令 / 4 Jarde + 4 JADX、TWR 23 命令及 C2W 27 命令 / 8 全类编译 / 16 次运行均同原。匿名类完整物理类组仍有 super 前 capture 赋值的编译差距，准确保留 compile-failed / no-runtime，不宣称匿名类通过。

冻结 CLI：`/private/tmp/jarde-ordered-field-final-check-root-v12/candidate-cli`，62277024 bytes / 0555，SHA `c8c18f26a256027262b514455c33f24d0ae1cbcec658e3d6ab76b2c3a9c2ccfd`。该旧产品交付时无活动工具链且target不存在。旧的Cargo、CI捕获与verifier session均terminal；当前活动工具链看下方联合候选状态。

## 正在推进的两个关联语法点

[recover-debug-reference-lifetimes](openspec/changes/recover-debug-reference-lifetimes/tasks.md) **5/6**，planning strict 通过。当前 debug/none 双 JDK 完整类基线 50 命令 / 100 raw、11 方法、原 17 场景已[归档](openspec/changes/recover-debug-reference-lifetimes/results/baseline-product-7e93-root-v1/README.md)。JADX raw alias 丢失导致编译失败；Jarde debug 有 slot5 Integer/String 类型冲突，debug/none 另有 handled() 拒绝。整体均 no-runtime。

v8 引用候选已应用，复用真实4763B完整fixture。unused身份用唯一已证明Store、下一decoded instruction与row.start严格相等、row包含身份范围的证据附名；保留旧alias/CFG/owner门。root修正真实alias-boundary夹具为`consume(value, value="new".toString())`，确保第二写入拥有exact String类型，旧Integer仍跨astore留在栈上；实际javac/javap及SSA拒绝测试均通过。新增夹具仅增加1 class/3 Code/0 handler/0 branch/0 jsr，reader实测178/178，清点1164/5160/501/3279/10。

当前两项联合完整类矩阵已实际通过并由root独立验收：[联合证据归档](openspec/changes/recover-debug-reference-lifetimes/results/joint-fullclass-candidate-root-v1/README.md)含390成员，root逐成员复开；debug/none各25命令、合计100raw、8条Jarde整类运行各17场景与同JDK原类逐字一致，11方法及完整source-map/物理owner保留。JADX两个profile在两个compiler模式中共8条编译失败，raw alias泛型写入错误仍是negative control，不作runtime通过。LVT和outertry两项目前各**5/6**，整CF11/CF18/71/612计数不变。

候选CLI：`/private/tmp/jarde-reference-prefix-combined-focused-run-root-v6/candidate-cli`，62562992 bytes / 0555，SHA `fe00fa7f0edc587a4e85bcc1136bd4397738c2a9df632432a2e79a8221c8bdd6`。v6命令全0，状态保留commands_passed_pending_independent_acceptance；v7库409/0、邻近integration71/0/1、structured8/0/1均过，整体仅Clippy int_plus_one失败，不能称pipeline通过。该断言已等价修正。

root新增真实negative fixture `tests/fixtures/p3-protected-prefix-loop-negatives/Negative.{java,class}`，实际JDK23 --release8 -g编译和javap都0；class1026B SHA `f15e25e3da8ab7a686ac057fa8b6542756c7a62f1a818572b9073316da91c489`。实际独立Code census增加1class/3Code/2handlers/5branch/0jsr，reader新断言1165/5163/503/3284/10尚待新reader测试。strict effect-suffix永久测试已实际通过：同次IR先核prefix0→loop7/16→next26<protected_end34，再核try_level完整回滚且预算不退款；没有把不验证candidate入口的direct-return诊断当永久成功测试。v8仅新增test临时引用E0716 failed，exact source已存`/private/tmp/jarde-reference-prefix-v8-exact-failure-source-root-v1`，finally exact clean0/target absent。root改显式Plan/Sites局部绑定，actual v9（session88674 terminal0）已独立验收，[最终局部门](openspec/changes/recover-debug-reference-lifetimes/results/final-focused-root-v9/README.md)为667/0/2，410Java lib/178reader/71邻近integration/8structured；fmt/当前CI Clippy/strict360/fresh CLI/finally exact clean全部0，4860来源恒同，13个本次具名门逐项核对。新CLI与上述v6字节完全相同；1.2/1.3可据此关闭。reader新count1165/5163/503/3284/10已由actual178项库测试验证。

fresh双seed工作区 root-v1 的 session**19763**已实际 terminal exit 1：第52条命令失败于旧测试 `same_name_lvt_sources_keep_their_existing_failure_verbatim`，它要求保留本次正要恢复的同名 LVT 类型错误。该run尚未进入第二seed，finally exact clean0/target absent，不得称workspace通过。全部receipt/raw和精确旧测试已[归档](openspec/changes/recover-debug-reference-lifetimes/results/same-name-fullclass-and-workspace-failure-root-v1/README.md)。root另实际接受双JDK共8个同名完整类的16条default/all CLI请求、2条未手修整类编译及8原始/8生成类运行，stdout逐字一致，3方法和物理source-map完整；原冻结baseline和其他负例不变，仅将旧失败要求改为精确类型拆分测试。[focused-v10](openspec/changes/recover-debug-reference-lifetimes/results/final-focused-root-v10/README.md) session**96405**已terminal0并独立接受：675/0/2、fmt/当前CI Clippy/strict360/fresh CLI/finally exact clean全0，4860来源恒同，14项具名门通过；新CLI仍为62562992B/0555/SHA fe00fa7f。全新双seed工作区为`/private/tmp/jarde-reference-prefix-full-workspace-run-root-v2`、session**61262**已terminal exit1，v4工具适配已审读；动态365targets/96batches每seed/4860sources是当前实际观察值。首seed第65条命令仅语料指纹失败：新增的Negative和ReferenceAliasBoundary共4个文件未登记；finally exact clean0/target absent，尚未进入第二seed。root只登记4项后，[fresh focused-v11](openspec/changes/recover-debug-reference-lifetimes/results/final-focused-root-v11/README.md) session36976已terminal0并独立接受680/0/3，14具名门/4860来源恒同，CLI字节仍同候选；全新双seed workspace-v3 的命令已完成：receipt `/private/tmp/jarde-reference-prefix-full-workspace-run-root-v3/execution.json`（SHA256 `f3ef8761467bca849881ddcea9a8369583689652c7341907c61fe3126239e3b8`）记录 7094 passed / 0 failed / 196 ignored；每 seed 3547/0/98，96 batches / 365 targets，4860 source pins，finally cargo clean。root 已独立接受并[归档完整工作区](openspec/changes/recover-debug-reference-lifetimes/results/full-workspace-accepted-root-v3/README.md)，390 raw、4860 来源恒同及 exact clean 均已核对；这不关闭 2.3。候选产品提交/push 与本次自身 CI 尚未发生，真实最终 HEAD clean-main 也仍开放。


[recover-protected-prefix-loop-sequences](openspec/changes/recover-protected-prefix-loop-sequences/tasks.md) **5/6**，planning strict 通过。root [同次 Walker trace](openspec/evidence/java-syntax-2026-10-13/outer-try-foreach-trace-root-v1/README.md)证明 try 只消费 prefix，返回 next=7；[同次子调用](openspec/evidence/java-syntax-2026-10-13/outer-try-foreach-child-probe-root-v1/README.md)实际在 header7 返回 ExceptionEdge(7,0)，未进入 loop body。来源 4854 全部按字节恢复，测试 1/0/0、finally clean / target absent。不能把预期的 Loop 当观察。

设计以一一绑定当前 Catches 的 private named row 事实贯通 header/body，限定 ordinary 单 range 的 prefix + 单 Loop；完整 Sequence 留在 Try.body，严格证明纯 range-end bridge 才成为兄弟 continuation。arm 继承，nested try/guard 清除或拒绝；保留 physical row、takeability、coverage、Stop/checkpoint 等旧门，不新增 pass/solver/global index。v6候选及v7编译API适配已应用。production path复用计费后的迭代owner集合，shape只浅判定；LoopTarget复制费用含exits；真实P5 Guard拒绝和晚期Stop全Walker回滚实际通过。Luna独立静态审查在`/private/tmp/jarde-protected-prefix-loop-review-luna-v3/report.md`，root已审读；该报告明确整体focused失败，不宣称runtime。普通重复owner不去重、Switch legacy path不放宽。接口事实问题已修正并实际通过2个integration测试；v9已接受新增反例/reader census/当前Clippy和fresh CLI，详见上段。

[独立边界完整类基线](openspec/changes/recover-protected-prefix-loop-sequences/results/boundary-baseline-product-7e93-root-v1/README.md)真实 24 命令 / 48 raw / 20 全类编译 / 4 原始运行：iterator、hasNext、next、output.add 四边界各 RuntimeException/Error 加 success/empty 共 10 场景；原四腿逐字同，16 份 JADX/Jarde 全类编译均失败、无 runtime。这是历史基线。当前candidate已由root独立接受32命令/64raw、4原始与8Jarde整类运行各10场景逐字相同，8JADX compile-failed/no-runtime；保留原handler/escape/消息/调用事件/effects，见上方联合归档。

## 架构债务与推进边界

[reference-definition-order-audit.md](openspec/evidence/java-syntax-2026-10-13/reference-definition-order-audit.md)记录 alias gate 的 definitions 顺序可能受 CFG/SSA 发布顺序影响；目前是静态路径推断，没有实际 IR 反例或错码证明。单独核验，不把排序修复混进当前 LVT 改动。匿名类 constructor 与 DF.loopFinally 的独立差距分别保留在各 change results 和 [completion-review.md](openspec/completion-review.md)。

前一项 Structured List MVP **9/9** 已交付；自身产品 ca45 与真实最终 HEAD 9dc64e4d 的干净主线审计均已接受，详见其 [verification-root.md](openspec/changes/recover-structured-list-generic-bodies/verification-root.md)。旧产品证据不替代新产品验收。CF11 / CF18 / 71 / 612 完成数保持账本原值。

## 工具、Git 与资源

root 独占共享 Git / Cargo / rustfmt / JDK / JADX / CLI；Luna 只读或私有准备，不改共享源码、不运行工具链、不创建分支/worktree。仅 main；14 个辅助 checkout 已 detached，须持续核 clean / main ancestor / 无 target。用户已授权提交推送和主线清理。

构建最低 **5 GiB** 可用空间、target 上限 **1 GiB**、**1 秒进程组守卫**、default Rust stack，finally exact cargo clean。不得并行 root 工具链或清活动 target。ScriptOS 已结束任务的缓存/重复临时产物累计释放约 2.35 GiB，独有验收材料保留；2026-10-13 最近复查可用约 22.6 GiB；其他任务并行使用磁盘，构建前应重新测量。

关闭文档实际 HEAD 845cfdc8d 的外置审计已通过。workspace-v3 已独立验收，下一步进行精确产品提交/push、本次自身 CI 与真实最终 HEAD clean-main；旧workspace-v1/v2已失败，保留其历史证据，不得拼接历史失败、删方法、修补生成 Java 或降低断言。之后的独立MVP方向见[本地JADX测试与实现分析](openspec/evidence/java-syntax-2026-10-13/try-list-signature-next-mvp-analysis-root-v1/README.md)：复用普通Try/foreach证书和现有List用途census，统一覆盖现有精确元素类型，不引入通用solver；该记录是静态分析，尚未实施或运行验收。

下一MVP已保存[未编译的完整候选源码与验收方案](openspec/evidence/java-syntax-2026-10-13/try-list-signature-next-mvp-analysis-root-v1/test-plan-source-prep-root-v1/README.md)，含root静态审查及原17oracle保留核对；不可当作已执行基线或语法完成。
