# DT-08 主线独立验收

主线 `6798eec5` 纳入实现后，root 以独立构建的 `jarde-cli`（SHA-256 `0f3a8826aa7ada5034f1faad6d7fcacad0ee87f7fd1ac3f1c59d1389c962c4b8`）运行 [replay-implementation.py](replay-implementation.py)，输出暂存于 `/tmp/jarde-dt08-root-accept-6798/`。摘要除 CLI 哈希外与仓内 [代理实现重放摘要](implementation-replay/summary.json)逐项一致，JADX revision、输入/源码哈希未变。原 class、JADX 和 Jarde 的完整 Java 8 源码都能重编，并在 `java -Xverify:all` 下逐行输出 `2.5`、`-0.0`。Jarde 根类只输出内联匿名 `Runnable`，不引用源级 `Capture$1`；物理 child 独立查询仍成功。

root 另外运行 `cargo test -p jarde --test class_source`，77/77 通过；`cargo check --workspace`、`cargo fmt --all --check`、`openspec validate recover-proved-anonymous-local-capture --strict` 均通过。代理的负例覆盖错误槽/字段身份、额外写入、异常范围、method-handle、跨类使用、预算和取消；root 审阅确认根参数值先由 SSA 入口来源和唯一构造实参闭合，child 的 synthetic-final `D` 字段按准确构造器、写入和读取 BCI 证明，最后经现有匿名家族路径原子输出。没有引入新的 JVM 层或 Java AST 机制。

此验收只给 DT-08 的直接返回、单个 `double` 参数捕获首片结项。多字段、外层实例与局部参数混合捕获、复杂局部赋值和闭包链仍按独立形态处理。
