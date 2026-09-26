# Region 续走与负例验证（2026-09-26）

冻结输入 `ConditionalIntermediateJoin.class` 的 SHA-256 为 `d135bb8ff9515fe79713cfec396245f96e6267d7a7b95bddabb02e7a62f36e92`。`choose(I)I` 的内层 join 是 BCI 18，外层 join 是 BCI 26。定向测试确认 BCI 18 的块留在外层真臂，每个可达 canonical 块 0、4、9、15、18、23、26 只有一个 Region owner；来源映射分别覆盖真实指令 18、19、20、26。旧双臂条件值、携带参数条件值与同 join 嵌套测试均通过。

负例 `tests/fixtures/p3-intermediate-join/IntermediateJoinNegative.java` 由 `javac --release 8 -g:none -Xlint:-options` 编译；源文件 SHA-256 `f12e16cac32ff1fab98a4ab3609f64d5105edef0747aecad1a335bce646ec5eb`，类 SHA-256 `168430179ff6c4485dc2c6399e44e8a9199a35f57e7ac5dbbba8549c01c9f6f9`。重新编译与已存类 `cmp` 相同。以 `java -Xverify:all` 执行四个方法输出 `13`、`6`、`-1`、`10`，无 verifier 错误。

- `independentEffect(I)I`：内 join 18 后是 `invokestatic side` 18、`iadd` 21、`goto` 22，独立效果阻止组合；完整候选引用并映射这些物理指令。
- `throwingBridge(I)I`：内 join 18 后是 `idiv` 19、`goto` 20；可能抛错的桥接运算被拒绝，物理来源保留。
- `externalEntry(I)I`：`javap -c -p` 显示 9→28 是内层条件以外的入口，与 19→28、25→28 同时进入 join 28；Region 以 `jre_region_arms_do_not_meet` 引用候选，包含 28–30，未单独认领或丢弃桥接块。
- `caughtBridge(I)I`：异常表确有 `[0,28)→29` 的 `ArithmeticException` handler；Region 保守引用分支候选，handler 保持可见，所有真实指令 BCI 都能在 all 证据来源中查到。

验证命令及结果：

```text
CARGO_TARGET_DIR=/tmp/jarde-intermediate-agent-target cargo test -q -p jarde-java --test p3_intermediate_join --test p3_conditional_values --test p3_carried_conditional_arguments
4 passed; 2 passed; 4 passed
CARGO_TARGET_DIR=/tmp/jarde-intermediate-agent-target cargo test -q -p jarde-java --lib nested
9 passed
javac --release 8 -g:none -Xlint:-options -d /tmp/jarde-intermediate-negative-recompile tests/fixtures/p3-intermediate-join/IntermediateJoinNegative.java
cmp tests/fixtures/p3-intermediate-join/IntermediateJoinNegative.class /tmp/jarde-intermediate-negative-recompile/IntermediateJoinNegative.class
相同
java -Xverify:all -cp /tmp:tests/fixtures/p3-intermediate-join VerifyIntermediateJoin
13 / 6 / -1 / 10
```

任务 1.2、2.2、3.1 仍需合并审查完整负例集：本记录没有把历史 `jsr` fixture 当作双 join 的 Call 边反例，也没有把无法由 verifier 有效 Java 源生成的额外 SSA use/owner 重叠伪装成字节码反例；这些应在直接 proof-unit 层验证后再勾选。
