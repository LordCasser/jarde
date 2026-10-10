# root-v3 私有验证 runner

`run-validation-build-root-v3.py` 是基于保留的 `run-validation-build-root-v2.py` 制作的新版本，没有覆盖 v2。它把三个精确测试调用合并为一次完整 target 调用：

```sh
cargo test -p jarde --test p3_loop_boolean_exit --locked -- --nocapture
```

命令矩阵共 12 项（原 11 项加一个 target）。runner 要求该项 summary 精确为 3 passed、0 failed、0 ignored，并核对以下三个测试均有成功行：`proved_header_test_chains_become_short_circuit_loop_conditions`、`an_independent_effect_between_header_tests_is_never_folded_into_the_condition`、`recovered_loop_exit_matches_the_java8_class_for_terminating_and_continuing_inputs`。test 文件与 include 闭包 pin 沿用 v2。20 GiB free、1 GiB target、CI lint、JDK23 manifest 和三工具 SHA 检查及 JAVA_HOME/PATH 控制沿用 v2；四个 Java 覆盖变量会从子进程环境移除，preflight 只记录被移除的键名。输出目录递增为 `validation-build-root-v3`；CLI 与 metadata 仍沿用 v1 名称。

未运行 Git、Cargo、JDK、JADX、CLI 或 validation runner。

SHA-256：

- v2 runner：`7e97b66fa68f831f4c48690f55c7b6924e89cfb739744541bb336251e0d4fd75`
- v3 runner：`ffd135a3df3751d730034216de6aaf441700cbd94082991ea4b7f229581801e3`
- JDK controls manifest：`ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec`
