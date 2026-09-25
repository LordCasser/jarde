# 实施验收（2026-09-25）

冻结 `ChainExtraBoundary.class` SHA-256 为 `2630a7ea3e395b74121053dfa9370288dc510ea4d1fbfcf3bd9fbb9d04e1ba55`，`assign(ZZZZ)V` 有 16 个已解码 BCI。既存 evidence 的原源码重编与冻结 class 逐字节相同；原类、JADX 1.5.6 和变更前 Jarde 完整类都经 Java 8 重编和 32 路径执行。JADX 在 16/32 行错误，最小反例 mask 0 原类 `0:false:1` 而 JADX `0:true:0`；原 Jarde 整体引用基线 32/32 与原语义不同。详细原始命令及完整输出见 `openspec/evidence/java-syntax-2026-09-25/short-circuit-chain-extra-entry/analysis.md`。

现有私有 `Region::ShortCircuitValue` 增加有界 gateway 角色。只有单条已解码直接 `goto`、零 SSA 读写、前向同路径、唯一物理正常入口和出口、目标与解码一致、无异常/call 边的块可进入图；所有测试、gateway、两个 producer 和唯一消费者仍以精确物理前驱/后继与同槽 Phi 证明。图全闭合后才提交 owner。表达式沿真实测试 taken/fallthrough 和 1/0 producer 构造现有 `Conditional`，gateway 保留独立 BCI 来源，不创建公开 AST/IR/pass。字段消费者沿原 `field@1` 收窄路径，返回和实参消费者没有新增准入。

永久正例 `p3_short_circuit_transfer_gateway` 在原 class 与恢复完整类上分别以 `javac --release 8 -g:none` 编译、`java -Xverify:all` 执行，32 行 `result:calls` 与冻结原输出逐字相同。`assign` 为 `structured/java`、恰一处 `ChainExtraBoundary.result =`、无 fallback；BCI 0/1/4/5/8/11/12/15/16/19/22/25/26/29/30/33 均有 source map。低 analysis budget 和预取消只产生未完成结果。既有来源测试也确认 BCI 26 的块内第二指令与 BCI 8 在 essential/all evidence 下仍映射。

新增三个拒绝对照：经 `java -Xverify:all` 验证的 BCI 12 回跳 BCI 8 双入口类、BCI 8 同宽 `iinc` 独立效果类、BCI 8–11 `Throwable` 保护区类，均未发布结构化字段写入。多入口和异常表衍生 class、再生程序位于 `tests/fixtures/p3-conditional-values/short-circuit-transfer-gateway-controls/`，本机按程序重生后与 fixture 字节一致；效果类在测试内从冻结字节作确定性改写。异常表对照不改变原类 32 路径运行值，但因额外控制边保持保守引用。已有三测试双消费者与两种异常链对照继续拒绝。

验证命令与结果：

- `cargo test --test p3_short_circuit_transfer_gateway --test p3_region_owner_overlap --test p3_region_fallback_origins --test p3_mixed_short_circuit_field --test p3_mixed_short_circuit_return --test p3_mixed_short_circuit_argument --test p3_short_circuit_exception_chain --test p3_short_circuit_exception_edge -- --include-ignored`：20/20，通过字段 16 路径、返回 8 路径、实参 8 路径的完整类 JVM 比对。
- `cargo test -p jarde-java --test p3_conditional_values --test p3_short_circuit_chain_controls`：3/3，覆盖既有两/三测试表达式与双消费者拒绝。`cargo test -p jarde-java --lib`：168/168。
- `cargo fmt --all --check`、`git diff --check`、`openspec validate recover-short-circuit-transfer-gateways --strict` 通过。
- `cargo clippy -p jarde-java --lib --no-default-features` 通过，但共享树其他位置仍有 16 条既有告警；`--all-targets` 因无关测试 `class_initializer_candidates` / `p3_patterns` 对并发修改后的 `recover_for_class_source` 少传第三个 `bool` 参数而报 E0061，未在本 change 扩修。新增代码引入的 Clippy 告警已消除。

所有 Cargo 命令使用私有 `CARGO_TARGET_DIR=/tmp/jarde-gateway-target`，验收后以 `cargo clean --target-dir /tmp/jarde-gateway-target` 清理。本 change 不归档；没有把字段 presented 报告等相邻债务并入。

根代理从最新源码独立重建 CLI 并导出[完整类](../../evidence/java-syntax-2026-09-25/short-circuit-chain-extra-entry/jarde-after-gateway-root-ChainExtraBoundary.java)与[全证据](../../evidence/java-syntax-2026-09-25/short-circuit-chain-extra-entry/jarde-after-gateway-root-report.json)。`assign` 实测 `structured/java/contains_statements`，恰一处字段写入；Java 8 完整类重编成功，同一 Runner 的 [32 行](../../evidence/java-syntax-2026-09-25/short-circuit-chain-extra-entry/jarde-after-gateway-root-run.txt)与原类 32/32 一致。根代理另重跑库内 168/168、网关 6/6、相邻字段 1/1（另 1 项已有 ignored）、返回 4/4、实参 3/3、异常链 1/1、所有权 1/1，格式、diff-check、strict 均通过。根代理使用的 `/tmp/jarde-sept25-final-target` 留给紧接的局部值实施复核，最终统一清理。
