# 2.4 来源、预算与回归核对

本轮只增加了双构造器边界测试，没有改动生产行为或公开报告结构。新增测试覆盖 `-g` 与 `-g:none` 两种输入：默认证据和 `RecoveryEvidenceRequest::all()` 的完整类源码相同；JSON 字段表保持 `ZERO`、`ONE`、实例字段 `value`、`$VALUES` 的物理顺序与身份，方法表保持原七项索引和两条构造器 descriptor。两条构造器的 JSON outcome 仍是 `recovered`，各自 report 为 `produced/structured`；每个物理方法在 default/all 的 item、outcome kind/quality 和正文相同，source map 则按证据请求是否选择 materialization。

完整证据在终端构造器原报告中保留 Code 来源：helper 调用的 BCI 7 与字段写入的 BCI 12 都指向同一 `<init>(Ljava/lang/String;II)V` 物理方法，且 `text_of_bci` 能分别回到 helper 调用和 `this.value` 写入。helper `ConstructorEffects.record(I)V` 属于外部类，因此不是 `DelegatingEnum` 的物理方法 item；它以终端构造器 report 的调用正文和 BCI 7 来源保留。默认 evidence 未请求 source map，故其 segment 表为空；这不影响默认/all 的完整类源码正文相同。方法单体恢复通过 `recover_method_with_evidence` 单独请求原 `value()I`，其 report 正文和 source map 与 class-source 同一物理方法的原 report 完全相等，没有返回整组 enum 投影。

当前 CLI 也从冻结正例源码独立重编 `-g` 和 `-g:none` 的 jar 后，分别请求默认/all 的 `class-source --format json` 及 `recover value()I --evidence all`。两种 debug 模式的默认/all 类源码 SHA 都是 `c748f28b586e93a7763f83314a7ba9db7e21cf294d4846283de1fa1a4f68ded5`，JSON 均保留 4 个 field 与 7 个 method item；method-only 正文和来源 map 等于 all class-source 中该 `value()I` method report，终端 ctor map 可见 BCI 7 和 12。

预算边界也在双构造器 class-source 路径上核对：`ir_items=0`、`elapsed_millis=0` 和预先取消分别产生有界停止/取消结果；任何已返回的 class-source report 都没有 `Proved` 证书、`ZERO, ONE(1)` 或无参 `this(0)`，因此不会发布半个委托投影。已有 `delegating_projection_output_budget_stop_keeps_both_physical_constructors` 用完整投影的 `output_bytes - 1` 覆盖最后输出收费失败，断言报告退回原物理字段、常量及两条物理构造器。2.1/2.2 的委托边和终端正文候选还各自有预算/取消停止测试。

回归结果：`enum_constants::tests` **26/26**、`class_source::tests` **20/20**、`tests/class_source` **47/47** 通过。前两组包含 Stage/Measure 单构造器证明与 default/all 正文比较、普通类 `<clinit>` 投影、普通物理构造器的类源码呈现；Stage/Measure 此前冻结的正文 SHA 和双构造器运行时验证见 [2.3 Root 验收](verification-root-2.3.md)。当前私有 CLI SHA-256 为 `8299ace8b147b900d7471db3e988a7500836335cf552182a7b472938f47a5df5`，从冻结副本用它重放 1.3 的 `negative-controls/replay.py`，九个 verifier-valid 控制 **9/9** 通过，Jarde 对九例均拒绝整组投影。

命令与结果：

- `cargo test --target-dir /tmp/jarde-enum-delegate-projection-target -p jarde --lib enum_constants::tests -- --nocapture`：26/26 通过。
- `cargo test --target-dir /tmp/jarde-enum-delegate-projection-target -p jarde --lib class_source::tests -- --nocapture`：20/20 通过。
- `cargo test --target-dir /tmp/jarde-enum-delegate-projection-target -p jarde --test class_source -- --nocapture`：47/47 通过。
- `cargo test --target-dir /tmp/jarde-enum-delegate-projection-target -p jarde-cli --test class_source_cli -- --nocapture`：15/16 通过。唯一失败是现有普通类用例 `the_text_mode_writes_the_librarys_own_source` 对 `HistoricalControlFlow` 的 unreachable-handler 注释钉在 `// @bytecode 9`，实际正文包含 `// @bytecode 9 10 13 14`；本轮只改了枚举测试，没有改该普通方法或其 marker 行为，也未调整这条相邻断言。
- `cargo build --target-dir /tmp/jarde-enum-delegate-projection-target -p jarde-cli`：成功；上列 SHA 为最终测试源码后的重建二进制值。
- `cargo clippy --target-dir /tmp/jarde-enum-delegate-projection-target -p jarde --lib --no-deps`：退出 0，仍有 10 条先前存在的 warning，没有新增 warning。
- `cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-proved-enum-constructor-delegation --strict`：通过。

2.4 验收不替代 Root 的 3.1 整案复验；任务框保持未勾选，私有 Cargo target 留给 Root。
