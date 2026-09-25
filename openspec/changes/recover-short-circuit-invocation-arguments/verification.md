# 实施验收（2026-09-25）

现有 `ShortCircuitValue` 的闭合图、精确正常前驱、异常边、测试依赖、1/0 producer 和唯一同槽 Phi 证明保持一份。新的私有消费者分支只接受真实 `invokestatic` `0xb8`、`CONSTANT_Methodref` 解出的静态目标、描述符恰为 `(Z)V`、消费指令只读该 Phi 且无写入。构建器把已证明的条件值暂时交给现有 `call_expr`/`arguments`，经原有布尔参数适配发射一次 `Call` 表达式语句；完整调用构成后才发布 Region 语句。后缀 BCI 24 `return` 仍按原块顺序写出。字段与直接返回分支沿原证明运行。

冻结 `MixedBooleanArgument.class` SHA-256 为 `5dbdfc2e66a44bab18c3ab39b824a0173a09cc38fa1bdf33d5ff9fdf04a2eeaa`；原源码由 `javac --release 8 -g:none` 重编逐字节一致。冻结 JADX 完整类重编退出 0，原/JADX 八行输出相同。直接返回任务验收后的 CLI 报告中，`call(Z)V` 仍为 `fallback/mixed` 且八行 `sinkCalls=0`；本次当前 CLI 则给该方法 `structured/java`、恰好一个 `sink(...)`、完整 BCI 0/1/4/7/10/13/16/17/20/21/24 来源。

- `cargo test --test p3_mixed_short_circuit_argument --test p3_mixed_short_circuit_field --test p3_mixed_short_circuit_return -- --include-ignored`：3/3、2/2、4/4 通过。调用正例恢复后的完整类经 `javac --release 8` 和 `java -Xverify:all`，八行 `result`、`bCalls`、`cCalls`、`sinkCalls` 与原 class 逐字相同；字段十六行与直接返回八行也保持一致。调用测试另证实低 analysis budget 和预取消均不发布部分调用。
- `cargo test -p jarde-java --lib --test p3_short_circuit_chain_controls`：168/168 与 verifier-valid 双消费者控制 1/1 通过。新增 `javac --release 8 -g:none` 多参数 `(ZI)V` 与实例 `invokevirtual (Z)V` 两个真实近例，均整图引用且全部已解码 BCI 可追；fixture 编译与描述见 `tests/fixtures/p3-conditional-values/mixed-short-circuit-argument-controls/README.md`。
- `cargo test --test p3_region_owner_overlap --test p3_region_fallback_origins --test p3_short_circuit_exception_chain --test p3_short_circuit_exception_edge --test p3_invocation_arguments --test p3_deferred_value_order`：额外入口、来源预算、两种异常边和相邻调用顺序控制全部通过；两个调用顺序套件的需 JDK 测试再以 `--include-ignored` 复跑，7/7 通过。
- `cargo fmt --all -- --check`、定向 `rustfmt --check`、`git diff --check`、`openspec validate recover-short-circuit-invocation-arguments --strict` 通过。`cargo clippy -p jarde-java --lib` 退出 0，仍报告当前共享树其他位置的 16 条既有告警。

保留拒绝边界：更多参数、实例接收者、第二 Phi 消费、异常边、额外入口、未知 Methodref、不能折入条件值的独立效果和预算停止。后四种沿未改的公共图/SSA 守卫及既有控制验证；本次未为畸形未知 Methodref 单独生成类文件。私有 Cargo target `/tmp/jarde-short-arg-target` 在验证后以 `cargo clean --target-dir` 清理；本 change 不归档。

根代理独立复验：当前共享树的 `jarde-java --lib` 168/168；调用实参/字段/直接返回及所有权/来源/异常边定向测试分别 3/3、2/2、4/4、1/1、2/2、1/1、1/1；`jarde-java` 双消费者控制 1/1。重新构建 CLI 后，从同一 SHA 的冻结 class 导出[当前完整类](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-argument/jarde-after-argument-MixedBooleanArgument.java)和[报告](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-argument/jarde-after-argument-report.json)，将源码以其公共类名保存并独立以 Java 8 编译、`java -Xverify:all` 执行，8/8 行逐字等于原 class/JADX，`sink(...)` 仅一次、无引用。格式、diff 检查及本 OpenSpec strict 均通过；[javac 日志](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-argument/jarde-after-argument-javac.log)记录成功复验。
