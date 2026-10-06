# Tasks

> 纪律：门控实验先行（定位 `jre_concat_split` 的链所有权判据，只接"中间块=已证条件值物化"，NI 翻转/真跨块负例不翻）；行号按锚点名重验。判别探针已冻结（`../recover-committed-local-multireads/evidence/probes/`，NI/NI2/NMA/NMB/CMP，双腿）。

- [ ] 1.1 插桩定位 `jre_concat_split` 发出处的链所有权证明（toString 终端同块判据）；判明中间块的条件值物化能否以 `recover-conditional-values` 的既有两臂证明作子证明（只读复用，不重实现）；门控实验记录。
- [ ] 1.2 冻结负例双腿：真跨块副作用链（分支臂含赋值/调用副作用）、双比较嵌套形、跨异常表拼接——各保持 `jre_concat_split` 逐字；NMA/NI2 零回退基线。
- [ ] 2.1 实现链所有权判据的子证明接入；`jre_concat_split` 文本与其余判据逐字不动。
- [ ] 2.2 对照测试：NI/NMB/CMP 恢复（重编+`-Xverify:all` 行为一致，NI `3/10/5/true`）；负例三形拒绝逐字；`recover_scv_concat_consumers`/`recover_ref_eq_boolean_argument`/`recover_conditional_values` 既有测试零回退。
- [ ] 3.1 全门禁 + corpus 指纹 + 分逻辑提交（不 push）。
- [ ] 3.2 root 独立复核：门控实验、子证明只读复用、锚/负例实测、账本（multiconsumer 族重定位后的新缺口关闭）。（留 root）
