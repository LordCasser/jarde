# Root 独立验收（2026-10-07，合并）

## 判据逐项

1. **前提漂移四项处置复核**：锚 SHA 逐字相同 + HEAD 基线同句拒绝 ✓；姊妹片已覆盖的部分**不重复实现**（`BN.pickSeq` 基线 0 引注→逐字不动）——剩余 delta 恰六条 `→ java.lang.Number` 边 ✓；闭集以 rt.jar 全部 20400 类名反射核对（直接子类恰六、间接 0、无跨表目标→walk 死代码不写）✓；巡查链接勘误（真实巡查在 10-03）✓。
2. **diff 审查**：`NUMBER_FAMILY` 六行 + 文档 + `.any()` 一行（34/3 核心改动）；既有六表行、release 门、阵列位谓词逐字未动；Boolean 源级不可产生形钉在单元测试（正确归属——语言事实非表行）。
3. **root 实测**：定向 3+1 ignored 回放绿（双腿编译+`-Xverify:all`）；oracle ignored 3/3。
4. **门禁（权威口径）**：全量 exit 0、**334 targets ok、0 FAILED**；fmt OK；CI 逐字 clippy `Finished` 0；openspec **307/307**；实现片首轮 `ordinary_generic_projection` temp-dir flake 单测复跑 ×2 绿（既有家族）。
5. **corpus**：候选面 22 类→移动 2 渲染（C8 散装+jar 条目）、引注 2→0；全量面 1987 类→移动 1（巡查 C8）；BNX 表外形逐字保持；census `(857,3723,290,2273,8)→(863,3757,290,2309,8)`，指纹 +9 纯增。
6. **CI**：合并推送后 run 为准（监控在案）。

## 残余

- `java.math.*`/atomic 族与 `Number→Serializable` 保持拒绝（保守六行，spec 如文）；扩行需新 patrol 证据 + javap/反射转录。
