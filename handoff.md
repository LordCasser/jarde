# Jarde 接续入口

更新：2026-10-20。先拉取GitHub并核对工作区；当前事实以本页、Git和对应验收收据为准。[历史快照](docs/history/handoff-through-2026-10-19.md)仅描述过去时点。

## 停止交接：四原件验收完成，两项OpenSpec各7/8，尚未归档

用户要求把未完成工作写入handoff并停止，本会话不再推进。代码产品 `8ea98e2e11d64f02092ac200d2b596511dcf5e40` 已通过本机完整验收及[自身CI39278336384](https://github.com/LordCasser/jarde/actions/runs/39278336384)。两项OpenSpec [return leaves](openspec/changes/recover-proved-post-test-loop-return-leaves/tasks.md)与[prefixed retry](openspec/changes/recover-proved-prefixed-catch-retry-loops/tasks.md)各7/8：3.3已接受，3.4未完成；两个目录仍在active changes，尚未移动到archive。主规格已同步2要求/9场景。本次仅提交交接文档和CI证据，不冒称文档后继提交另跑产品CI。

### 接续者先做这些

1. 先拉取GitHub并核对当前main。无需重做两项实现、三方矩阵、native或产品CI；当前剩余是文档、归档及最终交付检查。
2. [自身CI证据包](openspec/changes/recover-proved-prefixed-catch-retry-loops/results/own-ci-root-v1/README.md)已入本次交接：874 payload，tar SHA256 `cd6190ccd2d8d4db922c7b30054ab0f2ec2b6b2fc3c448986039827857c218c1`。root已逐成员复开；额外Luna独立tar复核在用户停止时中断，尚未验收。包内root合同决定、实际复跑报告、原始失败与源码绑定均可查。
3. 复核两个delta仍逐字同步主spec，按OpenSpec归档流程关闭3.4；归档日期使用接续时的实际日期。归档后修正Markdown链接，保留JSON/raw/tar中的历史路径。最后一次OpenSpec strict为归档前390 passed/0 failed；归档后尚未运行，结果数量应实际派生，不能照抄390。
4. 完成文档最终状态、归档后strict与链接检查，再勾选3.4、提交推送，并实时检查HEAD/origin/GitHub main相等、工作区干净、全部15 checkout无target、14辅助worktree detached/clean/为main祖先，以及5333来源仍绑定产品。不要把本次handoff提交或旧产品检查点当成上述最终关闭验收。

本机接续工具在 `/private/tmp/jarde-last-four-closeout-tools-root-v1-20261019/`，包括`audit_delivery.py`与`run_logged.py`；5333来源绑定原件是 `/private/tmp/jarde-last-four-product-binding-root-v1-20261019.json`。两个合同/独立审计的root实际复跑分别在 `/private/tmp/jarde-last-four-ci-contract-review-root-rerun-v3-20261020/` 和 `/private/tmp/jarde-last-four-ci-independent-rerun-root-v1-20261020/`，也已封存在CI包中。私有临时路径不保证跨机器存在，按清单迁移工具副本，不改封存原件。

曾为准备归档临时将3.4勾成完成，并取得8/8的apply/status私有回执；用户要求停止后已恢复3.4未完成，当前实际7/8。`/private/tmp/jarde-last-four-{apply,status}-recover-proved-*-root-v1-20261020/` 的8/8回执仅是中间状态，不代表归档或最终交付完成。未创建`results/final-delivery-root-v1`，也未移动任何change目录。

### 已完成事实与边界

基线和真实IR见[四原件基线](openspec/evidence/java-syntax-2026-10-19/cf09-last-four-original-root-v1/README.md)、[同轮IR](openspec/evidence/java-syntax-2026-10-19/cf09-last-four-ir-root-v1/README.md)。DoWhileBreak2复用终端return叶证书，保留StatementFree既有优先级；BreakInLoop3精确持有未保护lead/单named row/retry/normal DAG，组合现有Straight+LoopBreak，并复用Sites/SSA证明允许循环前构造保存跨try使用，初始化效果不移动。无新CFG/AST或Dex前端。

[最终完整对照](openspec/changes/recover-proved-prefixed-catch-retry-loops/results/final-comparison-root-v1/README.md)：Do2十一例70命令/20VM220CASE；Loop3原件94命令/36VM含实际check；Do3十例94命令/36VM360CASE；独立重试控制105命令/40VM400CASE。8API/default-all稳定，完整生成源码原样重编及成功产物双runtime隔离执行；框架55/52失败单列。Loop3 NYI/@Test注释与当前实际JADX成功分开，Do3无check/外层JUnit断言未运行。

[native完整验收](openspec/changes/recover-proved-prefixed-catch-retry-loops/results/native-accepted-root-v1/README.md)：一次fresh运行396 targets/104批、3856通过/0失败/98既有忽略，111精确门；3954具名结果由实际Running重建。5326 native/3373 build并集5333来源逐Git blob/bytes/mode绑定，默认栈/5GiB/1GiB/每秒守卫与finally clean均接受。runner旧observed_counts错配93批314目标项，不消费原错误归属。

[本产品自身CI与合同审计](openspec/changes/recover-proved-prefixed-catch-retry-loops/results/own-ci-root-v1/README.md)：4 jobs/52 API步骤全部成功；双seed各396 targets、3856通过/0失败/98 ignored、3954具名结果与156精确门，两轮完整Counter相等。跨native比较时统一should_panic展示后缀，并唯一源码绑定一个被GitHub遮蔽的测试名；不声称原始Counter字面相等。官方ZIP有38个独立step成员、缺14个，fuzz CLI原流不完整；原始aggregate与API补充证据均保留，未拼造缺失正文。私有ZIP完整成员门仍失败；root已另按tasks 3.3合同实际重跑接受，不与OpenSpec strict混淆。

13个固定原件中12项已接受，Loop6是Smali/Dex输入边界：JADX实际解码原Smali和生成Dex，Jarde原始输入拒绝、Dex-only ZIP无class候选；无JVM oracle，不以Java控制替代。来源债务：Loop3 writeMore 48/49缺BCI 88，独立控制run 40/41缺BCI 71；两者各自为goto映射缺口，不与旧Loop5 BCI 103推断同根因。Iterator泛型字段呈现和runner observed_counts错配仍独立记录。[成本模型债务](openspec/evidence/java-syntax-2026-10-19/cf09-prefixed-retry-cost-model-debt-root-v1/README.md)单列重复扫描、边集物化轮询及额外visited复制。CF09/71/612保持partial。

## 上一完整产品：Loop4/5 完整验收通过

主线`main`。当前产品`412245d514896b1d0eb9a8fe24388f6a70668637`，由首版恢复a4ccac6b4和最小预算修正412245d51组成。Luna私有实现，root审查、集成及实际对照；无新分支/worktree。

Loop4给自然循环外的闭合break分支DAG建立精确owner证明，复用既有sequence/If/LoopBreak，保留ForHeader。Loop5在精确普通named catch范围内组合prefix+完整While+return-tail；保留effectful latch的append/update。Builder复用已有catch身份，区分handler入口异常绑定与同物理槽普通double，不引入通用槽拆分。外来入口/出口、Call/Return跨owner、异常行漂移、重复/遗失owner及预算/取消/晚期Stop均有拒绝和回滚门。

- [完整三方与定向验收](openspec/changes/recover-named-catch-loop-return-tails/results/focused-and-full-comparison-root-v1/README.md)：两原件各70命令/140raw、8API/24方法structured；各Jarde8及JADX2完整JDK23编译成功、20生成VM同原，10次JDK8框架55/52编译边界保留。所有case及实际check按同profile/runtime对照，default/all、identity和映射稳定。Loop5扩展12例的20VM/240case亦同原件，含实际null/M/g/重复点/零/超大数。root与独立raw审计已接受。
- 最终定向CF12 9、Java核心540、公开API4、Clippy/fmt均通过。首轮native和首版CI39248350582在旧CF12精确预算门失败，因非候选单出口循环被多付完整scan费用；仅前置既有两出口必要条件，保留真实候选收费和取消轮询。旧365@29/130@1测试完全未改。
- 最终fresh workspace：394 targets、103批、3841通过/0失败/98既有忽略，96精确门通过；5299 source pins前后相等，默认栈及5GiB/1GiB守卫未触发，finally target absent。构建3346/native5299来源并集5306均绑定产品Git blob和实际字节/模式。独立raw审计已接受。[本产品自身CI39251504476](https://github.com/LordCasser/jarde/actions/runs/39251504476)四jobs全部成功，52个官方步骤逐一绑定原始ZIP；双seed各394目标、3841通过/0失败/98既有忽略，全部3939具名结果逐项相等，141精确门各一次通过。root和独立审计均接受；[native/自身CI归档](openspec/changes/recover-named-catch-loop-return-tails/results/native-and-own-ci-root-v1/README.md)保留完整原流、工具与来源绑定。
- 既有工具债务：runner的`observed_counts`把编译产物顺序与执行摘要错配，影响316个target项/95批。已有verifier及独立审计都按实际Running头重建394目标和3939具名结果，二者逐项相等；原收据不改，不能直接消费其错误计数表。本轮不改语法范围外的runner。
- 来源覆盖：Loop4 test23/23；Loop5 test59/60，缺BCI103（物理goto14），[独立映射债务](openspec/evidence/java-syntax-2026-10-19/cf09-implicit-loop-transfer-mapping-debt-root-v1/README.md)已记录，不宣称完整来源覆盖，也未推断与旧69/78同根因。

上轮最终CLI：`/private/tmp/jarde-loop45-final-focused-root-v4-20261019/candidate-cli`，64,631,664 bytes、0555、SHA-256 `d29b6fee723d8f59fbecd9fd8be9f15a8ec73de3a90f5a2a29b24ff91e4af607`。临时路径不保证跨机器存在；证据及来源pin进入持久归档，二进制/target不入库。上轮主规格同步2要求/8场景，两项OpenSpec均8/8。此段仅为Loop4/5历史产品检查点。

## 下一步与独立范围

Loop4/5旧轮交付时为9/13；当前四原件状态以上方新检查点为准。CF09、71单元及612文件保持partial，见[JADX账本](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md)。

四原件静态输入边界历史见[静态输入边界](openspec/evidence/java-syntax-2026-10-19/cf09-breakinloop3-to6-static-scope-root-v1/README.md)；Do2与Loop3两项真实gap已实现并完成验收，Loop6继续单列输入边界。finThrow5/5、nestedFin16/16、Loop2与ComplexIf已验收旧片不重做。

## 工作与资源边界

root独占共享写、Git/网络及Cargo/JDK/JADX/Jarde CLI；Luna只读共享文件，在新私有tmp用Python`-B`准备实现和审计。按OpenSpec派发，root负责架构、失败路径及完整对照验收。

Cargo串行：余量≥5GiB、target≤1GiB、默认Rust stack、每秒进程组守卫、finally精确清理。Cargo期间不共享写、不Git mutation、不网络或手动并行JDK/CLI。shell使用`login:false`。所有完成/clean/远端/worktree结论须由最终实时审计核对，不能沿用历史快照。

交付审计：主线产品与origin/main的同步、工作区清洁及target absent由最终推送后实时复核；此前14个辅助worktree均detached、干净且HEAD为main祖先，未新增分支/worktree，只有main占用分支。历史干净辅助checkout不作为待完成实现。
