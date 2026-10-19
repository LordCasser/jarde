# Jarde 接续入口

更新：2026-10-19。这里只记录当前交接状态；旧检查点已移入 [历史快照](docs/history/handoff-through-2026-10-19.md)。其他机器接续先拉取 GitHub 并核对工作区。

## 已交付与证据

主线为 `main`。最近产品提交为 `c8af822ef6d929f8ad1221f58d02e27669c63fc7`，证据/文档后继为 `0a697fc3ec80d88fd0d9e8ba9b0f85492c194afc`；当前 HEAD 以 Git 为准。后续纯文档提交不代表重新运行产品测试或 CI。

finThrow（5/5）、nestedFin（16/16）已完成，不重做。最近完成 `bound-loop-header-chains-at-body-effects`（6/6）：AND 后续候选遇到本块效果时保留 body 边界，复用已有 Region/If/LoopBreak，未新增恢复机制。

- [产品验收总入口](openspec/changes/bound-loop-header-chains-at-body-effects/results/README.md)：定向 Java lib 521 通过；一次 fresh workspace 为 392 targets、3816 通过、98 既有忽略；[产品自身 CI](https://github.com/LordCasser/jarde/actions/runs/39210601042) 四个 job 成功，双 seed 具名结果一致。
- [ComplexIf 完整原件基线](openspec/evidence/java-syntax-2026-10-19/cf09-testbreakincomplexif-original-root-v1/README.md) 与上述产品最终矩阵：16 个隔离 VM、144 次观察均与原件一致，目标来源映射 32/33。
- [ComplexIf2 最终对照](openspec/evidence/java-syntax-2026-10-19/cf09-testbreakincomplexif2-fixed-root-v1/README.md)：同产品，16 个隔离 VM、176 次观察均与原件一致，映射 33/34，metadata 无差异。两矩阵均保留原 JDK8 framework 55/52 编译边界。

最终 CLI：`/private/tmp/jarde-loop-header-body-boundary-focused-root-v5-20261019/candidate-cli`，64,287,728 bytes，模式 0555，SHA-256 `52ff01f7794c962f2fbb9d809612c0cc371ef297de8cb32a29629323d168cbca`。源码/构建来源与产品 Git blob 已绑定；验收归档保留在原路径，不搬迁或改写。

## 正在处理：TestBreakInLoop2

[完整基线与验收](openspec/evidence/java-syntax-2026-10-19/cf09-testbreakinloop2-original-root-v1/README.md) 已完成：46 个命令、8 个 API、20 次完整源编译尝试。Jarde 目标 8 次因 `jre_region_ownership_overlap` 拒绝，BCI 0 有重复 owner；16 次 Jarde 整类重编均因缺少 `jarde_refused_body` 失败，尚无 Jarde VM 对照。17/17 映射属于 bytecode quote，不能当成结构恢复。

原件/JADX 各 8 次观察相同。原件 `funcB` 恒返回 false，因此两个输入只覆盖正常 break；catch、clear、sleep 动态不可达，不宣称这些路径已被运行验证。

[同轮 IR 实测与独立复核](openspec/changes/recover-branching-named-catch-retry-loops/results/same-run-ir-root-v1/README.md) 已完成：精确单测 1/0，两模式均无自然循环，实际首拒绝 check_20；handler 的重试分支再次进入保护区，最终 owner 为 `[0,9,17,19,0,9]`。临时源码精确恢复，target 已清理。这是根因证据，尚非修复验收。

当前 OpenSpec：[recover-branching-named-catch-retry-loops](openspec/changes/recover-branching-named-catch-retry-loops/tasks.md)，1/6，strict 通过。复用现有异常 retry 证明，只增加一次 handler 条件分叉、单块 throw 叶与单块 retry 叶的闭合证明，不新增 CFG/Region/Builder 机制，不放松最终 overlap 检查。

Luna 正在私有准备实现与测试，root 尚未集成候选：`/private/tmp/jarde-loop2-retry-impl-luna-v1-20261019`、`/private/tmp/jarde-loop2-retry-tests-luna-v1-20261019`。下一步审查精确边、owner/transfer 次数及分支内 continue，再受控构建；原件 catch 不可达的部分必须用独立可达控制例补验。不得把探针 1/0 当作产品实现通过。

## 后续队列与独立债务

固定 13 原件中，6 项已完整接受，Loop2 为 1 个实证缺口，其余 6 项待完整基线；CF09、71 单元、612 文件整体仍为 partial。覆盖范围以 [JADX 账本](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md) 为准，不把原件数当作特性数。

- [BreakInLoop3–6 静态计划](openspec/evidence/java-syntax-2026-10-19/cf09-breakinloop3-to6-static-scope-root-v1/README.md)：4/5 优先做完整 Java 基线；3 含 NYI，先确认实际 JADX；6 单列 Smali/Dex 与 class/JAR 输入边界。均未算覆盖完成。
- [隐式循环转移映射债务](openspec/evidence/java-syntax-2026-10-19/cf09-implicit-loop-transfer-mapping-debt-root-v1/README.md)：BCI 69/78 缺失原因仍须实际证明，不混入 Loop2 修复。
- collector 的参数、物理库存与语义投影统一属于工具债务，单独处理。

## 工作与资源边界

root 独占共享写入、Git/网络及 Cargo/JDK/JADX/Jarde CLI；Luna 只读共享文件，在新私有 tmp 用 Python `-B` 准备实现和审计，不建分支或 worktree。实现按 OpenSpec 派发，root 做架构、失败路径及完整对照验收。

Cargo 串行运行：可用空间至少 5 GiB、target 上限 1 GiB、默认 Rust stack、每秒进程组守卫、finally 精确清理。Cargo 期间不得共享写入、Git mutation、网络或手动并行 JDK/CLI。执行 shell 使用 `login:false`；单独检查命令退出码，不能让后续命令掩盖失败。

文档整理前的 clean-main 审计在 `/private/tmp/jarde-loop-header-body-boundary-final-clean-main-root-v1-20261019/execution.json`：主工作区干净，14 个辅助 worktree 均 detached、干净且已被主线包含，无本项目活动工具链，target 不存在。此为历史检查点，接续时重新核对实际状态。
