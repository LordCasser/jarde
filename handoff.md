# Jarde 接续入口

更新：2026-10-19。当前状态以本页和 Git 为准；旧检查点已移入 [历史快照](docs/history/handoff-through-2026-10-19.md)。其他机器接续先拉取 GitHub 并核对工作区。

## 当前交付

主线为 `main`。最近完整验收的产品为 `618b5e20f27f94ba28ea2f9e4310648718475f82`；后继为规格、证据和交接文档提交，不代表重新运行产品 CI。finThrow（5/5）、nestedFin（16/16）及旧循环片已完成，不重做。

[条件重抛与异常重试](openspec/changes/recover-branching-named-catch-retry-loops/results/README.md) 已完成（6/6）：扩展既有 RetryLoopFacts/Frame 的三块 handler 证明，复用 If/Throw/LoopContinue；精确保护区、边闭合、owner、预算及取消门保留，无新 CFG/Region/pass/Builder 机制。

- 完整 `TestBreakInLoop2`：8 API，16 次 Jarde 完整类重编，32 个隔离 VM、64 次观察均同原件；四个方法结构化、目标来源17/17、default/all恒同。原件 catch 动态不可达，由独立可达控制例的32 VM/160观察验证重试、同一异常重抛、条件反转及 retry 效果自身异常。JADX 原 JDK8 framework 55/52 编译边界保留。
- [最终完整验收](openspec/changes/recover-branching-named-catch-retry-loops/results/native-and-own-ci-root-v1/README.md)：一次 fresh workspace 为393 targets、103批、3823通过/0失败/98既有忽略，78精确门；[产品自身CI](https://github.com/LordCasser/jarde/actions/runs/39227711617) 四 job 成功，双 seed 全部具名结果一致。5280构建/native来源逐Git blob与实际字节/模式绑定。主规格已同步；严格校验以该验收入口为准。
- [前一完整交付](openspec/changes/bound-loop-header-chains-at-body-effects/results/README.md)：ComplexIf/ComplexIf2已对照接受，来源32/33与33/34的缺口单列。

最终 CLI：`/private/tmp/jarde-loop2-retry-final-focused-root-v2-20261019/candidate-cli`，64,347,312 bytes，模式0555，SHA-256 `7e321a7f4fd6a029540ab169f79572c6ad83e74119900ac6d695290c9c055318`；与完整对照 CLI 同字节。临时目录不保证跨机器存在，持久证据从上面的验收入口读取。

## 当前处理：Loop4/5

[完整三方基线](openspec/evidence/java-syntax-2026-10-19/cf09-loop4-loop5-original-root-v1/README.md)已并行采集并root/独立复核接受：每目标46命令/8API/20完整编译尝试。两个test均fallback `jre_region_uncovered_blocks`，Loop4遗漏3块、Loop5遗漏11块；ctor/check均structured。每目标Jarde16整类编译失败、无VM；JADX2次JDK23整树编译成功/4VM全部case+check同原件，2次JDK8编译框架55/52边界保留。原件check在JDK23通过、JDK8依赖异常独立记录。

[四原件真实IR](openspec/evidence/java-syntax-2026-10-19/cf09-loop4-loop5-ir-root-v1/README.md)已执行、独立复核并归档：1 test/4 profile，默认栈，helper精确恢复、target清理。Loop4已证明ForHeader却漏外侧条件break体；Loop5普通try的prefix+loop候选只接受末尾goto，原件却为范围右边界return表达式。probe仅test且Sites为空，与公开路径输入边界已单列。

两项OpenSpec已由Luna提交私有补丁、root审查并集成：Loop4仅在闭合、无外部入口/额外出口的break分支DAG内扩展owner并复用既有sequence；Loop5精确绑定普通named catch的物理范围，将prefix+完整While+return-tail组合，保留append/update；Builder复用catch身份，区分handler入口异常绑定与同槽普通double。Call/Return跨owner边、异常行漂移、owner重复/遗漏及晚期Stop均保留拒绝/回滚门，无全局scope或ForHeader放宽。

首版产品a4ccac6b4的完整三方矩阵与Loop5扩展12例已由root/独立接受，但fresh workspace在旧CF12预算精确门失败。已确认单出口循环在必然拒绝前被多付候选扫描费用；Luna最小补丁仅前置原有两出口必要条件，真实候选收费/取消不变，未修改旧测试预算。修正后的CF12全部9项（仍365@29、130@1）、Java核心540、公开API4、Clippy/fmt均通过，构建前后源码一致且target已清理。

新CLI64,631,664 bytes、0555、SHA `d29b6fee723d8f59fbecd9fd8be9f15a8ec73de3a90f5a2a29b24ff91e4af607`。此新产物的完整三方/扩展对照、fresh workspace和自身CI/来源绑定待重新关闭；首版运行结果不能冒充新版本验收。首版实际来源覆盖为Loop4 23/23、Loop5 59/60，缺BCI103（goto14）单列映射债务。主规格2要求/8场景已同步，strict388通过；固定原件状态仍保留待最终验收。

## 下一步与独立债务

固定13原件中7项已完整接受、2项实际gap、4项待完整基线；CF09、71单元、612文件整体仍为 partial。覆盖范围以 [JADX账本](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md) 为准，不把原件数当作特性数。

- 按 [BreakInLoop3–6静态计划](openspec/evidence/java-syntax-2026-10-19/cf09-breakinloop3-to6-static-scope-root-v1/README.md) 4/5已完成完整基线且确认gap，按上述当前处理继续定位。3含NYI，先查实际JADX；6单列Smali/Dex与class/JAR输入边界，均未算完成。
- [隐式循环转移映射债务](openspec/evidence/java-syntax-2026-10-19/cf09-implicit-loop-transfer-mapping-debt-root-v1/README.md)：BCI69/78缺失仍待实际证明，独立处理。
- collector 参数、物理库存与语义投影统一属于工具债务，不混入语法修复。

## 工作与资源边界

root 独占共享写入、Git/网络及Cargo/JDK/JADX/Jarde CLI；Luna只读共享文件，在新私有tmp用Python `-B`准备实现和审计，不建分支或worktree。实现按OpenSpec派发，root做架构、失败路径及完整对照验收。

Cargo串行：可用空间至少5GiB、target上限1GiB、默认Rust stack、每秒进程组守卫、finally精确清理。Cargo期间不得共享写入、Git mutation、网络或手动并行JDK/CLI。shell使用 `login:false`；单独检查退出码。

最终主线/远端/worktree与target状态由交付后的实际审计核对；接续时重新检查实时状态，不将历史clean快照当作当前结论。
