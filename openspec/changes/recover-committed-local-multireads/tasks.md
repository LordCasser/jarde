# Tasks

> 纪律：门控实验先行（定位计数门发出处，区分两类输入，只对已提交局部放宽并验证 NI 翻转/BI 不翻）；行号按锚点名重验。

- [ ] 1.1 插桩/读码定位 "has N consumers, so one local binding cannot prove its execution count" 的发出处；判明该处可见的"生产者是否已提交为局部声明"事实（或需穿入的通道）；门控实验记录。
- [ ] 1.2 冻结锚与负例双腿：NI（≥3 消费者主锚）、NJ（2 消费者零回退对照）、BI（循环携带形，拒绝逐字）、无声明栈携带多消费者负例（合成探针）。
- [ ] 2.1 实现输入分类（已提交局部 vs 栈携带 saved 值），计数门只保留给后者；判据块与拒绝文本逐字不动。
- [ ] 2.2 对照测试：NI 恢复（重编+`-Xverify:all` 输出 `3/10/5/true` 一致）；NJ/BI/栈携带负例/deferred-value 顺序锚（preserve-deferred-value-order 既有测试）零回退。
- [ ] 3.1 全门禁 + corpus 指纹 + 分逻辑提交（不 push）。
- [ ] 3.2 root 独立复核：门控实验、分类判据零放宽、锚/负例实测、账本更新（第 4 族可恢复性 NI 形关闭，BI 形登记后续）。（留 root）
