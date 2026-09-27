## 1. 隔离和证明

- [x] 1.1 冻结最小 Java 8 父子类/runner：A、B 同包且各有同名 protected/包可见 boolean 字段，`B.set` 只写 A 字段；记录 BCI、CP、原/JADX/Jarde 基线与反射四值。
- [x] 1.2 在现有直接父字段证书中加入同包与准确访问标志检查，复用原有选中父类、唯一字段、descriptor、BCI、SSA 类型及 owner cast；不得新增 pass 或改写字段绑定。
- [x] 1.3 用错 owner/name/descriptor、非直接父类、跨包、private/static/final 和非 B 接收者负例确认拒绝及来源；保留 public 字段和 private accessor 正例。

## 2. 三方闭环与验收

- [x] 2.1 原 class、固定 JADX 与 Jarde 完整源码以 `javac --release 8 -g:none` 重编，`java -Xverify:all` 均得到 `true:true:false:false`；核对 Jarde 输出显式 A owner cast，并检查同一 BCI 的来源。
- [x] 2.2 重新检查固定 DT-29 组合中 `B.self` 的 BCI 7/12，确认受证且其余 C/D/root 问题未被混入；运行相关 Rust 测试、`cargo fmt --all -- --check`、`cargo check --workspace --locked`、`git diff --check` 与 OpenSpec strict。清理本轮 Cargo target，供 root 独立复验。
