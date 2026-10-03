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

**验收门禁口径（2026-09-30 CI 36678484547 教训；10-01 增补）**：CI 的 "stable / test and specification" 跑 `cargo test --workspace --all-targets --all-features --locked` 双固定 seed、ignored 的 JDK oracle/P3 对照与两个 example。实现任务的验收与 root 复核必须同口径跑整仓命令（注意 `--all-targets` 含 benches/examples，宽于 `--tests`）；只跑 `-p jarde-java --tests` 会漏掉根 crate（如 `enum_constants` 的投影消费方）——恢复层改进（例：`array_of_value` 使增强 for 可证）会改变下游折叠器的输入形状，消费方期望不同步即回归。**Clippy 门禁必须从 `.github/workflows/ci.yml` 逐字复制整条命令**（2026-10-04 root 实测教训）：本地常引的精简 `-A` 清单会漏报，而**漏掉 `--all-features` 会误报**——root 首跑 `cargo clippy --workspace --all-targets --locked -- <29 项 -A> -D warnings` 时报出 11 个 `unit_arg`/`let_unit_value` 错误于 `p5_optimize_workloads`（一个未被当次改动触碰的文件），加回 `--all-features` 后同命令 exit 0、Finished 干净。可靠做法是**用脚本从 workflow 生成命令**，不手抄：

```sh
sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' ' > /tmp/ci-clippy.sh && sh /tmp/ci-clippy.sh
```

当前 CI 清单为 **29 项** `-A`（`grep -oE '\-A [a-z_:]+' .github/workflows/ci.yml | sort -u | wc -l` 实测；旧文所载"30 项"已过期，勿再引用）。且 CI 的 1.98.1 与本地 1.98.0 存在 patch 版 lint 差（实例：`iter_cloned_collect`，CI-only 报出，b27e0443 修复）——本地全绿不等于 CI clippy 绿，推送后必须核对 CI。

**合成成员消隐的前置不变量（2026-10-04 root 以 javac 实证确立，适用全部消隐片）**：隐藏 javac 合成成员（`access$NNN`、lambda 伴生 `lambda$x$N`、擦除桥、`$SwitchMap` 辅助类）的**唯一合法性来源**是"源码自身能让 javac 重新生成同一合成物"，而不是"该成员在字节码里可证是合成的"。故每个消隐片都必须回答：**javac 重编时靠什么重建它？**
- 擦除桥 → 靠**类头的类型实参投影**（`implements Comparable<Impl>`）。实证：裸 `implements Comparable` + 隐藏桥 → javac 报"未覆盖 compareTo(Object)"；参数化 `implements Comparable<ParamI>` + 隐藏桥 → javac 自行重建桥（`javap -v` 实测 ACC_BRIDGE=1、exit 0）。证据 `openspec/evidence/java-syntax-2026-10-04/bridge-method-patrol/header-invariant/`。
- lambda 伴生 → 靠**调用点的 invokedynamic 站点**仍在源码中（已闭合，`recover-lambda-inline-bodies`）。
- `access$NNN` → 靠**嵌套类族在同一文本内平铺**（javac 为跨类私有访问重新合成；已闭合，instance-folding 片）。
**推论**：消隐决策的 owner 是"该合成物的重建前提由谁提供"，前提缺失时必须**保持合成物可见**（响亮失败）而非隐藏——否则会产出"信息丢失 + 仍不可编"的更差中间态。owner 分离示例：类头文本投影归 `recover-parameterized-interface-headers`，桥消隐决策归 `recover-bridge-admission-gates`，二者以"类头是否带类型实参"这一事实为共享契约。

**改序类切片的验收纪律（2026-10-04 回归教训，强制）**：任何改变指令/语句**顺序**的切片（重排、移动、提前、延迟），验收必须 (1) 跑既有 order-sensitive 反例 fixture——本项目现有两个：`tests/fixtures/proved-java-structure/anonymous-super-dispatch/`（构造期虚分派，`Base()` 在 super() 中虚调用覆写方法读捕获字段；重排捕获写入会使 `visibleDuringSuper` 由 true 变 false）与 `openspec/evidence/java-syntax-2026-09-25/ordinary-new-void-effect/`（构造实参 CST 序）；(2) 检查同域既有 change 的**未勾任务**是否已声明该风险（本例 `present-proved-java-structure` 2.10 自 2026-09-25 就明写"旧任务所称'改序不损失任何效果'已被运行反例否定"）；(3) 验收判据必须包含"不得产出可编译且行为不同的文本"——`recover-return-in-do-while-false` 已把该不变量写进 spec，改序类切片一律适用。失误实例：`recover-synthetic-ctor-super-order`（daa4fb31，root 验收）重排 pre-super 合成字段存，未查该反例 → 静默行为回归；修复片 `recover-ctor-reorder-dispatch-guard`（判据加 super 目标 == `Object.<init>`）。安全判据的一般形：**移动跨越"可能虚分派/可能触发副作用"的调用边界时，只有目标不可能产生该副作用才可移动**。

**立项查重（2026-10-04 失误教训，强制）**：写新 OpenSpec change 前**必须先查重**——`ls openspec/changes | grep -i <域关键词>`（含英文与机制名，如 assert/bridge/enum/loop/literal），再读同域 change 的 `proposal.md` 范围与 `tasks.md` 勾选状态（`grep -c '^\- \[x\]'` vs `'^\- \[ \]'`）。原因：仓库有 240+ change、大量"已立未实施"（如 `project-proved-enum-switch-labels` 1/9、`present-proved-java-structure` 51/91）与"已实施未勾"，巡查发现的缺口**常常早已被既有 change 精确描述**。正例：2026-10-04 巡查桥方法 name-clash 时先查到 `project-proved-bridge-forwards`（机制已合入），改为立"准入门扩展"片而非重复机制；巡查枚举 switch 时查到 `project-proved-enum-switch-labels`，改为证据补强不新立。反例：同日巡查 assert 时**未先查**，新立 `recover-assert-statement-sugar` 并实现，事后发现 `project-proved-assert-statements`（2026-09-26 立）范围高度重叠——功能已交付不回滚，但两处 change 都补记了重复关系并把既有片残留范围收窄为（预算/取消测试、定向测试粒度、root 验收）。**若既有 change 已覆盖：优先补强其证据/收窄其残留范围并派发它，而不是新立同域 change。**

**账本诚实纪律（2026-10-04）**：实现者被配额/磁盘杀死于收尾段而由 root 代收尾时，其已完成的 1.1–3.2 必须**据证据实证后补勾**（读 impl-record、results/、in-crate 测试名核对），并在 tasks.md 追加补勾说明（谁、何时、依据、实现形式差异如"变体以合成 class 生成器实现而非独立 fixture"）——只勾 root 自己的 3.x 会让账本失真（`recover-synthetic-ctor-super-order` 曾因此显示 5 未勾/2 已勾，实际全片已验收）。反之，功能被后立切片覆盖时**不代勾**原 change 的 tasks，而是记录重叠与收窄后的残留范围（assert 域如此处理）。

**磁盘纪律（2026-10-02 用户指令强化）**：subagent worktree 的 cargo target 单份 ~20-23G，多 worktree 并行或主仓同建会迅速打满盘（本会话已三次临界：8.5G/6.7G/3.9G，两次杀死 agent）。规则：(1) 每个 subagent 任务书已含磁盘纪律段——每轮构建/测试前 `df -h /` 检查、低于 12-15Gi 先 `cargo clean`、**每完成一轮全量测试后若接下来是文档/分析等非构建工作，先 clean**、报告前必 clean；(2) root 在合入验收后立即 `git worktree remove`（含 target），不留残留；(3) 主仓自己的验收构建用完即清（`rm -rf target`）；(4) 已知 flake 家族（export_cli 计时、bulk_recovery_delivery 4-worker、observable_equals、backward_second_entry、p4_plugins、engine::standalone、ordinary_generic_projection 临时目录碰撞）重跑判定即可，勿误判回归。**串行派发约束**：因单份 target ~20G，同一时刻只允许一个 subagent 构建；root 巡查期间**复用运行中 subagent worktree 的 HEAD 版二进制**（先核实 `git status` 无 crates/src/tests 改动且二进制 mtime 新于源码，确认忠实反映 HEAD），避免自建 target 造成双 target 并存打满盘。
