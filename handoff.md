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

CLI8/v41完整对照与本地门禁已验收，OpenSpec8/9，全部代码和证据已提交到 `7172fdcb`。该提交实际CI的MSRV1.88失败：`report.rs`用了1.88不支持的if-let match guard。当前仅改为等价内层match；本地真实 `rustup run 1.88.0 cargo check --workspace --all-targets --locked` 已通过。报告单测、新CLI输出等价核验及修复后的最新HEAD四job CI仍需完成。失败日志与v42本机Cargo入口失败均保留，不能写成已完成远端交付。

多轮接续历史保留在本片 `results/handoff-history-before-clean-v41.md` 与 `verification-root.md`，历史失败不覆盖最新验收。本地完整命令、日志和exit见 `results/local-gates/final-gates-v41-index.json`。

## 本轮已完成的源码范围

`recover-same-class-generic-call-consumers` 复用 Signature 擦除、同次 AST/Code/SSA、InitRecord 与原 method→field publication。直接 T/T[]、bounds/多参数/宽槽、独立 method binder、无环有限 relay 与逆声明序、void/调用结果参数位、Object() 后普通 catch 构造正文、封闭 overload 的 Number 上转型，按完整入边及最终源码类型共同验证后原子发布。

事务只保存八个可变源码投影字段。独立已证 abstract NoBody、具体无 TypeVariable 声明和 constructor 自身 method formal 保留原路由；stop/cancel 沿现有 Partial/diagnostic，未提交组移动回退，此前完整独立 commit 保留。没有新增 crate、parser、IR pass、fixpoint、容器或跨类推断。

## 验收与证据入口

- [本片 root 验收](openspec/changes/recover-same-class-generic-call-consumers/verification-root.md)、[任务](openspec/changes/recover-same-class-generic-call-consumers/tasks.md)、[设计](openspec/changes/recover-same-class-generic-call-consumers/design.md)。
- 固定 CLI8：`/private/tmp/jarde-generic-calls-candidate-v8-cli`，SHA256 `8770d823c70e7f61749cac836e468c0a991093822d1926822d4769adc1cf7339`。源码真实八文件 v41 archive SHA256 `d2367f1cc6a63c0c99804412a7d4f84c7c97eca3807987eab36828e40f97df1d`，详见 `results/local-gates/candidate-cli-v8.json`。
- 真 Corretto8/OpenJDK23 × debug/no-debug：140 主矩阵 + Nested4，完整重编/行为 144/144，完整泛型 API 96/144。基线完整重编72/144、API20/144。JADX 的真实失败照实保留。
- 44 控制 root 逐输入、完整源和实际 Probe 核对；九族可靠拒绝、pure bridge/合法条件单invoke 两族正控制。拒绝不计 API 恢复。
- GC09：23 字段族双腿46输入，44完整编译/行为、26完整API，SCGB两既有拒绝；80构造行为、36完整API；raw64行为，字段/方法/类API60/52/60，旧已接受API零回退。
- 独立验收 JSON、实际 runner、命令双流/退出码、前后hash在本片 `results/root-input-audit`。完整门禁见 `results/local-gates`，历史各轮失败保留；不借原jar执行，不删除拒绝成员。

## 下一位 agent 从哪里继续

先按 [71 单元账本](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md) 选一个明确未闭合的家族，结合 `/Users/lordcasser/workspace/testzone/jadx` 的具体测试与生产算法确定完整源码/原class/JADX/Jarde差异，再写 OpenSpec。确定性实现用 Luna，root 独立做边界审阅、完整对照与验收。71 是验收单元数，不是成功率；本片不意味着整个泛型单元或全部 JADX 语法追平。

剩余泛型边界：getter 对字段发布的反向依赖、容器/wildcard、任意 alias/phi、继承/跨类泛型、this/super 委派和完整成员类 TestGeneric8。SCGB 保留的正文拒绝也单独处理。共享 Signature 解码/缓存、物理事实复制计费是另片架构债务，不混入下一语法修复。

其他旧队列继续按账本证据排序：ScopeRefusalsEscape 合法原形、LoopTestValues.storeTest 实际源码、switchBody guard、完整 LG 的局部类型复用/finalize。不要从零散 exploratory 样例直接推定一个语法单元全部覆盖。

## 构建纪律

主仓共享 Cargo 只允许一个 agent 操作：`CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1`。20 GiB可用空间为停建线，验收后清理。冻结输入jar和失败/成功证据保留，生成class与Cargo残留及时清理；不动其他项目的target。

14个辅助工作树均detached、干净且提交为main祖先，无待合入工作或分支占用。Codex保护的副本保留，不在这些树另起构建。
