# 本片 workspace 分组门禁 runner v1

新增 `run-partitioned-workspace-gates-v1.py`，结构沿用前片分组方式，但只执行本仓新鲜 gate，不读取或引用任何前片/旧 runner 的测试结果。runner 使用本片 `run-root-gate-v1.py`，标签与原始 command 目录均写在本片 `results/partitioned-workspace-gates-v1/`。

执行时先调用 `cargo metadata --no-deps --format-version 1 --locked`，保存 metadata 的原始 stdout/stderr 和 SHA，动态枚举 workspace member 的全部 target；按 lib/bin/example/bench 分组，并把 integration test 按最多十个 target name 分组。每组顺序运行两个固定 `PROPTEST_RNG_SEED`，Cargo 命令统一使用 `test --workspace --all-features --locked`。无测试用例的 example 仍通过 `--examples` 构建/运行空 harness，不从 target census 中剔除。

manifest 记录每个 target 的 package/name/kind/path/test 标记和源文件 SHA；另冻结 `init.rs`、`report.rs`、`build.rs`、`Cargo.lock` 的路径/字节数/SHA。每条命令前后以及全部结束时复核这两组文件身份；发生变化立即保留已有输出并停止。每条 gate 保存实际 argv、seed、exit、root wrapper 退出、20 GiB guard 状态、完整 result 路径/SHA、wrapper 原始 stdout/stderr 路径/字节数/SHA；root runner 内 Cargo 原始 stdout/stderr 也纳入输出闭集。

成功完成一组的两 seed 后，只按 target 名在本仓 `target/debug/deps` 与 `target/debug/examples` 顶层精确删除匹配的普通可执行文件，记录每个删除文件的 SHA/字节数与磁盘空间；不递归删除，也不清 Cargo cache/共享依赖。最终仓库级 `cargo clean` 由 root 在所有分组成功后单独执行。分组两 seed 不是两次单命令全 workspace 运行，不能把它表述成后者。

本文件和 runner 仅完成静态准备，未执行 Python runner、Cargo、Git 或任何测试；完整目标数量、分组数、raw 结果、结束状态均待 root 实际运行产生 manifest 后确认。