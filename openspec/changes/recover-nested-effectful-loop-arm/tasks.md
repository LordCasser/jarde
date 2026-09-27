## 1. 固定输入与当前拒绝

- [x] 1.1 独立重跑 `nested-effectful-baseline/replay.py`，校验输入/class SHA、固定 JADX revision 和算法文件哈希；记录原/JADX 四行验证运行相同、当前 Jarde 未覆盖 BCI 11/17/26/35/38/44 且源码 Java 8 编译失败。

## 2. 有界嵌套 Region 证明

- [x] 2.1 在现有带效果双出口证书上证明单个外层 `if` 臂的边界、循环/两出口的唯一归属、BCI 44 内部 join 的两个正常前驱及直线尾段到 BCI 50；运行新的正例定向 Rust 测试，确认 Region/源码覆盖且不改局部引用门。
- [x] 2.2 用额外循环入口、异向出口、绕过尾段、内部 join 第三入边、异常边等 verifier 有效负例确认逐 BCI 安全拒绝；运行预算耗尽与预先取消测试，确认无半份结构化输出。

## 3. 三方集成验收

- [x] 3.1 用冻结脚本的 `--require-jarde` 门槛重编原/JADX/Jarde 完整类源码并执行 `java -Xverify:all`；四行均为 `-1:0 / 8:1 / 3:0 / 8:1`，Jarde `pick` 无 `@bytecode`，物理 BCI 11/17/26/35/38/44/50 来源与 Region owner 可查。
- [ ] 3.2 重放已验收的顶层带效果双出口、单出口循环臂及相邻 if/loop Rust 测试，并确认固定 `TestNotIndexedLoop` 仍保守拒绝；通过 `cargo fmt --all -- --check`、`cargo check --workspace --locked`、`git diff --check`、`openspec validate recover-nested-effectful-loop-arm --strict`，root 记录验收并清理专用 Cargo target。
