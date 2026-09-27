# CF-08 二层 if 带效果双出口：主线独立验收

主线 `ff018325` 在独立 Cargo target 构建 CLI（SHA-256 `c1e928262242ec03a3bce9b72535e09ede88f14566c167e20a545b6d548ebefd`），root 用固定 `two-level-if-baseline/replay.py --require-jarde` 重跑原/JADX/Jarde 三方完整类。脚本校验 Java 输入、原 class、JADX checkout revision `2fb1b16386941660fda07e9017285aec40fcb37f` 及七个固定测试/算法文件哈希。三方均通过 `javac --release 8 -g:none` 和 `java -Xverify:all`，四条输出逐行一致：`-1:0 / -1:0 / 3:0 / 8:1`。Jarde 的 `pick` 为 structured，无 `@bytecode`；Region 所有权为外层 `[0,4,9,16,21,23,28,37,46,49,55]` 与单次续接 `[61]`，内层 join BCI 55 未被循环单独认领。

root 从 `TwoLevelIfNegatives.java` 和 Runner 重新按 Java 8 编译，两个 class 与归档逐字节一致（SHA-256 `8f29f5db6c51e8a62fbcec09b4f162e498004ba19159daa56721563a2af1b1be`、`55499494c10891f9a3bb2d29dc32a27b75a7383ae801dac744ab687f8ab28f0a`）；`java -Xverify:all` 对七个同层负例依次输出 `8 / 9 / 7 / 11 / 13 / 14 / 15`。Rust 定向测试逐一检查它们不能被新三来源证书接受，且保留物理 BCI；另核 SSA 三输入 φ、来源图、预算及取消的原子停止、旧顶层和单层形态。

root 完整运行 `cargo test -p jarde-java --tests --locked`（库 235/235，集成测试全通过）、`cargo build -p jarde-cli --locked`、`cargo check --workspace --locked`、`cargo fmt --all -- --check`、`git diff 0e46cc74..HEAD --check` 与 `openspec validate recover-two-level-effectful-loop-join --strict`，均通过。固定 `TestNotIndexedLoop` 原 Java 8 class 仍输出 `null / null / f / h`，当前 Jarde 在内层 BCI 4 报 `jre_region_arms_do_not_meet`，未覆盖 `[64,25,38,55,58]`，完整源码仍有局部跨引用和缺返回；其 File 构造、虚调用及复杂控制流另行拆分，CF-08 保持已证差距。
