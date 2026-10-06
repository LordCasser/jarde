# Tasks

> 纪律：门控实验先行（定位计数门发出处，区分两类输入，只对已提交局部放宽并验证 NI 翻转/BI 不翻）；行号按锚点名重验。

- [x] 1.1 插桩/读码定位 "has N consumers, so one local binding cannot prove its execution count" 的发出处；判明该处可见的"生产者是否已提交为局部声明"事实（或需穿入的通道）；门控实验记录。
  - **已定位**：`crates/jarde-java/src/build.rs` `Builder::prepare_deferred_bindings`；门的前置条件要求生产者指令写 `Slot::Stack(_)`，故"已提交局部声明"类**不达门**（无需穿入通道）。
  - **实验结论（STOP 依据）**：NI 的 BCI 8/BCI 71 两处拒绝是**栈携带单消费者值**（`getstatic System.out`、`append` 返回值），"3 consumers" = 1 次指令读 + 2 条被替换平凡 phi 的操作数记录；已提交局部的表达式内多读**今天已按源码形态恢复**（`NMA`/`NI2`：`local1` ×4）；NI 被拒的真实机制是 `concat@1` 的 `jre_concat_split`（`+` 链内分支）→ deferred-binding 回落。唯一能归零 NI 计数诊断的通道（排除 phi 操作数记录）**不翻 NI**且**改变 BI 拒绝文本**。
  - 证据：`evidence/gating-experiment.md`、`evidence/dumps/`、`evidence/renders/`、`evidence/probes/`。
- [ ] 1.2 冻结锚与负例双腿：NI（≥3 消费者主锚）、NJ（2 消费者零回退对照）、BI（循环携带形，拒绝逐字）、无声明栈携带多消费者负例（合成探针）。
  - **基线已冻结**（`evidence/renders/`，含 sha256、两条编译器腿一致）：NI/NJ/BI/BS + 探针 NMA/NMB/NI2/CMP/SC。**双腿 jar fixture 未冻结**：那是实施产物，实施因 1.1 结论阻断。
- [ ] 2.1 实现输入分类（已提交局部 vs 栈携带 saved 值），计数门只保留给后者；判据块与拒绝文本逐字不动。**阻断**：目标分类不达门（空操作），无法翻转 NI；不近似实施。
- [ ] 2.2 对照测试：NI 恢复（重编+`-Xverify:all` 输出 `3/10/5/true` 一致）；NJ/BI/栈携带负例/deferred-value 顺序锚（preserve-deferred-value-order 既有测试）零回退。**阻断**：依赖 2.1。
- [ ] 3.1 全门禁 + corpus 指纹 + 分逻辑提交（不 push）。**未执行**：源码零改动（仅新增本变更目录证据与任务状态），全门禁与 HEAD 基线同值；待重新切片后执行。
- [ ] 3.2 root 独立复核：门控实验、分类判据零放宽、锚/负例实测、账本更新（第 4 族可恢复性 NI 形关闭，BI 形登记后续）。（留 root）
  - **待 root 决定**：本片前提被证伪（NI 的阻断点是 `concat@1` 跨块 `toString` + deferred-binding 回落，非计数门输入分类），需重新切片或改派（见 `evidence/gating-experiment.md` §6 的 P1/P2 选项）。
