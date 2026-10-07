# Root 独立验收（2026-10-07，合并）

## 判据逐项

1. **两问门控复核**：`results/01-gating.md` —— Q1 定位：`scan/find` 拒在 **region 测试纯度门**（`latch_test_suffix_is_effect_free`/`header_test_chain`→`test_is_pure`），`cond` 拒在 `build_two_exit_return` 的外测试块 lead（该缺口**无后缀也复现**——父提交对照实证）；**快照消费者从不是拒绝点**（`PROBE-CLAIM` 显示 A 相证明已认领）。实验 A：`cond` 由 lead 修复翻转、`scan/find` 不翻；实验 B（`test_expression_instruction` 单处陈述测试块可含物 + `iload;iinc` 对 + 条件位数组读 under length 单读者）翻转 `scan/find`。门控质量高，两问分离回答。
2. **diff 审查**：`region.rs` +428 重构为**单一** `test_expression_instruction`（原来分散的测试块判据收拢——结构改善非平行机制）；`ChainPositionBound`（`jre_region_chain_position_bound`）围栏强制 spec 声明边界（一端一位），延后形具名拒绝；two-exit-return 家族**不**围栏（参数目标形父提交已呈现，围栏会回退既有恢复——正确取舍）。
3. **root 实测**：CP7 三形 0 引注/0 crossing，`do { … } while (arg0[local1++] != 0 && local1 < arg0.length);` 原源码形态；定向 4+1 ignored 回放绿（**do-while 迭代计数精确** `3/2/1/2` 逐项比对）；A 相 3+1、dup-store 4+1 全绿；oracle ignored 3/3。
4. **守卫更新**：`NG.condShape` 断言按其 README 预告翻态（更新非删除）；`p3_loop_test_values` 两条陈旧 `while (arg1-- > 0)` 拒绝更新为呈现形（保持 no-movement 性质）。
5. **门禁（权威口径）**：全量 exit 0、**329 targets ok、0 FAILED**；fmt 0；CI 逐字 clippy `Finished` 0；openspec **307/307**；census `(822,3564,290,2161,8)→(828,3596,290,2211,8)`，指纹纯新增。
6. **corpus**：moved=10 全分类（4 本片 + cp7 + NG 对 + p3-loop-test-values + 2 外沿 jar）；**数组读取准入外沿**（`SW.sum2d`/`NL.findMid`——**非后缀形**）双腿行为回放一致——**追认为机制内外沿**（同判据外推族），未钉面登记为边界（后续巡查覆盖）。
7. **CI**：合并推送后 run 为准（监控在案）。

## 残余

- 多变量复合/链中段形保持拒绝（ChainPositionBound 具名）——后续按需扩；
- 数组读外沿未钉测试（两 corpus 例已行为验证）；two-exit-return 家族外沿同；
- `jvm_bytecode_oracle` 的 JDK-25 boundary 属 CI 专属（本机 JDK 23 先拒 preview API）。
