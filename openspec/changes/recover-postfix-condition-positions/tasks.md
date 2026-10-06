# Tasks

> 纪律：门控实验先行（三形各自的 local-crossing 拒绝点定位；A 相机制单独是否翻转、是否需 crossing 门成对豁免——两问分开回答）；行号按锚点名重验。

- [ ] 1.1 插桩定位三形的 local-crossing 拒绝链（crossing 判据在哪层先拒：region `test_is_pure`/loop 判定/名字层）；门控实验：只开快照消费者条件位（不动 crossing 门）→ 观察翻转；若不翻，再做 crossing 门的成对豁免门控（同 A 相 `unobservable_store_dance_part` 先例）。转录存证据。
- [ ] 1.2 冻结锚与负例双腿（巡查 postinc-condition fixture 复用 + 补 if 短路位）：负例=多变量复合条件、短路链中段形、`i = i++` 陷阱（A 相已有）——各保持拒绝。
- [ ] 2.1 实现条件位消费者泛化（+必要的 crossing 成对豁免）；拒绝文本与 A 相判据逐字不动。
- [ ] 2.2 对照测试：三形恢复（重编+`-Xverify:all` 行为一致，do-while 扫描迭代数精确）；A 相套件（`recover_postfix_old_value_snapshot` 7+1）零回退；负例拒绝逐字。
- [ ] 3.1 全门禁（含 oracle ignored 腿）+ corpus 指纹 + 分逻辑提交（不 push）。
- [ ] 3.2 root 独立复核：门控两问、判据零放宽、三形/A 相/负例实测、账本（postfix 域 B 相关闭）。（留 root）
