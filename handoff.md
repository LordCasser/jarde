# Jarde 接续入口

更新：2026-10-19。先拉取GitHub并核对工作区；当前事实以本页、Git和对应验收收据为准。[历史快照](docs/history/handoff-through-2026-10-19.md)仅描述过去时点。

## 当前产品：Loop4/5 完整验收通过

主线`main`。当前产品`412245d514896b1d0eb9a8fe24388f6a70668637`，由首版恢复a4ccac6b4和最小预算修正412245d51组成。Luna私有实现，root审查、集成及实际对照；无新分支/worktree。

Loop4给自然循环外的闭合break分支DAG建立精确owner证明，复用既有sequence/If/LoopBreak，保留ForHeader。Loop5在精确普通named catch范围内组合prefix+完整While+return-tail；保留effectful latch的append/update。Builder复用已有catch身份，区分handler入口异常绑定与同物理槽普通double，不引入通用槽拆分。外来入口/出口、Call/Return跨owner、异常行漂移、重复/遗失owner及预算/取消/晚期Stop均有拒绝和回滚门。

- [完整三方与定向验收](openspec/changes/recover-named-catch-loop-return-tails/results/focused-and-full-comparison-root-v1/README.md)：两原件各70命令/140raw、8API/24方法structured；各Jarde8及JADX2完整JDK23编译成功、20生成VM同原，10次JDK8框架55/52编译边界保留。所有case及实际check按同profile/runtime对照，default/all、identity和映射稳定。Loop5扩展12例的20VM/240case亦同原件，含实际null/M/g/重复点/零/超大数。root与独立raw审计已接受。
- 最终定向CF12 9、Java核心540、公开API4、Clippy/fmt均通过。首轮native和首版CI39248350582在旧CF12精确预算门失败，因非候选单出口循环被多付完整scan费用；仅前置既有两出口必要条件，保留真实候选收费和取消轮询。旧365@29/130@1测试完全未改。
- 最终fresh workspace：394 targets、103批、3841通过/0失败/98既有忽略，96精确门通过；5299 source pins前后相等，默认栈及5GiB/1GiB守卫未触发，finally target absent。构建3346/native5299来源并集5306均绑定产品Git blob和实际字节/模式。独立raw审计已接受。[本产品自身CI39251504476](https://github.com/LordCasser/jarde/actions/runs/39251504476)四jobs全部成功，52个官方步骤逐一绑定原始ZIP；双seed各394目标、3841通过/0失败/98既有忽略，全部3939具名结果逐项相等，141精确门各一次通过。root和独立审计均接受；[native/自身CI归档](openspec/changes/recover-named-catch-loop-return-tails/results/native-and-own-ci-root-v1/README.md)保留完整原流、工具与来源绑定。
- 既有工具债务：runner的`observed_counts`把编译产物顺序与执行摘要错配，影响316个target项/95批。已有verifier及独立审计都按实际Running头重建394目标和3939具名结果，二者逐项相等；原收据不改，不能直接消费其错误计数表。本轮不改语法范围外的runner。
- 来源覆盖：Loop4 test23/23；Loop5 test59/60，缺BCI103（物理goto14），[独立映射债务](openspec/evidence/java-syntax-2026-10-19/cf09-implicit-loop-transfer-mapping-debt-root-v1/README.md)已记录，不宣称完整来源覆盖，也未推断与旧69/78同根因。

最终CLI：`/private/tmp/jarde-loop45-final-focused-root-v4-20261019/candidate-cli`，64,631,664 bytes、0555、SHA-256 `d29b6fee723d8f59fbecd9fd8be9f15a8ec73de3a90f5a2a29b24ff91e4af607`。临时路径不保证跨机器存在；证据及来源pin进入持久归档，二进制/target不入库。主规格已同步2要求/8场景；两项OpenSpec均8/8完成。严格校验与最终交付检查见验收入口。最终文档提交使用[skip ci]，不冒称另跑产品CI。

## 下一步与独立范围

本轮Loop4/5已完成并归档，固定13原件现为9已接受、4待完整基线。CF09、71单元及612文件保持partial，见[JADX账本](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md)。

剩余固定原件为BreakInLoop3、BreakInLoop6、DoWhileBreak2/3。按[静态输入边界](openspec/evidence/java-syntax-2026-10-19/cf09-breakinloop3-to6-static-scope-root-v1/README.md)先做完整三方基线：3的NYI先核实际JADX，6的Smali/Dex与class/JAR必须分开，不能用JVM构造例代替原输入。finThrow5/5、nestedFin16/16、Loop2与ComplexIf已验收旧片不重做。

## 工作与资源边界

root独占共享写、Git/网络及Cargo/JDK/JADX/Jarde CLI；Luna只读共享文件，在新私有tmp用Python`-B`准备实现和审计。按OpenSpec派发，root负责架构、失败路径及完整对照验收。

Cargo串行：余量≥5GiB、target≤1GiB、默认Rust stack、每秒进程组守卫、finally精确清理。Cargo期间不共享写、不Git mutation、不网络或手动并行JDK/CLI。shell使用`login:false`。所有完成/clean/远端/worktree结论须由最终实时审计核对，不能沿用历史快照。

交付审计：主线产品与origin/main的同步、工作区清洁及target absent由最终推送后实时复核；此前14个辅助worktree均detached、干净且HEAD为main祖先，未新增分支/worktree，只有main占用分支。历史干净辅助checkout不作为待完成实现。
