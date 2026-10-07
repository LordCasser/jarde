# Tasks

> 纪律：门控实验先行（三形各自的 local-crossing 拒绝点定位；A 相机制单独是否翻转、是否需 crossing 门成对豁免——两问分开回答）；行号按锚点名重验。

- [x] 1.1 插桩定位三形的 local-crossing 拒绝链（crossing 判据在哪层先拒：region `test_is_pure`/loop 判定/名字层）；门控实验：只开快照消费者条件位（不动 crossing 门）→ 观察翻转；若不翻，再做 crossing 门的成对豁免门控（同 A 相 `unobservable_store_dance_part` 先例）。转录存证据。
      → `results/01-gating.md`：三形分别在 `latch_test_suffix_is_effect_free`+`test_is_pure`（scan）、`header_test_chain`→`test_is_pure`（find）、`build_two_exit_return` 的 test-block 依赖判据（cond，且 A 相快照证明本已认领该消费位）；实验 A 单独翻转 cond（外层 test block 的 lead 是既有缺口，与 postfix 无关），实验 B（region 成对豁免 + 条件位数组读）翻转 scan/find；两问分开作答，依赖 walk 的 updated-value 臂写了又删（不需要）。
- [x] 1.2 冻结锚与负例双腿（巡查 postinc-condition fixture 复用 + 补 if 短路位）：负例=多变量复合条件、短路链中段形、`i = i++` 陷阱（A 相已有）——各保持拒绝。
      → `tests/fixtures/recover-postfix-condition-positions/`（`CP7` 巡逻源逐字节复用 + `CN` 两负例与链控制 + `CC` 计数位，双腿 + sha256 + README）；陷阱从 A 相 `NG.class` 读取（同 `recover-dup-store-conditional` 读 CF-06 的先例）。
- [x] 2.1 实现条件位消费者泛化（+必要的 crossing 成对豁免）；拒绝文本与 A 相判据逐字不动。
      → `region.rs`：`test_expression_instruction`（含 `snapshot_condition_part` 成对豁免与条件位数组读单读者判据）、`latch_test_suffix_is_effect_free` 复用同一判据、`FallbackReason::ChainPositionBound` 链边界；`build.rs`：`build_two_exit_return` 外层 test block 的 lead + `test_effects`、跨块旧值读的 `consumer_block_precedes` 有界形。A 相拒绝文本（`i = i++`、多消费方）逐字不变（测试与扫掠双重证据）。
- [x] 2.2 对照测试：三形恢复（重编+`-Xverify:all` 行为一致，do-while 扫描迭代数精确）；A 相套件（`recover_postfix_old_value_snapshot` 7+1）零回退；负例拒绝逐字。
      → `tests/recover_postfix_condition_positions.rs`（4+1 ignored；`CC` 的三形各答自增计数，do-while 扫描的迭代数由输出精确比对）；A 相套件 4+1 绿（`NG.condShape` 这一 A 相 out-of-scope 门按预期移动，断言更新而非删除）；`recover_dup_store_conditional` 3+1 绿（`ioLoop` 兄弟形未动）。
- [x] 3.1 全门禁（含 oracle ignored 腿）+ corpus 指纹 + 分逻辑提交（不 push）。
      → `results/04-gates.md`（fmt/clippy/workspace 329 ok 0 FAILED/openspec 307/oracle 3-3）；`results/03-corpus-*.{sh,out,md}`（10 moved，逐个分类，两个 outgrowth 行为复核）；读者夹具普查与 `corpus-fingerprint.json` 按先例重测重渲染；提交见 `results/04-gates.md` 末节。
- [ ] 3.2 root 独立复核：门控两问、判据零放宽、三形/A 相/负例实测、账本（postfix 域 B 相关闭）。（留 root）
