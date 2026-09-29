# HANDOFF — jarde 接续说明（2026-09-30）

本文件是给接续 agent 的入口。先确认下面的 Git 状态，再决定是否开始新工作；不要从旧分支名推断仍有未合入实现。

## 当前状态

- 13 条被工作树占用的历史分支已在 `aeb18c0f` **全部合入并推送到 `main`**；该合并的文件树与合并前 `056709b8` 完全一致。随后释放全部分支占用并删除旧引用；本地只剩 `main`，远端只剩 `origin/main`。本文档提交后以 `git status` 和 `git rev-parse HEAD` 核对新的仓库状态。
- 长期 Java 语法恢复 `/goal` 当前状态为 **paused**。用户恢复该目标前，不自行派发新语法点或继续无边界巡查。本文件记录接续路径，不代表目标已恢复。
- 最近一次已确认的代码 CI 是 [GitHub Actions 36606512666](https://github.com/LordCasser/jarde/actions/runs/36606512666)，所有 job 成功。它验证的是 `fcc864ce` 的代码；后续 `056709b8` 仅增本文档，`aeb18c0f` 仅合并历史、文件树无变化。新提交的 CI 状态需单独查看。
- 根目录 Cargo `target` 已清理约 10.4 GiB；先前隔离构建产物清理约 103 MiB，本次又从 `dt13-nested-enum` 清理 158 MiB。后续测试会重新占用磁盘，Rust 工作结束后留意 `target`。

## 用户确定的工作方向

先以本地 `/Users/lordcasser/workspace/testzone/jadx` 的测试和实现为基础，明确语法特性清单，逐项追平 **可证明正确** 的 JADX 已有能力；之后再探索双方都未覆盖的情况。可以参考 JADX 的反编译代码和算法，避免重复试错，但不能照搬其错误转写。对每个语法点：读相关测试和生产 visitor/region 实现，构造 Java 源码并编译，对照原 class、JADX、Jarde 的完整源码、Java 8 重编和执行；确认差距与 JVM/架构证据后写独立 OpenSpec。确定性、简单的实现任务派 Luna subagent（按难度调整思维强度），由主 agent 独立重放验收。同批互不冲突的点可并行。不要漫无目的地追加场景，也不要把一个窄 fixture 通过称为整个特性追平。

入口是 [JADX 特性清单](openspec/evidence/jadx-feature-inventory-2026-09-27/README.md)、[71 个验收单元与状态账本](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md)、[路线图中的 Java 8 对标章节](openspec/roadmap.md)。清单基于 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 612 个集成测试文件，去重得到 71 个工程验收单元；这个数字不是已追平数。状态账本目前记载 46 个“冻结差距已修复但待扩验”、23 个“部分已测”、1 个“已证差距”（CF-16 的剩余 `finally` 形态）、1 个“JADX 未完成”。这些是文档最后登记的状态，恢复工作时先复核源码和最新主线，避免按旧统计重复开工。

架构约束：理解现有 reader → CFG/SSA → Region/AST → 类级装配与报告的证据流后再改代码；如非必要勿增实体，不考虑后向兼容。先判断是已有证明路径未消费、证据不足，还是确需新机制。与当前语法点无关的架构债务单独记录和拆分。JADX 的测试源码、文本相似度或可编译输出，都不能代替原 class 的行为证明。具体误判和拒绝边界见上述清单与各单元证据。

## 最近一次主线收口

用户要求先提交、推送全部当前工作，并把其它分支工作合入 `main`，随后清理不再需要的分支。审计 158 个非主线本地分支及工作树后，确认大多数历史提交已由主线后续或等价实现覆盖；直接重放旧提交会倒退代码。实际缺少的只有两项 `finally` 恢复：

- `80523785`：固定 catch 值字段的 `finally`（原提交 `2c694adc`）。
- `fcc864ce`：固定可空资源的 `finally`（原提交 `fa790c17`）。

两项冲突在 `crates/jarde-java/src/build.rs` 和 `crates/jarde-java/tests/p3_shared_join_finally.rs` 合并时已保留双方证明与 Test7/Test9 回归。主线已推送。验收包括 `cargo fmt --all -- --check`、OpenSpec strict 216/216、`p3_shared_join_finally` 30/30、整仓两颗固定 seed 测试，以及 CI 等价的 Clippy；上述 GitHub CI 同时通过 stable、JDK 25 oracle/P3、MSRV、fuzz 与 supply-chain job。更早的 JDK 25 CI 修正也已在主线，见 [CI 36377418834](https://github.com/LordCasser/jarde/actions/runs/36377418834)。

先清理了 146 条不再需要且未被检出的本地分支。随后重新审计其余 13 条工作树分支：17 个独有历史提交中，12 个有主线上相同 patch-id；另外 5 个不同 patch-id 的功能、测试或证据由主线后续实现覆盖，分支新增路径在主线没有缺失。逐条核对后，用一次 `ours` 合并记录这些**已被主线内容覆盖**的分支历史，没有重放过时代码；`aeb18c0f` 的 13 个分支 tip 均已成为 `main` 祖先，且 `git diff 056709b8 aeb18c0f` 为空。本次临时集成分支已删除。

14 个辅助工作树（13 个旧分支 checkout 加一个原本 detached 的 checkout）已逐个确认没有未提交、未跟踪文件或使用它们的本地进程，然后统一 detached 到当前 `main`；13 条旧本地分支均以普通 `git branch -d` 删除。辅助工作树目录仍在，但全部干净、不占用分支，也没有未合入的工作。Codex 归档接口对当前任务附着的工作树返回“protected by a pinned task or workspace”；没有绕过保护或直接删除目录。需要物理回收目录时，先解除其 Codex 固定/归属保护，再用工作树归档工具处理。并发工作不要在根 checkout 随意切换分支；优先复用空闲的隔离工作树，操作前重查状态。

## 接手时的最小核对

```sh
git status --short --branch
git rev-parse HEAD
git worktree list --porcelain
git branch -a
```

若用户恢复语法目标，从状态账本选一个**尚未闭合的具体子形态**，重新读对应 JADX 测试/算法和 Jarde 当前代码，再冻结三方正反例。按现有 OpenSpec 写窄任务、实施、Java 8 完整类重编与验证运行、来源/拒绝边界及主 agent 验收；每完成一个切片就回写账本。CF-16 是已登记的剩余真实差距，但它的多个首片已修，不能仅凭编号重做。旧工作树均无剩余实现任务，也不需要重新合并；物理目录的后续归档受 Codex 固定工作区保护约束。
