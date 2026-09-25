# 实施验收（2026-09-25）

`ShortCircuitValue` 沿用同一个有界测试图、精确正常前驱、异常边检查、1/0 producer、同槽唯一 Phi 和唯一 use 证明。私有消费者证明仅增加直接 `ireturn`：真实 opcode `0xac`、方法描述符读出的返回类型 `Z`、消费块只有这一条指令。字段写入仍走原 `field@1` 身份与布尔收窄路径；返回使用既有 `Conditional`、`Return` 和 `adapt_return`。证明或预算停止前不发布语句。

冻结 `MixedLocalReturn.class` SHA-256 为 `688ad8ed65e7da2cd3663b497ac6caa94e53885acae7656f468d457be5bd3288`。原源码由 `javac --release 8 -g:none` 重编逐字节一致；冻结 JADX 类的 Java 8 重编记录退出 0，原/JADX 八行相同。变更前 CLI 编译产物从该 class 新导出 `value(Z)Z` 为 `fallback/mixed`，完整引用 BCI 0/1/4/7/10/13/16/17/20/21，方法缺 `return`；变更后当前 CLI 与永久测试确认 `structured/java`、单个返回语句和这十个 BCI 的 source map。

- `cargo test --test p3_mixed_short_circuit_return --test p3_mixed_short_circuit_field -- --include-ignored`：返回 4/4、字段 2/2 通过。两个完整类分别经 `javac --release 8` 和 `java -Xverify:all`；返回八行的值与 `bCalls/cCalls`、字段两方法十六行的值与调用次数均与原 class 一致。返回测试另证实低 analysis budget 与预取消不会发布部分结果。
- `cargo test -p jarde-java --lib --test p3_short_circuit_chain_controls`：166/166 库测试和双消费者控制 1/1 通过。`MixedIntReturn.value(Z)I` 是独立 `javac --release 8 -g:none` 生成的同形三测试/1/0/`ireturn` 类，返回类型证明拒绝，整个图和全部十个 BCI 均可追。
- `cargo test --test p3_region_owner_overlap --test p3_region_fallback_origins --test p3_short_circuit_exception_chain --test p3_short_circuit_exception_edge`：额外入口、来源预算、两种异常边 5/5 通过。相邻 loop 5/5、nested try 2/2、switch loop 3/3、旧 jsr isolation 1/1 通过。
- `cargo fmt --all -- --check`、定向 `rustfmt --check`、`git diff --check`、`openspec validate recover-short-circuit-return-values --strict` 通过。`cargo clippy -p jarde-java --lib` 退出 0，报告 16 条当前共享树其他位置的告警；`-D warnings` 因这些既有告警失败。

相邻全局测试现有无关阻断：`p3_method_ir` 的一条硬编码 canonical/SSA 计数与当前共享树不符（3/4 通过）；`p3_prefix_survival` 的两条旧 fallback 文本断言与当前结构化 `if`/`while` 输出不符（1/3 通过）。两处均未在本 change 扩修。私有 Cargo target `/tmp/jarde-short-return-target` 在验证后以 `cargo clean --target-dir` 清理；本 change 不归档。

根代理独立复验：当前共享树 `jarde-java --lib` 166/166；返回永久测试 4/4、字段 Java 8 完整类 2/2；额外入口、来源、异常边、循环、嵌套 try、旧 jsr 的定向控制均通过。另从当前 CLI 重新导出[完整类与报告](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-return/jarde-after-return-report.json)，经 Java 8 重编与 `java -Xverify:all`，八行逐字等于原 class/JADX。独立 `(Z)I` 同形类仍为 `fallback/mixed` 且十个已解码 BCI 全在引用来源中；未把报告中空的 `fallbacks` 数组误认为已结构恢复。
