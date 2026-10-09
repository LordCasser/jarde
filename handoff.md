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

root发现并修正了“仅查store直接uses会漏掉aload后的旧栈值”问题；HeldUse以及两javac的非零栈checkcast控制都直接验证命名元数据未发布分段。初候选的失败、最终完整32腿、10个负控制及逐字段source-map审查均永久保留。[任务](openspec/changes/split-proved-reference-slot-lifetimes/tasks.md)、[实现与真实SSA](openspec/changes/split-proved-reference-slot-lifetimes/results/implementation-notes.md)、[最终CLI2元数据](openspec/changes/split-proved-reference-slot-lifetimes/results/candidate-cli-v2.json)、[门禁实际index](openspec/changes/split-proved-reference-slot-lifetimes/results/gates-v3/index.json)是入口。最终CI和清理记录见本片root验收，HEAD状态仍以顶部命令为准。

此前同类泛型调用已合入主线，9/9任务完成，历史root验收在 [recover-same-class-generic-call-consumers](openspec/changes/recover-same-class-generic-call-consumers/verification-root.md)。本片开始前主线82090226的CI37892494665四job成功。旧handoff完整保存在新片results/handoff-before-reference-slot.md。

## 下一片：EM-18异构数组初始化

[recover-heterogeneous-array-init](openspec/changes/recover-heterogeneous-array-init/)已有计划但尚未实现。当前CT.cov的Number[]中Integer/Long元素因精确类型比较被拒；up/io/io2已恢复。最小入口为Builder::array_initializer_element，读取既有platform_interface_argument_widens中的封闭boxed→Number事实后保留元素表达式，不新增层级、LUB、cast或数组恢复机制。

先使用openspec-update-change修正规划：旧platform_reference_argument_widens名称和“任意子类型”承诺过时；组件由anewarray/Operation::NewArray陈述，aastore只陈述引用存储。以真实两javac整类CT重编/运行、已有数组控制及有来源的拒绝边界验收。保持物理元素求值次序，不照搬JADX按下标排序的表面实现；TestArrayFill2.test2的NYI是int后缀副作用，不能作为Number[]同一锚。已有只读审计草稿为/private/tmp/em18-next-architecture-audit.md，实施前root复核当前代码。

[71单元账本](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md)的71是验收单元数，612是JADX测试文件数，均不是成功率。每片明确完整正例、保守拒绝、JADX真实失败与待扩验范围；不要恢复漫无目的的随机巡查。

TWR另有真实javac8两资源未闭合：[当前root复核](openspec/changes/recover-twr-javac8-close-sequence/verification-root-current.md)。javac23 --release8两资源成功不能代替真实javac8两资源；不要把旧的剥离成员结果计作整类成功。泛型剩余getter反向依赖、容器/wildcard、任意alias/phi、继承/跨类等边界另片处理。共享Signature缓存与物理事实复制计费是架构债务，不混入本片。

## 主线、工作树与构建纪律

未新建分支或工作树。14个辅助工作树均detached、干净、提交为main祖先，无剩余工作、无分支占用且无target；只有main/origin/main，受Codex保护的副本保留。实际审计见新片results/worktree-audit-before-commit.json。

只允许root在主仓串行Cargo/Git/rustfmt；环境CARGO_BUILD_JOBS=1、CARGO_INCREMENTAL=0、RUST_TEST_THREADS=1。20GiB为停建线，验收后清Cargo；保留冻结输入、完整源集和真实双流/exit证据。不动其它项目target，不删拒绝成员，不借原jar编译或执行渲染代码。

本片本地最终门禁已全通过：两seed各349targets、3311passed/0failed/93ignored，MSRV1.88、CI-exact Clippy、显式Java对照、strict及diff均exit0。Cargo清理10177文件、17.8GiB，可用空间约51.5GiB，主仓与fuzz均无target；实际清理JSON位于本片results/cargo-clean-final-v1.json。
