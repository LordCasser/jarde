# CF-19 双臂多出口验收

root 在合入主线后独立检查 `guard::monitor_branches` 的类级输入边界：恰好三处 monitor exit（两条正常返回与共享 handler）、同一锁、单个分支和两条直线臂、每臂单次消费返回值、精确两段 catch-all、唯一共享重抛 handler 及 handler self row。Region 只认领已证明的块，builder 为两条返回绑定各自正常退出。修审时去掉了无效的全区间 `explained` 自证，改为要求 handler 紧接第二个返回；正常 cleanup 的两个 lock load 和完整 handler BCI 也纳入 plan 来源，并加定向断言。证书只接受这个有界形状，不扩成通用监视器区域递归。

在临时 Cargo target 中重新运行 `cargo test -p jarde-java --lib --locked`（216 项通过）、`cargo check --locked --workspace`、`cargo build -p jarde-cli --locked`。从原始 Java 源重建 class，SHA-256 仍为 `6658190aa7575f8be5095d71aa3ee07aa453d464666b1d34dea6a962950c45f2`。原始类、冻结 JADX 完整源码、修后 Jarde 完整源码分别以 `javac --release 8 -g:none` 编译并用 `java -Xverify:all` 运行同一个 runner，三方逐字输出：

```text
first:return=10:trace=1
second:return=20:trace=2
first-throws:throw=java.lang.IllegalStateException:same=true:trace=1
```

修后 Jarde 源码 SHA-256 为 `2ea0d9473b8139cbcf855f032be90c8426d876832fc2868387fdb1590a9b559d`。真实 Java 8 三臂 [负例](../../evidence/java-syntax-2026-09-22/synchronized-multi-exit/negative-three-arm/README.md) 由 JVM verifier 接受；修后 Jarde 对整个 `choose` 方法保留 bytecode fallback，没有发布双臂投影。`monitor_branch_tests` 2/2 再次通过，断言正常退出、返回、handler 全部 BCI 进入来源。`cargo fmt --all -- --check`、`git diff --check`、`openspec validate --all --strict`（139/139）通过。临时目标目录自动清理，验收后可用磁盘约 78 GiB。

`cargo test -p jarde-java` 的全目标编译仍在主线既有 [匿名分配测试 API 漂移](../../evidence/jadx-feature-inventory-2026-09-27/anonymous-allocation-test-api-drift.md) 处停止；本项不修改该测试。root 另跑 reader lib：176 项通过、1 项仅因累计 fixture census 从固定的 `(330, 1730, 160, 1000, 8)` 漂至 `(351, 1804, 160, 1006, 8)` 失败；corpus fingerprint 4 项通过、1 项因已提交但未入 manifest 的语法 fixture 失败、1 项 ignored，独立债务见 [corpus pin 漂移](../../evidence/jadx-feature-inventory-2026-09-27/syntax-fixture-corpus-pin-drift.md)。代理运行严格 Clippy 时报告 30 项主线 lint，新增 guard/build/region 行没有警告；root 未将 Clippy 报告记作通过。这些维护门禁需在独立改动中恢复，不改变 CF-19 冻结三方语义验收结论。
