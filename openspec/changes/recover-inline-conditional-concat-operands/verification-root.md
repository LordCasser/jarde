# Root 独立验收（2026-10-06，合并）

## 判据逐项

1. **门控复核**：`evidence/01-gating-experiment.md` —— 发出点 `concat.rs::verify` split 检查（逐字未动）；**实测矩阵揭示原 spec 的"终端接受"必要非充分**（只跳 split → walk 改判 interleaved；强制 owned → 仍被 BCI 8 `getstatic` 的 phi 膨胀计数门挡），充分条件=证书同时许可 deferred-binding 侧（cut 透明性 + join 块内读者按真读计）——偏差如实上报并成为实现的一部分，追认。
2. **diff 审查**：`concat.rs` +535（`verify_conditional_cut_chain` 四条件证书 + `plan_conditional_cut_chains`，只对 split 归因拒绝的候选进入）；`build.rs` +114（`crosses_a_proved_cut`/`binding_consumers`/`shares_a_declaration_region`）；`jre_concat_split` 文本与其余判据逐字；子证明 `prove_conditional_value` **只读复用**（证书按 CFG 陈述 region 交其重校验，无重实现）。
3. **root 实测**：NI 旗舰（巡查冻结件重编）完整恢复——`"" + local1.make(3).v + … + (externalMake(local1,1).outerRef() == local1)` 原源码形态、0 split、0 计数门；定向 4+1 ignored 回放绿（双腿编译+`-Xverify:all`，NI `3/10/5/true`）。
4. **oracle 新纪律执行**：`p3_execution_comparison --ignored` **3/3 绿**（corpus 移动 10 类：6 fixture + NI + SZ（拒绝行消失、方法因自身它因仍引=收窄）+ 2 个 `getClass()==getClass()` 形（Main 恢复且 `sameClass=true`；AnonymousDoubleSite 为冻结字节补丁负例保持不可编译=安全形））。
5. **门禁（权威口径）**：exit 0、**325 targets ok、0 FAILED 行**；fmt 0；CI 逐字 clippy `Finished` 0；openspec **305/305**。指纹 +85 纯增；census `(789,3311,286,2061,8)→(801,3363,288,2095,8)` 逐项归因。
6. **回归套件**：scv-concat 5/5、ref-eq-boolean 2/2、conditional-values 7/7。
7. **CI**：合并推送后 run 为准（监控在案）。

## 残余

- 证书刻意窄（单块两臂/0-1 常量/等值/append(Z)/无异常边/≥2 appends）；臂内调用存储、嵌套双比较、跨异常表保持拒绝（ICN 四负例钉住）——扩证需新 patrol 证据；
- ICQ 的 identity-hash 掩码比较已在测试注明；SZ 的收窄面如实记录。
