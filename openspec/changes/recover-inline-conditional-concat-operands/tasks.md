# Tasks

> 纪律：门控实验先行（定位 `jre_concat_split` 的链所有权判据，只接"中间块=已证条件值物化"，NI 翻转/真跨块负例不翻）；行号按锚点名重验。判别探针已冻结（`../recover-committed-local-multireads/evidence/probes/`，NI/NI2/NMA/NMB/CMP，双腿）。

- [x] 1.1 插桩定位 `jre_concat_split` 发出处的链所有权证明（toString 终端同块判据）；判明中间块的条件值物化能否以 `recover-conditional-values` 的既有两臂证明作子证明（只读复用，不重实现）；门控实验记录。
      → [evidence/01-gating-experiment.md](evidence/01-gating-experiment.md)：发出处 = `concat.rs::verify` 的 split 检查（`concat.rs:882`，逐字未动），HEAD 逐字复现冻结渲染（NI `4b78780a…b534`）。矩阵：只跳过 split 拒绝 **不翻** NI（walk 改判 BCI 28 `jre_concat_interleaved_effect`）；整段 span 强制 owned（模拟链拥有）**仍不翻**——唯一剩余阻塞是 BCI 8 的 `getstatic System.out`（1 真读 + 2 条 join 平凡 Phi 记录 = "3 consumers"）；接上同子证明许可的 cut 条件后翻转。子证明 = `build::prove_conditional_value`（`pub(crate)`，不用 `Region::If::prefix`）**只读复用**：证书按 CFG 陈述 region 交给它重新校验每条边/臂入口/Phi/消费者，无重实现 → STOP 条件不成立。
- [x] 1.2 冻结负例双腿：真跨块副作用链（分支臂含赋值/调用副作用）、双比较嵌套形、跨异常表拼接——各保持 `jre_concat_split` 逐字；NMA/NI2 零回退基线。
      → `tests/fixtures/recover-inline-conditional-concat-operands/`（ICC/ICB/ICQ 锚 + ICM 无分支控制 + ICN 四负例，双腿，README 记 SHA）；`renders/ICN.txt`、`renders/ICM.txt` 与基线逐字节相同（sweep 自检钉住）。

## 2. 实现

- [x] 2.1 实现链所有权判据的子证明接入；`jre_concat_split` 文本与其余判据逐字不动。
      → `concat.rs::verify_conditional_cut_chain` + `plan_conditional_cut_chains`（只在 split 归因拒绝的候选上进入，四条件字符串证书先跑）；证书补两臂块内"只有所推常量与 transfer"、常量为 `0`/`1`、Phi 唯一消费者是本链 `append(Z)`；walk/其余判据逐字未动。链端到端 owned（头块到分支 + 两臂 + join 到 toString），比较值经既有布尔通道（`boolean_position_values`/`equality_argument_parameter`）在 `append` 实参位呈现——`concat_expr` 的布尔位证据因此多读一项"值自身已呈现为 boolean"。
      偏差已记录：同一子证明还许可 **deferred-binding 侧**的第二条件（`Builder::crosses_a_proved_cut` + `binding_consumers` + `shares_a_declaration_region`），否则 BCI 8 的 `System.out` 仍以计数门拒绝整方法（1.1 实验 B 实测）。
- [x] 2.2 对照测试：NI/NMB/CMP 恢复（重编+`-Xverify:all` 行为一致，NI `3/10/5/true`）；负例三形拒绝逐字；`recover_scv_concat_consumers`/`recover_ref_eq_boolean_argument`/`recover_conditional_values` 既有测试零回退。
      → `tests/recover_inline_conditional_concat_operands.rs`：三锚源码形态（`"" + … + (a == b)`）+ 控制逐字节 + 四负例 `jre_concat_split` 逐字（双腿、`--evidence rule_details`）；ignored replay 双腿重编运行 `-Xverify:all` 与原类一致（ICQ 的 identity hash 已注明掩码）。三个既有套件 5/5、2/2、7/7 绿；NMA/NI2 渲染 sha 与冻结值相同。

## 3. 验证与验收

- [x] 3.1 全门禁 + corpus 指纹 + 分逻辑提交（不 push）。
      → [evidence/03-corpus-delta.md](evidence/03-corpus-delta.md)（sweep 10 MOVED 全分类 + 指纹 +85/-0 + census 重测 `(801,3363,288,2095,8)` + oracle ignored 腿 3/3）、[evidence/04-gates.md](evidence/04-gates.md)（权威口径）。三个逻辑提交（机制 / fixture+测试 / 证据+登记），未 push。
- [x] 3.2 root 独立复核：门控实验、子证明只读复用、锚/负例实测、账本（multiconsumer 族重定位后的新缺口关闭）。（root 2026-10-06 完成，见 [verification-root.md](verification-root.md)：门控矩阵复核（终端接受必要非充分的偏差追认为实现）✓、子证明只读复用 ✓、NI 旗舰 root 实测原源码形态 ✓、oracle ignored 3/3（新纪律）✓、门禁权威口径 325 targets ok/0 FAILED ✓；multiconsumer 族重定位缺口关闭——账本同步）
