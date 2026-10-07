# Tasks

> 纪律：门控实验先行（行集扩展单独翻转 IO 两法、LK 三法不动；宽化行单独可独立验证）；行号按锚点名重验（lock-guard 刚合入，其证书结构为现状）；javap 转录协议。

- [ ] 1.1 插桩复核 IO 两法的 2 行形状（与 LK 单行形状的判别变量钉死：行数/资源句柄 vs 保存值）；lock-guard 证书的行集扩展点定位；门控实验。转录存证据。
- [ ] 1.2 javap 转录两宽化行（`InputStreamReader(InputStream)`/`BufferedReader(Reader)` + FIS/ISR 的 extends 头行）入 widening-row-sources 协议文件。
- [ ] 1.3 冻结锚与负例双腿：IO（巡查件）+ FileReader char 循环形；负例=多资源嵌套 try、close 带返回值形——各保持现状。
- [ ] 2.1 实现行集 resource-guard 扩展 + 两宽化表行（纯增）；lock-guard 单行判据与既有表行逐字不动。
- [ ] 2.2 对照测试：IO 两法恢复（重编+`-Xverify:all` 行为一致）；`recover_lock_guard_loop_finally`/`preserve_local_scope_plan`/宽化系列套件零回退；负例拒绝逐字。
- [ ] 3.1 全门禁（含 oracle ignored 腿）+ corpus 指纹 + 分逻辑提交（不 push）。
- [ ] 3.2 root 独立复核：门控、行集安全来源（SSA 同一）、锚/负例实测、账本（local-scope 锚 13 关闭、1.2/1.3 回填评估）。（留 root）
