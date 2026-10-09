# HANDOFF — jarde 主线与下一语法片

当前聊天按用户指示继续推进：root负责架构、OpenSpec、对抗验收；确定性实现交给Luna。先读取本文件和71单元账本，再核对实际主线，不能把历史通过日志当作当前HEAD的结果。

```sh
git status --short
git log -1 --oneline
git rev-parse HEAD origin/main
git branch -av
git worktree list
gh run list --workflow CI --commit "$(git rev-parse HEAD)" --limit 3
```

## 当前语法片

[split-proved-reference-slot-lifetimes](openspec/changes/split-proved-reference-slot-lifetimes/verification-root.md)补齐无LVT普通引用槽的独立生命周期，复用既有SSA owner、变量分段、名称和声明。八类32完整输入从8/32增至24/32：16 no-debug新增完整重编/隔离行为成功；16 debug保持原结果，其中8同名LVT仍失败。原/JADX32腿成功对照已冻结。完整LG/finalize与整个EM-20不宣称完成。

root发现并修正了“仅查store直接uses会漏掉aload后的旧栈值”问题；HeldUse以及两javac的非零栈checkcast控制都直接验证命名元数据未发布分段。初候选的失败、最终完整32腿、10个负控制及逐字段source-map审查均永久保留。[任务](openspec/changes/split-proved-reference-slot-lifetimes/tasks.md)、[实现与真实SSA](openspec/changes/split-proved-reference-slot-lifetimes/results/implementation-notes.md)、[最终CLI2元数据](openspec/changes/split-proved-reference-slot-lifetimes/results/candidate-cli-v2.json)、[门禁实际index](openspec/changes/split-proved-reference-slot-lifetimes/results/gates-v3/index.json)是入口。代码4fba93438的[CI37915062977](https://github.com/LordCasser/jarde/actions/runs/37915062977)已四job全部success，包含JDK25oracle；本片7/7任务完成。原JSON和清理记录见本片root验收，HEAD状态仍以顶部命令为准。

此前同类泛型调用已合入主线，9/9任务完成，历史root验收在 [recover-same-class-generic-call-consumers](openspec/changes/recover-same-class-generic-call-consumers/verification-root.md)。本片开始前主线82090226的CI37892494665四job成功。旧handoff完整保存在新片results/handoff-before-reference-slot.md。

## 当前推进：EM-18异构数组初始化

`recover-heterogeneous-array-init` 已完成架构修订、产品实现、本地root与确切SHA CI验收（9/9任务），已提交推送main代码29dcd5e892696e9f6b5adbb657ffa0a7db576c27。实际CI37926854875四job、48steps全部success，含双seed和JDK25 oracle；原JSON及hash见本片root验收，当前下一片工作区修改不因此获得验收。现有平台/数组事实加本次Runtime选中的有界snapshot header关系授权具体aastore；匹配store BCI与完整source/component，保留元素表达式，支持等秩引用数组，无新层级服务、LUB或element cast。源码、快照事实与真实SSA需以当前文件核对，Atlas行号缓存过时。

永久基线、架构和失败审查入口：[results/README-baseline.md](openspec/changes/recover-heterogeneous-array-init/results/README-baseline.md)。原18有效腿的原程序、源码参考版JADX均完整重编/验证运行双流匹配18/18，Jarde旧行为0/18；exit0但stdout为空的4腿不算恢复。CLI1 `0ebf4e6189c02d1d84072c9aeacd380303e71e6885f95408e054e8f843154f79` 当前同输入8/18完全通过（fresh/frozen CT、boxed、nested各两腿），其余inline-new仍失败，不关闭EM18整单元。原两次runner的命名/观察器失败与修正均保留；以root verification而非早期汇总为准。

完整factory-family验证平台接口/集合/异常及自有类两跳/interface/引用数组；direct-new是独立完整未覆盖控制，不能伪称非法赋值方向。新手写observer的loop/ternary共享arrayread曾拒绝，v2完整保留；v3只简化observer、不删目标成员；root完整六类两compiler腿独立重放，factory 2/2隔离重编/双流匹配，共50个store来源/无element cast核验通过，direct-new 0/2明确未覆盖。永久入口为本片verification-root.md与results/candidate-v1-fixture-v3-root-verification.json。所有源码完整生成，runtime CP只能用其新编译目录。

已写好下一项 [compose-constructed-reference-array-elements](openspec/changes/compose-constructed-reference-array-elements/)（规划4/4、取证任务2/8已验收，产品与集成测试实施中）。构造器验证不承认aastore reader，ArrayInitializers又不接受verified constructor atom，需在现有两个证明内准确、原子组合；单开放白名单不安全。保留原arrays→sites编排对constructor内varargs/char[]的支持。周边concat/arraylength/field与primitive constructor-argument conversion缺口独立记录，不混入当前类型片。

构造组合片已冻结前片CLI与完整direct-v3输入，原始source与Jarde生成source的hash身份混淆经root校验发现，v1及勘误保留，修正后33项hash全部通过。新完整7类双javac family从创建时覆盖平台CharSequence/Collection/Throwable及自有直接/两跳/接口构造；root同输入全源码回放旧Jarde0/2、JADX none2/2，default2腿因类名观察双流不符。入口为该片results/baseline-freeze-root-verification-v2.json、baseline-cli1-complete-family-root-verification.json与current-proof-seam-audit.md。首轮fmt成功、init focused编译exit101共24项接口迁移错误，失败保留在results/focused-v1，集成和CLI未执行；Luna修复中。完成值定义于constructor而非new，root要求准确NewSite追溯；负控制还需补齐，不能把fixture原程序和JADX通过算作候选成功。

[71单元账本](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md)的71是验收单元数，612是JADX测试文件数，均不是成功率。每片明确完整正例、保守拒绝、JADX真实失败与待扩验范围；不要恢复漫无目的的随机巡查。

TWR另有真实javac8两资源未闭合：[当前root复核](openspec/changes/recover-twr-javac8-close-sequence/verification-root-current.md)。javac23 --release8两资源成功不能代替真实javac8两资源；不要把旧的剥离成员结果计作整类成功。泛型剩余getter反向依赖、容器/wildcard、任意alias/phi、继承/跨类等边界另片处理。共享Signature缓存与物理事实复制计费是架构债务，不混入本片。

## 主线、工作树与构建纪律

未新建分支或工作树。14个辅助工作树均detached、干净、提交为main祖先，无剩余工作、无分支占用且无target；只有main/origin/main，受Codex保护的副本保留。实际审计见新片results/worktree-audit-before-commit.json。

只允许root在主仓串行Cargo/Git/rustfmt；环境CARGO_BUILD_JOBS=1、CARGO_INCREMENTAL=0、RUST_TEST_THREADS=1。20GiB为停建线，验收后清Cargo；保留冻结输入、完整源集和真实双流/exit证据。不动其它项目target，不删拒绝成员，不借原jar编译或执行渲染代码。

引用槽片本地最终门禁已全通过：两seed各349targets、3311passed/0failed/93ignored，MSRV1.88、CI-exact Clippy、显式Java对照、strict及diff均exit0。Cargo清理10177文件、17.8GiB，可用空间约51.5GiB，当时主仓与fuzz均无target；实际清理JSON位于引用槽片results/cargo-clean-final-v1.json。EM18最终两seed各350targets、3319passed/0failed/93ignored，MSRV、fmt、Clippy、显式Java对照、strict及diff均通过。第一次旧varargs拒绝断言失败与首次census旧计数失败均保留，完整V3独立对照后迁移正确正例断言。两seed后磁盘触停建线，清理5323文件/18.3GiB再完成余下门禁；最终已再次清理，主仓与fuzz均无target，冻结CLI保留，清理后可用约39.2GiB。实际索引、原始双流hash与清理JSON见EM18 results/local-gates-root-verification.json、cargo-clean-mid-gates.json与cargo-clean-final.json；实时空间以df为准。
