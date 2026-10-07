# Tasks

> 纪律：门控实验先行（只加 ifeq/ifne 分支臂）；行号按锚点名重验（`proves_boolean_local_store` 的 `boolean_position` match）。
>
> 实施记录（coder，2026-10-07）：本片在 44d2cb5d（main）的隔离 worktree 实施。两处立案推断被实测更正，见 `proposal.md`「测量更正」与 `results/01-refusal-chain.md`：**循环条件位** `while (b)` 被 gate 的跨词法 region 判据先拒（循环头是自身 canonical block，读取路径 `[1]` ≠ 声明路径 `[0]`），本片保持拒绝并登记边界；**链中段读**不被区域判据先拒，分支臂准入并冻结为正例。

- [x] 1.1 插桩复核拒绝链（OP2.condAssignOld 当前诊断逐字）；门控实验：白名单加分支臂，OP2 翻转、跨 region 负例（合成：catch 内写 try 外读）不翻、既有三消费位锚逐字节。转录存证据。
      → `results/01-refusal-chain.sh` + `01-refusal-chain.out`（HEAD 的 gate 逐调用插桩：OP2/三元/语句/中段=消费者白名单；循环=跨 region 判据；catch 形=更早的 region ownership；`iflt`=白名单 opcode）；`results/01-gating.sh` + `01-gating.out`（37 输入基线/分支臂对照：5 移动——OP2、BranchReads 双腿、`ifne` 控制、`iflt` 控制的未补丁成员；32 逐字节不动，含循环/catch 负例、三消费位锚、scv 全家族、健康控制）。
      实测更正：`catch 内写 try 外读` 形在 region ownership 层就被拒（`canonical block at BCI 18 … has more than one owner`），**未到达 gate**；真正行使 `access.path != write.path` 的负例是循环头读形（`results/gating/branch-read-negatives-v8.*.java`）。
- [x] 1.2 冻结锚与负例双腿：OP2（巡查冻结件）+ if 语句位 / while 条件位 / 三元读位三新锚；负例=跨词法 region 形、短路链中段读形（`x && b` 的 b 在链内——如实记录是否被短路区域判据先拒）。
      → `tests/fixtures/recover-short-circuit-local-branch-reads/`（`BranchReads` 正例：三元/语句/中段 + 可编译的 `main`；`BranchReadNegatives` 边界：循环条件位、catch 跨形；`controls/NumericBranch|NotZeroBranch` 单字节 opcode 补丁；双腿 `v8/`+`v8-javac8/`；README 记 SHA、字节码、补丁偏移、冻结答案）。
      实测更正：中段读**未被**短路区域判据先拒（与三元同路到白名单，臂准入后按嵌套形呈现，求值序不变）→ 归入正例锚；while 条件位被跨 region 判据先拒 → 归入边界负例。
- [x] 2.1 白名单增分支臂（消费者 `ifeq`/`ifne` 读该加载布尔值）；其余判据逐字不动。
      → `crates/jarde-java/src/build.rs` `proves_boolean_local_store` 的 `boolean_position` 增 12 行分支臂（opcode `0x99|0x9a` + `CompareOp::JumpIfZero|JumpIfNotZero` 身份匹配）；其余判据零改动（`git diff` 仅此一处）。
- [x] 2.2 对照测试：四锚恢复（重编+`-Xverify:all` 行为一致，OP2 main 全行）；`recover_short_circuit_local_values`（7/7）+ scv 系列套件零回退；负例拒绝逐字。
      → `tests/recover_short_circuit_local_branch_reads.rs`：默认套件 5 测试（OP2 锚+残余、双腿三条件位、循环/catch 边界逐字、补丁控制宽度、预算）；`#[ignore]` replay 3 测试（OP2 冻结源码 `main` 替换后双腿编译运行与冻结 jar 逐行一致含 `condAssignOld(0)`；`BranchReads` 整类双腿往返一致；边界文本不可编译 + 控制 `-Xverify:all`）。
- [x] 3.1 全门禁（含 oracle ignored 腿）+ corpus 指纹 + 分逻辑提交（不 push）。
      → 见 `results/03-corpus-and-oracle.md`、`results/04-gates.md`、`results/05-fingerprint.md`。
- [ ] 3.2 root 独立复核：门控、白名单最小性、锚/负例实测、账本（census 重跑 OP2 单点关闭）。（留 root）
