# CF-16 实现验收

固定原 class SHA-256 为 `dcf5de9a4037ddd2169f103e426be38fac60dba8fb2d5cb38dc6ffe3c648018a`。`replay.sh` 使用 fresh Jarde CLI，对固定物理 class、原 Java 8 源码转写、固定 JADX 源码和 Jarde 完整类分别执行 `javac --release 8` 与 `java -Xverify:all`。四份类在成功关闭、抛出 `IOException` 和抛出 `IllegalStateException` 时均只调用一次 `close()`；前两种路径正常返回，运行时异常路径传播探针预先持有的同一对象。Jarde 完整类重编成功，目标方法没有 `@bytecode`、`finally` 或 recovery fallback；其物理 BCI `0,1,4,7,8,11,12,13,14` 均有来源映射。

三个近邻由 `make-neighbors.py` 从固定 class 作定点变异生成，均经 `java -Xverify:all` 执行并保持 Jarde 的可追溯拒绝：

| 近邻 | SHA-256 | 验证的拒绝边界 |
| --- | --- | --- |
| `changed-throwable.class` | `aa52a381c7fce255c3043dabce882c055a39e50ea616dd496b0f8e8fa590049b` | handler 抛出新异常，运行时路径的对象身份不再相同 |
| `rows-swapped.class` | `b0a5daef736ba2ff388cbc3267353a0609095703d7b2b42cf50c6268baa3f989` | catch-all 优先，`IOException` 不再被具名 catch 吞掉 |
| `handler-normal-exit.class` | `26130f374f5a3808240e293bae28cd81ba5de8ab6dd2e09e1582806f7316102e` | handler 增加正常出口，运行时异常被吞掉 |

定向 Rust 测试覆盖固定正例、三个近邻、正文构建失败的整段回退及预算/取消的空产物。`cargo test -p jarde-java --tests --locked`、`cargo check --workspace --locked`、`cargo fmt --all -- --check`、`openspec validate recover-transparent-empty-finally --strict` 与 `git diff --check` 是本分支的验收门槛。重放的完整输出写入可丢弃目录；运行 `replay.sh CLI OUTPUT_DIR` 可以重新取得 class、`javap`、源码、行为和报告。
