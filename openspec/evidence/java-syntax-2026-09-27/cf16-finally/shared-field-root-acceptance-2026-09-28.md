# CF-16 字段型共享 catch-all：主线独立验收

主线合入 `f4b1148c` 后，root 独立构建 CLI，从固定 Java 8 `SharedFinally.class` 实时生成完整 `class-source`；它与本 change 归档的 `jarde-field-after/SharedFinally.java` 逐字节一致，SHA-256 `e2e966dad66c5d55a6e2c84adb31c6820be16a9edc5cdc408ed68e5950a5092a`。固定原 class SHA-256 `8677378307e4d1bdd422b6dbe6860c9fe1aca6c3eff944258c19078b71bfd0bd`。实时输出的 `handled` 有一个 `finally`、两条字面量返回和一次字段增量，无 `@bytecode`。root 分别重编原源码、固定 JADX 完整源码和**实时生成的** Jarde 完整源码，使用 `javac --release 8 -g:none -Xlint:-options` 及 `java -Xverify:all` 跑同一 Runner，正常/具名 catch 计数依次为原类 `1/1`、JADX `2/1`、Jarde `1/1`。原 class 而非 JADX 文本是副作用次数基准。

代码审阅确认：共享证书只新增三份准确的静态 int 字段读、常量、`iadd`、同字段写的完整 span；三个副本使用相同字段身份和常量，每个中间栈值只有唯一消费者。`guard.rs` 逐 BCI 核异常表覆盖、返回保存/加载、原异常重抛及清理块入口；`build.rs` 只把第一份完整 span 作为现有 `finally` 的正文，其余物理 BCI 保留在来源图。集成测试核目标方法全部 33 个物理 BCI 来源及预算/取消原子性。调用型共享清理继续通过。

root 另从冻结原 class 单项改出字段目标、增量常量、异常保护终点三个负例，并加载本 change 的中途入口 class；四个类均在 `java -Xverify:all` 下成功加载，实时 CLI 四次都保留 `@bytecode`/停止标记且不输出 `finally`。这证明拒绝出自恢复证书，而非无效 class。`cargo test -p jarde-java --tests --locked`、`cargo check --workspace --locked`、`cargo fmt --all -- --check`、`openspec validate recover-shared-field-finally --strict` 和 `git diff --check` 通过；专用 Cargo target 已清理。

本次仅闭合 `SharedFinally.handled` 的字段型三副本。`FinallyOnce.handled/escaping` 等不同保护形态仍需独立证明，CF-16 整单元不标为追平。
