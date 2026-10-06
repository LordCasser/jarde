# Root 独立验收（2026-10-06，合并 `545d2626`）

## 判据逐项

1. **双落点门控复核**：`results/01-gating.md` —— int 形（`iadd;dup;istore;ifle`）走 copy 门（`prove_local_assignments` 曾证明后因 `slot < parameters` 丢弃）；引用形（`invoke;dup;astore;ifnull`）先撞 `region.rs::test_is_pure`（StatementFree）再撞同 copy 门——两落点各自门控成立。
2. **diff 审查**：`region.rs` 的 `unobservable_store_dance_part`——循环测试块的纯度豁免**成对要求**（copy+其喂的 store，且 store 值无可观察读者），可观察 store/被读 store 保持原拒绝原 BCI（循环守卫不变，`p3_loop_test_values` 4/4 钉住）；`build.rs` 三态呈现决策（Eliminated/Split/Expression）+ merge 跟随的 `local_store_is_observed`（无名 merge 视为读者——condAssign 的 join-phi 携带值正确判为未观察）；`duplicate_expression` 提取 + `#[inline(never)]`（实测内联溢出深栈）。
3. **root 实测**：定向 `recover_dup_store_conditional` 3+1 ignored 回放绿；**oracle ignored 腿 3/3 绿**（新纪律：corpus 移动片必跑——本片 6 类移动全为本片 fixture+巡查锚，oracle 无过期期望）；守卫 `p3_local_rewrite` 7/7、`p3_loop_test_values` 4/4。
4. **前提证伪处置裁定**：`condAssignOld` 实测 javac 发射裸 `iinc`（无 dup 舞蹈，双腿）——其拒绝属 `recover-short-circuit-local-values` 的 boolean 位置边界（`proves_boolean_local_store` 只认 putstatic-Z/boolean-ireturn/append-Z 位）。**追认为已登记边界**，不并入本片；该域后续扩位另裁。
5. **边界裁定**：局部活目标保持 copy 族的**原地赋值表达式**（非 Split）——**追认**：CF-06 契约（`cf06-inner-assignment/replay.py` 钉 `(local1 = …) > 5` 形）是既有已验收行为，本片只对参数目标/结构自有测试启用 Split，尊重既有钉住；循环测试内活目标与短路步参数目标保持拒绝（NEG 钉住）。
6. **门禁（权威口径）**：全量 exit 0、**324 targets ok、0 FAILED 行**；fmt 0；CI 逐字 clippy `Finished` 0；openspec **305/305**。corpus 差分 6 类全分类（4 本片 fixture + 2 巡查锚），指纹 +45 纯增，census `(783,3275,286,2023,8)→(789,3311,286,2061,8)`。
7. **CI**：合并推送后 run 为准（监控在案）。
