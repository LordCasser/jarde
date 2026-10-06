# Root 独立验收（2026-10-06，合并）

## 判据逐项

1. **插桩定夺复核**：`results/01-instrumentation-*.txt` —— BCI 56 的两个主张者是同一 `Loop` 区域的两条臂（外层 then 臂先收 latch、内层臂再遇 → `FallbackReason::Loop` 引注 → 树双重持有）；判别变量 = **两继续路径是否共享 latch 块**（`loopElseIfRet` 共享独立 latch，`loopIfElseRet` 单前驱融合）。与恢复形的差分成立。
2. **diff 审查**：`region.rs` **纯增量 242+/0-**——`ladder_join` 读法（两臂严格前向路由在循环 scope 内交于唯一首块、无路由绕经 join 抵达 frame 边界、入口全经本分支、`LADDER_MAX_BRANCHES=1` 单层口径）+ 会合判据的 `ladder_elected` 豁免；`overlapping_owner`、拒绝文本、既有判据逐字未动。
3. **root 实测（合并态自建 CLI）**：`BS.bsearch` 完整恢复（while + 阶梯树形，0 canonical、0 `@bytecode`）；回放（ignored）passed（双腿编译 + `-Xverify:all`，`LR2` 整类 `2/-3` 与原一致）；三对照与巡查归档逐字节相同（实现片钉住 + root 抽验 BS 形）。
4. **定向/门禁**：`recover_loop_else_if_early_returns` 4 passed + 1 ignored ✓；全量 **3101 passed / 0 failed / 60 ignored**（321 targets）；fmt exit 0；CI 逐字 clippy `Finished` exit 0；`openspec validate --all --strict` **303/303**。corpus 差分（自检先行）：moved=11（锚+新 fixture 单元），overlap 拒绝句 36→23，其余零移动；reader `(739,3083,284,1833,8)→(749,3139,286,2005,8)`，fingerprint 纯新增 15。
5. **CI**：合并推送后 run 为准（监控在案）。

## 残余裁定（root，2026-10-06）

1. **同级 switch/`for` 头形一并恢复——追认**：`ladder_join` 只读阶梯自身两臂路由，同级语句不参与证明；proposal"switch 混合保持拒绝"的本意是 switch **混入阶梯臂**（`switchInArm` 已钉住拒绝）。属机制内正确外沿，非越界。
2. **`firstArmRet`（早退在第一臂）保持拒绝——接受为登记边界**：两臂不在 latch 汇合，join 几何不同；后续片按需另证。
3. **整类回放锚定 `LR2`——接受**：BS/CB/LB 的 `main` 被拒时 emitter 按设计写保留符号，整类剥离本不可能编译；方法级回放 + LR2 整类（同形全可呈现）是正确证据形态，改 emitter 超出本片。
4. **`tryLadder` 拒绝为 `Try` 区域内部同族 overlap**——如实记录，非本片判据。

## 账本

第 5 可恢复性族（canonical-overlap）的锚形状（`while + else-if 阶梯 + 早退`）关闭；族内残余（firstArmRet、try 内部、双层阶梯）已登记。census 第 5 条已由归因修正段指向本片。
