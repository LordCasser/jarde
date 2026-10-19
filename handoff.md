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

下一动作是运行已审过的 [同轮 IR 探针准备](openspec/evidence/java-syntax-2026-10-19/cf09-testbreakinloop2-ir-probe-preparation-root-v1/README.md)。私有候选在 `/private/tmp/jarde-testbreakinloop2-ir-probe-prep-luna-v2-20261019`，已静态复核 27 个 pin，尚未编译/运行，尚无修复 OpenSpec。

必须确认实际 Catches、`retry_loop_facts` 首拒绝、Frame 与最终所有权检查前的完整树，再决定最小修复。当前关于有分支 handler 回到保护区入口的解释只是静态假设。现有 `Loop.blocks()` 已区分 Endless header/body，不能靠全局去重隐藏所有权问题。

探针使用已有 `recover-branch-throw-return-finally/tools/run_validation.py` 的裸 Cargo 精确命令；`JRE_LOOP2_IR_PROBE=1` 仅加在私有 runner 子进程环境，记录实际继承范围。执行前与 finally 复开外部 class、baseline、review、runner 和源码 pin。结束后精确恢复 `region.rs`（SHA-256 `5da4d9f40e0892ac134bb42fb1508fffad090b92338a86b9403efd47b7977510`）、移除临时模块，确认 target 不存在。异机须恢复归档并重新审查路径和 pin。

## 后续队列与独立债务

固定 13 原件中，6 项已完整接受，Loop2 为 1 个实证缺口，其余 6 项待完整基线；CF09、71 单元、612 文件整体仍为 partial。覆盖范围以 [JADX 账本](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md) 为准，不把原件数当作特性数。

- [BreakInLoop3–6 静态计划](openspec/evidence/java-syntax-2026-10-19/cf09-breakinloop3-to6-static-scope-root-v1/README.md)：4/5 优先做完整 Java 基线；3 含 NYI，先确认实际 JADX；6 单列 Smali/Dex 与 class/JAR 输入边界。均未算覆盖完成。
- [隐式循环转移映射债务](openspec/evidence/java-syntax-2026-10-19/cf09-implicit-loop-transfer-mapping-debt-root-v1/README.md)：BCI 69/78 缺失原因仍须实际证明，不混入 Loop2 修复。
- collector 的参数、物理库存与语义投影统一属于工具债务，单独处理。

## 工作与资源边界

root 独占共享写入、Git/网络及 Cargo/JDK/JADX/Jarde CLI；Luna 只读共享文件，在新私有 tmp 用 Python `-B` 准备实现和审计，不建分支或 worktree。实现按 OpenSpec 派发，root 做架构、失败路径及完整对照验收。

Cargo 串行运行：可用空间至少 5 GiB、target 上限 1 GiB、默认 Rust stack、每秒进程组守卫、finally 精确清理。Cargo 期间不得共享写入、Git mutation、网络或手动并行 JDK/CLI。执行 shell 使用 `login:false`；单独检查命令退出码，不能让后续命令掩盖失败。

文档整理前的 clean-main 审计在 `/private/tmp/jarde-loop-header-body-boundary-final-clean-main-root-v1-20261019/execution.json`：主工作区干净，14 个辅助 worktree 均 detached、干净且已被主线包含，无本项目活动工具链，target 不存在。此为历史检查点，接续时重新核对实际状态。
