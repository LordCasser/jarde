# root-v5 私有验证材料

root-v5 组合 patch 由未应用的 root-v4 和本次对 `tests/p3_loop_boolean_exit.rs` 的 test-only delta 组成。delta 只在既有 `proved_header_test_chains_become_short_circuit_loop_conditions` 函数中增加两个内联断言：`andWhile(II)I` goto@20→2 与 `orWhile(II)I` goto@19→2。断言要求对应 BCI 有 derived anchor、派生来源的 method identity 等于该 method 的 `item.identity`，且关联段非空、从相应 while 头部开始并覆盖到语句闭括号。原有条件 BCI 3/7 来源断言保留。delta 不新增函数、fixture 或源文件。

断言依据只读旧 CLI 观察 `header-chain-old-cli-root-v1/observation-root-v1.json`：javap 与 default/all 共 3 条只读命令显示两处 latch 来源缺失；两个 profile 的正文/map 相同，所有已存在来源的物理 owner 正确。该记录明确标记 `new_cli_or_whole_class_runtime: false`，这里只作为断言规划依据，不作为完整类运行结果。

本次 delta 没有新增函数。完整 root-v4 中新增测试函数只有两个 `noPrefix` 用例：`no_prefix_while_latch_keeps_physical_source_and_one_owner` 与 `no_prefix_latch_budget_and_cancellation_publish_no_partial_source`。

`run-validation-build-root-v2.py` 保留 v1 原有 11 条命令，仅在 CLI build 前增加三条 `cargo test -p jarde --test p3_loop_boolean_exit --locked` 精确测试调用，并逐项校验三个测试名和 1/0/0 summary。test source pin 新增该集成测试；现有 include 闭包会继续 pin 它引用的 `LoopBool.java` 与 `LoopBool.class`。20 GiB free、1 GiB target 守卫和 CI 的 29 项 Clippy allow lint 沿用。

脚本输出目录为 `validation-build-root-v2`；冻结 CLI `/private/tmp/jarde-loop-latch-cli-v1` 与 `candidate-cli-v1.json` 名称保留。Java 子进程通过 `JAVA_HOME` 与 PATH 固定到 controls-v1 manifest 的 JDK 23，并校验 manifest SHA-256 和 java/javac/javap 三工具 SHA-256。

没有执行 Git、Cargo、JDK、JADX、CLI 或验证 runner。自动检查只解析生成 patch 的 hunk 头部与 +/-/context 数量。

SHA-256：

- root-v4 patch：`c42c69662cb8206f702d8d682cfd5567f1eaabc017faa2fd9bc0241d30b9d8df`
- test-only delta：`b816f6532506cabd9e140e0114cfe4a1c29f6962adee03c3a03a57309dc18f5a`
- 组合 root-v5 patch：`1c6afb3fc86f1f91f59e207e2b7814c5ae83fdf8a14ad1e0d5e3cee8a13c9387`
- validation runner v2：`7e97b66fa68f831f4c48690f55c7b6924e89cfb739744541bb336251e0d4fd75`
- controls-v1 JDK manifest：`ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec`

结构校验：test-only delta `2` 个 hunk，组合 patch `12` 个 hunk，所有 hunk 计数匹配。
