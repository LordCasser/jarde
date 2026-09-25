# 验证记录（2026-09-25）

冻结正例 `MixedBooleanLocal.class` 的 SHA-256 是 `8901ea8d51475567257d20b089c193fa4bd098fe70dc930755004cb72fca231b`。`one(Z)Z` 的栈 Phi 由原有闭合短路图证明，唯一直接读者是 BCI 21 `istore_1`；该 Store 的 SSA local 在 BCI 22 和 26 各被读取一次，分别流向 `putstatic result:Z` 和方法描述符 `Z` 的 `ireturn`。类型决策前仅为这一可命名、单写、同一区域且完整 Boolean use 链的 local 加入布尔证据。发射仍使用现有 `Declare` 和 `Local` AST，BCI 21 为声明来源，测试和 1/0 producer 为派生来源。完整 Java 8 类重编、`java -Xverify:all` 原类与恢复类的八组 `return/result/bCalls/cCalls` 对照全部相同。

定向执行结果：

- `CARGO_TARGET_DIR=/tmp/jarde-local-target cargo test --test p3_mixed_short_circuit_local`：5/5；包含 8 路径重编执行、source map、预算和取消。
- `CARGO_TARGET_DIR=/tmp/jarde-local-target cargo test -p jarde-java --lib`：169/169，含真实 Phi/Store 单测。
- 字段、返回、调用实参、网关、Region owner 和异常边的六个 root 定向套件：16/16 非忽略测试；字段 Java 8 完整类 16 路径的忽略测试单独执行通过。
- `cargo test -p jarde-java --test p3_conditional_values --test p3_short_circuit_chain_controls`：3/3。
- `cargo fmt --all --check`、`git diff --check` 和 `openspec validate recover-short-circuit-local-values --strict`：通过。
- `cargo clippy -p jarde-java --lib --no-default-features`：退出码 0；现有其它代码位置仍报 16 条建议/复杂度警告，本变更没有新增 `dead_code` 警告。
- `cargo clippy --workspace --all-targets --no-default-features`：被无关并发接口改动阻断。`recover_for_class_source` 现需第三个 `bool` 参数，而 `crates/jarde-java/tests/class_initializer_candidates.rs` 七处和 `p3_patterns.rs:389` 一处仍仅传两个参数，均为 E0061；本 change 未修改这些调用。

四个 Java 8 verifier-valid 拒绝控制已永久纳入 `p3_mixed_short_circuit_local`。CLI 报告显示 `numeric` 和 `duplicated` 最早在短路候选的 SSA 值证明处拒绝，`rewritten` 最早在 Region ownership overlap 拒绝，`crossing` 最早在异常边/局部声明作用域证明处拒绝。它们均整体 quote 并保留测试所列 source map；这些结果**不**单独证明前置 local 类型门的每一分支。无名变量控制尚未构造，故任务 1.2 保持未勾选。跨 Region 读取和提升赋值也不在本次已证明范围内；前置证据要求所有读写具有同一 RegionPath。`crossing` 的现有 fallback 仅映射 canonical block starts，这属于既有来源粒度问题。

私有 Cargo target 在验证结束后以 `cargo clean --target-dir /tmp/jarde-local-target` 清理。

根代理独立重建 CLI 并保存[最终全证据](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-local/jarde-after-local-report.json)、[完整 Java 类](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-local/jarde-after-local-MixedBooleanLocal.java)、[编译记录](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-local/jarde-after-local-javac.txt)和 [JVM 八行](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-local/jarde-after-local-run.txt)。方法实测 `structured/java/contains_statements`，一次 Boolean 声明、两次局部名读取、14/14 BCI 来源；Java 8 编译和 `-Xverify:all` 成功，八行与原 class 完全一致。根代理另重跑库内 169/169，以及局部/网关/字段/返回/实参/所有权/两类异常边八个定向套件；除字段套件一个既有 ignored，非忽略测试全通过。格式、diff-check、OpenSpec strict 亦通过。根代理 target 在此轮全部复核完成后统一清理；任务 1.2 保持未完成。

对任务 1.2 的独立证明门审计没有新增测试。现有 `fixture_value_attempts` 只返回候选 Region、尝试结果、恢复树与最终报告；它丢弃 `CanonicalCfg`、`SsaTable`、`Operations`、`reuse::Plan`、`field::Plan`、`NameTable`、`RegionPaths` 和局部 `SlotUse` 汇总。要在 `build.rs` 私有单测直接调用 `short_circuit_local_booleans`，需重建完整 `analyze_method_ir`/Region/reuse/field/name 上下文，而不只是复用现成正例 helper。四个永久 verifier-valid controls 也未隔离该前置门：`numeric`/`duplicated` 在通用 SSA/Phi consumer 证明处先拒绝，`rewritten` 先在 Region ownership overlap 拒绝，`crossing` 先在异常边/声明作用域处拒绝。移除 NameTable 条目只能靠构造不符合真实 slot-plan 的局部上下文；额外 Store 或不同 RegionPath 若只篡改 `SlotUse` 会使它与真实 SSA 不一致；非 Boolean consumer 还需重写私有 `Operations` 或新增 verifier-valid class。为避免把合成事实伪称为字节码证明，本轮未加这类测试或夹具，1.2 继续未勾。当前仍缺少分别到达 local Boolean/name/multiple-write/scope guard 的真实输入证明；第二 Phi use 的既有端到端拒绝也没有被等同为 local 前置门覆盖。
