# Tasks

> 纪律：门控实验先行（行集扩展单独翻转 IO 两法、LK 三法不动；宽化行单独可独立验证）；行号按锚点名重验（lock-guard 刚合入，其证书结构为现状）；javap 转录协议。
>
> root 2026-10-07 裁定（1.1 实测后）：本片 = 两命名组件 + 前置 `new@1` 深度 2→3（三分支各自独立提交与门控）；`readAll` 的可观察 copy-and-store 形与 guard 体内循环局部声明位**登记为独立后续片**（不纳入，不触碰 copy 族纯度判据）；整类验收以 `countLines` 方法级 + 驱动行为对照为准。

- [x] 1.1 插桩复核 IO 两法的 2 行形状（与 LK 单行形状的判别变量钉死：行数/资源句柄 vs 保存值）；lock-guard 证书的行集扩展点定位；门控实验。转录存证据。→ [results/01-anchors-and-discriminator.md](results/01-anchors-and-discriminator.md)、[results/02-gating.md](results/02-gating.md)、[results/03-component-gating.md](results/03-component-gating.md)
- [x] 1.2 javap 转录两宽化行（`InputStreamReader(InputStream)`/`BufferedReader(Reader)` + FIS/ISR 的 extends 头行）入 widening-row-sources 协议文件。→ [results/04-widening-rows.md](results/04-widening-rows.md) + [widening-row-sources](../../evidence/java-syntax-2026-10-05/widening-row-sources/README.md) + `results/selfcheck-javap-rows.sh`
- [x] 1.3 冻结锚与负例双腿：IO（巡查件）+ caller-owned 流同形（`IOMidRead`）+ FileReader char 循环形（登记边界）；负例=多资源嵌套 try、close 带返回值形——各保持现状。→ [tests/fixtures/recover-io-resource-finally/](../../../tests/fixtures/recover-io-resource-finally/README.md) + [tests/recover_io_resource_finally.rs](../../../tests/recover_io_resource_finally.rs)
- [x] 2.1 实现行集 resource-guard 扩展 + 两宽化表行（纯增）+ 深度 2→3；lock-guard 单行判据与既有表行逐字不动。→ 提交 `feat(java): present the resource guard across a finally`、`feat(java): widen the wrapped-stream chain's two java.io argument positions`、`feat(java): present a three-layer construction run`
- [x] 2.2 对照测试：`countLines` 恢复（重编+`-Xverify:all` 行为一致）；`recover_lock_guard_loop_finally`/`preserve_local_scope_plan`/宽化系列套件零回退；负例拒绝逐字。→ [results/02-gating.md](results/02-gating.md)（显式更新的三处对照）、[results/06-gates.md](results/06-gates.md)
- [x] 3.1 全门禁（含 oracle ignored 腿）+ corpus 指纹 + 分逻辑提交（不 push）。→ [results/05-corpus-and-oracle.md](results/05-corpus-and-oracle.md)、[results/06-gates.md](results/06-gates.md)
- [ ] 3.2 root 独立复核：门控、行集安全来源（SSA 同一）、锚/负例实测、账本（local-scope 锚 13 关闭、1.2/1.3 回填评估）。（留 root）
