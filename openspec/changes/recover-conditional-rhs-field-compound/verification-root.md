# Root 独立验收（2026-10-07，合并）

## 判据逐项

1. **门控复核**：`results/01-gating-refusal-point.txt` —— 拒绝点=join 块消费写值 receiver 副本（`receiver_copy_at` 扫描只走 copy 自身块）；子证明通道已有（区域 trace 显示 `Proved(ConditionalValueProof…)`）。门控矩阵 7 输入：CHANGED=BI 冻结+双腿+RC 双腿，IDENTICAL=RCN 三负例逐字节——准入恰为"已证条件物化"。
2. **diff 审查**：`build.rs` +553——证书 `conditional_receiver_copy_at` 只在普通扫描拒绝后进入，`prove_conditional_value` **只读**（region 陈述交其重校验）；`bitwise_boolean_operand` 收窄到字段写取值处（实测支撑：宽规则会暴露 BW 的已登记 `mix` 可编译错面——归还 `recover-boolean-int-bitwise-operands` 域，测量过程归档）；`crosses_a_proved_cut` 与 concat 先例同读法。既有判据/拒绝文本/循环级判据逐字未动。
3. **root 亲测**：冻结 `bi.jar` 渲染 **0 引注**，整类剥离编译 exit 0，BI 自带 main 输出 **`false/false/false/false` 与原 class 逐字一致**（root 双侧运行）——**第 15 critical 锚的可编译错面关闭**。
4. **门禁（权威口径）**：定向 4+1 ignored 回放绿；oracle ignored 3/3；全量 exit 0、**327 targets ok、0 FAILED**；fmt 0；CI 逐字 clippy `Finished` 0；openspec **306/306**。
5. **corpus**：moved=6 全为本片（冻结锚+副本+双腿 fixture）；CH/SC/BF/CA2 零回退；census `(805,3405,288,2095,8)→(811,3445,290,2145,8)`，指纹 +50 纯增。
6. **CI**：合并推送后 run 为准（监控在案）。

## 残余

- 更长 RHS（`n &= m + (x>0?1:0)`）保持拒绝（写值须即 phi 消费者）；
- `dup_x1` 条件形构造上支持但无锚钉住；
- BW.andNot/mix 两行归还 `recover-boolean-int-bitwise-operands`（附实测）。
