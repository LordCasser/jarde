## 1. 基线与负例

- [x] 1.1 重放固定 S1：核 fixture SHA、twrHelper 的实参被引诊断与 plainHelper 对照恢复；定位实参呈现的实际类型来源路径并记录（槽决策 vs 值定义）。
- [x] 1.2 构造并冻结至少三个 verifier 有效变体：多 catch 各型的槽复用、handler 内先存副本再传参、非 handler 的槽复用读取（保持既有行为）；记录实现前后输出。

## 2. 值归属呈现

- [x] 2.1 实参/读取呈现对绑定 store 的 Caught 值按行类型拼写（design 决策 1）；S1.twrHelper 完整恢复、整类 `javac --release 8` 通过；对照与 1.2 变体边界正确；预算/取消不变。
- [x] 2.2 回归：17b 家族、`p3_twr_enclosing_catch`、`p3_java_recovery`、Catches 既有路径全绿。

## 3. 验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿、fmt、**CI 完整 30 项 allowlist clippy**（本地 1.98.0；CI 1.98.1 有 patch 漂移，推送后核对）、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [x] 3.2 S1 三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 正常与注入异常路径一致；记录输出 SHA。
- [x] 3.3 root 独立复核归属边界与三方行为，更新 CF-17/CF-15 账本与巡查记录。（root 于合并主线 3bf942a5 复核：S1.twrHelper `return tag(local0);` 完整恢复、对照逐字不变、全仓 2728/0、fmt/openspec 228/228。实现者对 design 机制描述的修正复核认可——载体是绑定入口 store 指令 + 入口引用自身 Named ty（多贡献边下为 Phi），非 `def==Caught` 判定；无 TWR 双具名子句同缺陷一并恢复为有意泛化。遗留：TWR 多子句/multi-catch 槽复用仍被区域级 gate 先行拒绝，属后续片域。）
