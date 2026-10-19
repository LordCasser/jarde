# Jarde 接续入口

更新：2026-10-19。先拉取GitHub并核对工作区；当前事实以本页、Git和对应验收收据为准。[历史快照](docs/history/handoff-through-2026-10-19.md)仅描述过去时点。

## 当前推进：剩余四固定原件

完整基线与真实IR已经root/独立接受，见[四原件完整三方](openspec/evidence/java-syntax-2026-10-19/cf09-last-four-original-root-v1/README.md)和[同轮真实IR](openspec/evidence/java-syntax-2026-10-19/cf09-last-four-ir-root-v1/README.md)。DoWhileBreak3现有机制通过：8API、16 Jarde整类编译、32 JardeVM加4 JADX VM/360CASE同原，test映射10/10；无check，外层JUnit文本断言未执行。原始do-while与JADX while&&拼写不同不算缺口。

DoWhileBreak2真实gap：8API回退、16Jarde整类编译失败，提前return17被goto-prefix检查早拒绝；应提前复用已有terminal_returns证书。BreakInLoop3虽有NYI/@Test注释，但实际JADX整编/双runtime/check成功；Jarde8API因owner11重叠回退，catch回跳递归重开Try，不能把它当普通自然loop。canonical block11有未保护lead11..17，row17..28/handler31/type32；拟复用精确named retry证书与Try lead，保留正常catch diamond到join82。

两项OpenSpec [return leaves](openspec/changes/recover-proved-post-test-loop-return-leaves/tasks.md)与[prefixed retry](openspec/changes/recover-proved-prefixed-catch-retry-loops/tasks.md)的实现已由Luna完成并经root集成审查。Do2复用既有terminal_returns证明，保留StatementFree已有表头路径优先；Loop3复用named retry/closed DAG，精确拼接Straight+当前LoopBreak，并仅对Sites/SSA已证明构造表达式去除protected-arm位置限制，初始化效果不移动。两项负例和完整ownership回滚通过。

[定向实现及失败历史](openspec/changes/recover-proved-prefixed-catch-retry-loops/results/focused-implementation-root-v1/README.md)：Java核心552、CF12 9、Do2 public1、Loop3 public2及旧Loop4/5 public4通过；旧预算门365@29/130@1未改。最终Clippy/fmt/build通过，target已清理。fixture普查1367/5991/650/4360/16及指纹通过。最终同一CLI的Do2十一例、Loop3原件和Do3十例对照已完成，完整原始流正在归档；Loop3 writeMore来源48/49缺88，独立控制run40/41缺71，保持真实分母，单列映射债务。独立重试控制Jarde行为已通过，JADX默认包重命名导致原驱动未加载，正补齐关闭rename的完整三方；不能把32个Jarde VM算作三方完整。完整workspace和本产品自身CI尚待运行，产品与clean/远端结论等待最终收据。

Loop6原Smali/Dex已实际组装、JADX两输入均保留1break；Jarde直接输入unknown_artifact_kind，Dex-only ZIP为0class候选。此为架构外输入边界，不以等价Java补齐，不新增Dex前端，不冒称原件动态覆盖。固定13原件暂10接受、2JVM真实gap、1输入边界；CF09/71/612仍partial。

## 上一完整产品：Loop4/5 完整验收通过

主线`main`。当前产品`412245d514896b1d0eb9a8fe24388f6a70668637`，由首版恢复a4ccac6b4和最小预算修正412245d51组成。Luna私有实现，root审查、集成及实际对照；无新分支/worktree。

Loop4给自然循环外的闭合break分支DAG建立精确owner证明，复用既有sequence/If/LoopBreak，保留ForHeader。Loop5在精确普通named catch范围内组合prefix+完整While+return-tail；保留effectful latch的append/update。Builder复用已有catch身份，区分handler入口异常绑定与同物理槽普通double，不引入通用槽拆分。外来入口/出口、Call/Return跨owner、异常行漂移、重复/遗失owner及预算/取消/晚期Stop均有拒绝和回滚门。

- [完整三方与定向验收](openspec/changes/recover-named-catch-loop-return-tails/results/focused-and-full-comparison-root-v1/README.md)：两原件各70命令/140raw、8API/24方法structured；各Jarde8及JADX2完整JDK23编译成功、20生成VM同原，10次JDK8框架55/52编译边界保留。所有case及实际check按同profile/runtime对照，default/all、identity和映射稳定。Loop5扩展12例的20VM/240case亦同原件，含实际null/M/g/重复点/零/超大数。root与独立raw审计已接受。
- 最终定向CF12 9、Java核心540、公开API4、Clippy/fmt均通过。首轮native和首版CI39248350582在旧CF12精确预算门失败，因非候选单出口循环被多付完整scan费用；仅前置既有两出口必要条件，保留真实候选收费和取消轮询。旧365@29/130@1测试完全未改。
- 最终fresh workspace：394 targets、103批、3841通过/0失败/98既有忽略，96精确门通过；5299 source pins前后相等，默认栈及5GiB/1GiB守卫未触发，finally target absent。构建3346/native5299来源并集5306均绑定产品Git blob和实际字节/模式。独立raw审计已接受。[本产品自身CI39251504476](https://github.com/LordCasser/jarde/actions/runs/39251504476)四jobs全部成功，52个官方步骤逐一绑定原始ZIP；双seed各394目标、3841通过/0失败/98既有忽略，全部3939具名结果逐项相等，141精确门各一次通过。root和独立审计均接受；[native/自身CI归档](openspec/changes/recover-named-catch-loop-return-tails/results/native-and-own-ci-root-v1/README.md)保留完整原流、工具与来源绑定。
- 既有工具债务：runner的`observed_counts`把编译产物顺序与执行摘要错配，影响316个target项/95批。已有verifier及独立审计都按实际Running头重建394目标和3939具名结果，二者逐项相等；原收据不改，不能直接消费其错误计数表。本轮不改语法范围外的runner。
- 来源覆盖：Loop4 test23/23；Loop5 test59/60，缺BCI103（物理goto14），[独立映射债务](openspec/evidence/java-syntax-2026-10-19/cf09-implicit-loop-transfer-mapping-debt-root-v1/README.md)已记录，不宣称完整来源覆盖，也未推断与旧69/78同根因。

最终CLI：`/private/tmp/jarde-loop45-final-focused-root-v4-20261019/candidate-cli`，64,631,664 bytes、0555、SHA-256 `d29b6fee723d8f59fbecd9fd8be9f15a8ec73de3a90f5a2a29b24ff91e4af607`。临时路径不保证跨机器存在；证据及来源pin进入持久归档，二进制/target不入库。主规格已同步2要求/8场景；两项OpenSpec均8/8完成。严格校验与最终交付检查见验收入口。最终文档提交使用[skip ci]，不冒称另跑产品CI。

## 下一步与独立范围

Loop4/5旧轮交付时为9/13；当前四原件状态以上方新检查点为准。CF09、71单元及612文件保持partial，见[JADX账本](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md)。

四原件静态输入边界历史见[静态输入边界](openspec/evidence/java-syntax-2026-10-19/cf09-breakinloop3-to6-static-scope-root-v1/README.md)；它们的完整基线现已完成，后续按两项真实gap实施，6继续单列输入边界。finThrow5/5、nestedFin16/16、Loop2与ComplexIf已验收旧片不重做。

## 工作与资源边界

root独占共享写、Git/网络及Cargo/JDK/JADX/Jarde CLI；Luna只读共享文件，在新私有tmp用Python`-B`准备实现和审计。按OpenSpec派发，root负责架构、失败路径及完整对照验收。

Cargo串行：余量≥5GiB、target≤1GiB、默认Rust stack、每秒进程组守卫、finally精确清理。Cargo期间不共享写、不Git mutation、不网络或手动并行JDK/CLI。shell使用`login:false`。所有完成/clean/远端/worktree结论须由最终实时审计核对，不能沿用历史快照。

交付审计：主线产品与origin/main的同步、工作区清洁及target absent由最终推送后实时复核；此前14个辅助worktree均detached、干净且HEAD为main祖先，未新增分支/worktree，只有main占用分支。历史干净辅助checkout不作为待完成实现。
