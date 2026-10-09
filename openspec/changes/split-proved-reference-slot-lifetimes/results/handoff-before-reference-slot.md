# HANDOFF — jarde 主线交接

本轮只收尾既有工作树和同类泛型调用里程碑。先用下面命令核对当前主线；历史源码、失败日志与冻结输入都保留在 OpenSpec 中。

```sh
git status --short
git log -1 --oneline
git rev-parse HEAD origin/main
git branch -av
git worktree list
gh run list --workflow CI --commit "$(git rev-parse HEAD)" --limit 3
```

## 当前交付状态

本轮同类泛型调用消费位里程碑已完成，OpenSpec9/9，所有既有工作树均无剩余实现待合入。实现提交 `7172fdcb` 的实际MSRV1.88失败已由 `276a1c31` 等价语法修复；该实现提交的 [CI 37889792696](https://github.com/LordCasser/jarde/actions/runs/37889792696) 四job全部成功，两个seed各348targets、3304passed/0failed/93ignored，真实JDK25 oracle、显式Java对照、Clippy和strict OpenSpec均通过。最终交接提交的HEAD与CI按顶部命令查询，具体实现CI证据见 `results/root-input-audit/ci-276a1c31-root-verification-v3.json`。本文件不将文档提交冒充实现快照，也不宣称整个generic单元或全部JADX语法已追平。

多轮接续历史保留在本片 `results/handoff-history-before-clean-v41.md` 与 `verification-root.md`，历史失败不覆盖最新验收。本地完整命令、日志和exit见 `results/local-gates/final-gates-v41-index.json`。

## 本轮已完成的源码范围

`recover-same-class-generic-call-consumers` 复用 Signature 擦除、同次 AST/Code/SSA、InitRecord 与原 method→field publication。直接 T/T[]、bounds/多参数/宽槽、独立 method binder、无环有限 relay 与逆声明序、void/调用结果参数位、Object() 后普通 catch 构造正文、封闭 overload 的 Number 上转型，按完整入边及最终源码类型共同验证后原子发布。

事务只保存八个可变源码投影字段。独立已证 abstract NoBody、具体无 TypeVariable 声明和 constructor 自身 method formal 保留原路由；stop/cancel 沿现有 Partial/diagnostic，未提交组移动回退，此前完整独立 commit 保留。没有新增 crate、parser、IR pass、fixpoint、容器或跨类推断。

## 验收与证据入口

- [本片 root 验收](openspec/changes/recover-same-class-generic-call-consumers/verification-root.md)、[任务](openspec/changes/recover-same-class-generic-call-consumers/tasks.md)、[设计](openspec/changes/recover-same-class-generic-call-consumers/design.md)。
- 最终CLI9：`/private/tmp/jarde-generic-calls-candidate-v9-cli`，SHA256 `5bb2fcfdcf958839f0b1e060a2e55114510c6115cb225ec3279d7d7adf95e006`；真实八文件v42 archive SHA256 `fc4220ac5eb6906a786340a6ef5f8dd840ccbc14790df7bcee115f7c9f78fce8`，详见 `results/local-gates/candidate-cli-v9.json`。CLI8/v41完整编译/行为/API证据继续保留。
- 真 Corretto8/OpenJDK23 × debug/no-debug：140 主矩阵 + Nested4，完整重编/行为 144/144，完整泛型 API 96/144。基线完整重编72/144、API20/144。JADX 的真实失败照实保留。
- 44 控制 root 逐输入、完整源和实际 Probe 核对；九族可靠拒绝、pure bridge/合法条件单invoke 两族正控制。拒绝不计 API 恢复。
- GC09：23 字段族双腿46输入，44完整编译/行为、26完整API，SCGB两既有拒绝；80构造行为、36完整API；raw64行为，字段/方法/类API60/52/60，旧已接受API零回退。
- CLI9实际重放324条冻结输入请求，完整source/report/exit与CLI8一致，仅usage.elapsed_millis不同；18个旧临时字段jar已缺失，按原JDK/参数重建后class哈希18/18匹配，完整源码亦与CLI8一致，并补做原/baseline/JADX/CLI9完整编译、行为/API。新jar永久保存在 `results/candidate/field18-msrv-final`，不宣称zip字节与旧jar一致。root验收入口 `results/root-input-audit/final-msrv-acceptance-v45.json`。
- 独立验收 JSON、实际 runner、命令双流/退出码、前后hash在本片 `results/root-input-audit`。完整门禁见 `results/local-gates`，历史各轮失败保留；不借原jar执行，不删除拒绝成员。

## 下一位 agent 从哪里继续

先按 [71 单元账本](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md) 选一个明确未闭合的家族，结合 `/Users/lordcasser/workspace/testzone/jadx` 的具体测试与生产算法确定完整源码/原class/JADX/Jarde差异，再写 OpenSpec。确定性实现用 Luna，root 独立做边界审阅、完整对照与验收。71 是验收单元数，不是成功率；本片不意味着整个泛型单元或全部 JADX 语法追平。

剩余泛型边界：getter 对字段发布的反向依赖、容器/wildcard、任意 alias/phi、继承/跨类泛型、this/super 委派和完整成员类 TestGeneric8。SCGB 保留的正文拒绝也单独处理。共享 Signature 解码/缓存、物理事实复制计费是另片架构债务，不混入下一语法修复。

此前检查的 ScopeRefusalsEscape/sharedHandler、LoopTestValues.storeTest 参数来源和 switchBody guard 均已审为拒绝，不再列作候选。后续优先核查 LG 的 Ref→Ref slot reuse 与 finalize 分片，先以实际实现和 JADX 路径界定最小闭环；这不表示完整 LG 局部类型或 JADX baseline 已通过。

## 构建纪律

主仓共享 Cargo 只允许一个 agent 操作：`CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1`。20 GiB可用空间为停建线，验收后清理。冻结输入jar和失败/成功证据保留，生成class与Cargo残留及时清理；不动其他项目的target。

主仓Cargo已再次clean，本轮两次共移除约21GiB编译残留，当前无主仓target/fuzz target。磁盘及清理日志见 `results/root-input-audit/cargo-clean-final-v41.json`、`cargo-clean-final-v45.json`。

14个辅助工作树均detached、干净且提交为main祖先，无待合入工作或分支占用。Codex保护的副本保留，不在这些树另起构建。
