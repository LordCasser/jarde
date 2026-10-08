# Tasks

> 纪律：门控实验先行（fused-void-continuation 准入单独翻 nestedLocksBranching、五族锚不动）；行号按锚点名重验（`nestedLocksBranching`/`BCI 55` 拒绝与 loop-test-copy 的 fused continuation 读法）。

- [ ] 1.1 复核 `nestedLocksBranching` 在 HEAD 的现状（nested-lock 片 fixture 在库）；定位 void 完成形的 transfer 判据处与 loop-test-copy 片的 tail-span 读法（复用先例还是姊妹读法）；门控实验转录存证据。
- [ ] 1.2 冻结锚与负例双腿：branching 形（含 if/else 与单 if 两变体）；负例=尾 span 含 control flow 形（goto 非直线）、多重分支形——各保持拒绝。
- [ ] 2.1 实现 void 完成形的融合尾 continuation；既有证书判据逐字不动。
- [ ] 2.2 对照测试：branching 恢复（重编+`-Xverify:all` 双驱动一致）；五族 guard 套件零回退；负例拒绝逐字。
- [ ] 3.1 全门禁（含 oracle ignored 腿）+ corpus 指纹 + 分逻辑提交（不 push）。
- [ ] 3.2 root 独立复核：门控、尾 span 判据、锚/负例实测、账本（guard 族第 3 边界关闭）。（留 root）
