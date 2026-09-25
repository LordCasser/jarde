# 验证记录

- 冻结 `MixedArrayValue.class` 的 15 个指令起点全部有 source map；`one` 为 Structured，只有一次 `array(arg1)[index(arg2)] =`。
- 原源码与恢复完整类均以 `javac --release 8 -g:none -Xlint:-options` 编译；原源码重编 class 与冻结 class 逐字节相同。两类均以 `java -Xverify:all` 执行，24 路输出逐行等于冻结 `original-run.txt`，含正常、null、越界时的数组/下标/b/c 调用数与异常时点。
- verifier-valid 控制覆盖 `[B` 同 `bastore`、`aconst_null` 无 `[Z` 类型、额外数组/下标 use、`dup_x2` 第二值 use、受保护异常边、额外外层正常分支；每项均没有发表部分 `IndexAssign`。将 JVM 合法的 `index` 名改为 Java 保留字 `class` 后，字节码仍通过验证与执行，局部数组候选保守引用。
- 私有 Cargo target 下，新增数组测试 4 项通过；既有 Boolean array store、字段/实例字段/返回/调用/局部/网关 9 个 test target 通过；字段消费者原本 ignored 的 Java 8 执行测试单独运行并通过。预算耗尽与预取消控制未发布部分数组赋值。
- `rustfmt --edition 2024 --check` 通过；`openspec validate recover-mixed-short-circuit-array-values --strict` 通过；`cargo clippy -p jarde-java --lib` 成功。`cargo clippy -p jarde-java --all-targets -- -D warnings` 被此前已有的 enumswitch/report/region/build/reuse lint 阻断，本次新增数组代码未报 lint。

当前准入限于同一首测试块中严格按 array→index→RHS 连续求值、唯一直接 use 且可呈现的两个目标操作数。跨块目标、未知组件类型、额外 use、边或 owner 均保守引用；不扩展通用数组类型推断。

独立架构债：通用调用表达式当前可把 JVM 合法但 Java 保留字的成员名直接写成调用文本。本变更仅在短路数组操作数准入处调用既有 `is_java_identifier` 拒绝；通用成员名呈现应另案处理。
