# CF-16 调用型共享 catch-all：主线独立验收

在主线 `cac3cbe9` 上，root 使用重新构建的 CLI 从固定 Java 8 `SharedFinallyCall.class` 实时生成完整 `class-source`，与归档的 `jarde-call-after/SharedFinallyCall.java` 逐字节一致（SHA-256 `b5d33006e62fb6fbe6e93733b9af23503d4b5b9ab8d449f4a7537de1c98b7a12`）。原 class SHA-256 为 `644c0da948fcb64673898bff8f5509d9cefac7ecd7f58399e0f18b0491f2f325`。root 分别以 `javac --release 8 -g:none -Xlint:-options` 重编原源码、固定 JADX 完整源码及实时生成的 Jarde 完整源码，并以 `java -Xverify:all` 运行 Runner：原类和 Jarde 均为 `normal:1 / caught:1`；固定 JADX 为 `normal:2 / caught:1`。原 class 是副作用次数的验收基准。

root 另行复核了 `SharedFinallyExtraReturn` 与 `SharedFinallyOtherTarget` 的 Java 8 完整源码和 verifier 行为：前者为 `early:1 / normal:1 / caught:1`，后者在真实 `other()` 目标上为 `normal:1 / caught:1`。Rust 定向测试用这两种 verifier 有效的结构及 row 次序/范围变化确认守卫拒绝，并覆盖 `handled` 的 24 个物理 BCI 来源。完整 `cargo test -p jarde-java --tests --locked` 通过（库测试 235/235，各集成测试通过）；`cargo check --workspace --locked`、`cargo fmt --all -- --check`、`openspec validate recover-shared-catchall-finally --strict` 与 `git diff --check` 通过。独立只读审阅未发现高置信正确性漏洞，特别确认 Region 对 try/catch 两段拒绝外部中途入口，Guard 保持异常行次序与边界，Builder 失败时回滚整个候选。

此验收只对应调用型三副本。原有字段增量 `cleanupCount++` 和 `FinallyOnce.escaping` 继续留在 CF-16 已证差距；本次不改变整单元状态。
