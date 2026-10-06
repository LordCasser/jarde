# Tasks

> 纪律：门控实验先行（FieldCopies 复合 RHS 判据处，只开"已证条件物化"准入，BI 翻转/副作用负例不翻）；`prove_conditional_value` 只读复用（重实现即停手上报）；行号按锚点名重验。

- [x] 1.1 插桩定位复合 RHS 判据的拒绝点（`x > 0` 物化为何不被接受为 RHS）；门控实验记录；确认 `prove_conditional_value` 子证明通道（inline-concat 先例的证书形态）。
      → `results/01-gating-refusal-point.txt`：拒绝点在 `receiver_copy_at` 的同块消费者扫描——write 的 receiver 副本（`ValueId(33)`）被 join 块（bci 35）的 `putfield` 消费，而 copy 在 bci 14，扫描只走本块，`store` 恒 `None`；同文件记 `conditional region: branch=27 attempt=Proved(...)`，即 `prove_conditional_value` 子证明通道已存在（inline-concat 证书形态）。
      `results/02-gating-experiment.txt`：基线（`HEAD` 独立 worktree 二进制）↔ 本片逐输入 diff——冻结 `bi.jar`、BI 双腿、RC 双腿翻转，RCN 双腿**逐字节不变**（副作用臂不翻）。
- [x] 1.2 冻结锚与负例双腿：BI（冻结 bi.jar 为锚，BI.java 重编腿）；负例=物化臂含赋值副作用形、双比较嵌套形、物化跨异常表形——各保持拒绝逐字；CH/SC/BF 零回退基线。
      → `tests/fixtures/recover-conditional-rhs-field-compound/`：`bi.jar` 逐字节冻结（sha256 `99ada97475301a46261476f326486ab39c9e06033d033ec3bd244451ff1161f3`）+ 其 `BI.java`，本片 `RC`/`RCN` 双腿（javac 23 `--release 8` 与 Corretto 1.8.0_432）+ README。
      负例拒绝逐字（`results/02-gating-experiment.txt`）；CH/SC/BF 零回退：`results/05-corpus-delta.md` 的扫描 moved 列表不含它们，且 `recover_chained_field_assignment` 套件自测 `CF`/`NEG` 逐字节。
- [x] 2.1 实现条件物化 RHS 准入（只读子证明）；既有判据与拒绝文本逐字不动。
      → `results/03-implementation.md`：`conditional_receiver_copy_at`（证书陈述 region 交 `prove_conditional_value` **只读**重校验；自带臂=常量+transfer、常量 0/1、join Phi 即写值自身的消费者、read 值与副本值只在 store 处被读、区间=test 表达式 + join 表达式、无 handler）+ `boolean_position_values` 的 `bitwise_boolean_operand`（布尔位通道，**收窄到字段写取值处**——更宽规则实测使 `BW.andNot` 恢复并首次令该类文本可编译、暴露其既有 `mix` 引注为可编译错面，故退回给 `recover-boolean-int-bitwise-operands`）；`crosses_a_proved_cut` 读同一 cut（Phi 记录不计消费）。既有判据/拒绝文本逐字未动（证书只在普通证明拒绝后进入）。
- [x] 2.2 对照测试：BI.earlyRet 恢复且**整类剥离输出与原逐字一致**（`false/false/false/false`）；负例三形拒绝逐字；`recover_chained_field_assignment`/`recover_inline_conditional_concat_operands` 套件零回退。
      → `tests/recover_conditional_rhs_field_compound.rs`（4 常规 + 1 ignored 回放：冻结 jar 原类自带 main 输出比对 + 双腿编译运行）；`results/04-replay-frozen-bi.txt`；两先例套件绿（`results/06-gates.md`）。
- [x] 3.1 全门禁（含 oracle ignored 腿）+ corpus 指纹 + 分逻辑提交（不 push）。
      → `results/06-gates.md`；`results/05-corpus-delta.md`（moved=6，全为本片锚与 fixture；`BW` 已收回逐字节）；oracle ignored 腿 3/3；census `(805,3405,288,2095,8) → (811,3445,290,2145,8)`；fingerprint +10 文件、0 删除。
- [x] 3.2 root 独立复核：门控、子证明只读、BI 整类行为实测、账本（锚 15 关闭）。（root 2026-10-07 完成，见 [verification-root.md](verification-root.md)：门控矩阵复核 ✓、只读子证明 ✓、**root 亲测 BI 0 引注 + 整类输出逐字一致**（`false/false/false/false` 双侧）✓、门禁 327 targets ok/0 FAILED + oracle 3/3 ✓、宽规则收窄裁定（BW 面归还既有域）✓；**第 15 critical 锚关闭**——boolean-loop-earlyret 巡查登记行随本验收关闭）
