# Tasks

> 纪律：门控实验先行（只加 ifeq/ifne 分支臂）；行号按锚点名重验（`proves_boolean_local_store` 的 `boolean_position` match）。

- [ ] 1.1 插桩复核拒绝链（OP2.condAssignOld 当前诊断逐字）；门控实验：白名单加分支臂，OP2 翻转、跨 region 负例（合成：catch 内写 try 外读）不翻、既有三消费位锚逐字节。转录存证据。
- [ ] 1.2 冻结锚与负例双腿：OP2（巡查冻结件）+ if 语句位 / while 条件位 / 三元读位三新锚；负例=跨词法 region 形、短路链中段读形（`x && b` 的 b 在链内——如实记录是否被短路区域判据先拒）。
- [ ] 2.1 白名单增分支臂（消费者 `ifeq`/`ifne` 读该加载布尔值）；其余判据逐字不动。
- [ ] 2.2 对照测试：四锚恢复（重编+`-Xverify:all` 行为一致，OP2 main 全行）；`recover_short_circuit_local_values`（7/7）+ scv 系列套件零回退；负例拒绝逐字。
- [ ] 3.1 全门禁（含 oracle ignored 腿）+ corpus 指纹 + 分逻辑提交（不 push）。
- [ ] 3.2 root 独立复核：门控、白名单最小性、锚/负例实测、账本（census 重跑 OP2 单点关闭）。（留 root）
