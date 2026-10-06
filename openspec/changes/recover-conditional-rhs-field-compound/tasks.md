# Tasks

> 纪律：门控实验先行（FieldCopies 复合 RHS 判据处，只开"已证条件物化"准入，BI 翻转/副作用负例不翻）；`prove_conditional_value` 只读复用（重实现即停手上报）；行号按锚点名重验。

- [ ] 1.1 插桩定位复合 RHS 判据的拒绝点（`x > 0` 物化为何不被接受为 RHS）；门控实验记录；确认 `prove_conditional_value` 子证明通道（inline-concat 先例的证书形态）。
- [ ] 1.2 冻结锚与负例双腿：BI（冻结 bi.jar 为锚，BI.java 重编腿）；负例=物化臂含赋值副作用形、双比较嵌套形、物化跨异常表形——各保持拒绝逐字；CH/SC/BF 零回退基线。
- [ ] 2.1 实现条件物化 RHS 准入（只读子证明）；既有判据与拒绝文本逐字不动。
- [ ] 2.2 对照测试：BI.earlyRet 恢复且**整类剥离输出与原逐字一致**（`false/false/false/false`）；负例三形拒绝逐字；`recover_chained_field_assignment`/`recover_inline_conditional_concat_operands` 套件零回退。
- [ ] 3.1 全门禁（含 oracle ignored 腿）+ corpus 指纹 + 分逻辑提交（不 push）。
- [ ] 3.2 root 独立复核：门控、子证明只读、BI 整类行为实测、账本（锚 15 关闭）。（留 root）
