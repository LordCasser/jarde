## 1. 基线与负例

- [x] 1.1 重放固定 W1/W2：核 fixture SHA、`mix`/`contNoJoin` 的 `ownership_overlap`@9 基线与 `noCont` 恢复对照；在 arm 出口分类处定位现状首次失败点并记录。
- [x] 1.2 构造并冻结至少五个 verifier 有效负例/变体：arm 出边到非 latch/test 的任意块（保持拒绝）、两 arm 都 continue、default 整体 continue（无语句）、arm 内 break 出 loop、string switch 内 continue（回归现状如实记录）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 出口分类与构造

- [x] 2.1 switch arm 出口分类（design 决策 1–2）：continue 边呈现 arm 内 `continue;`、join 排除、join==latch 形态、selector 唯一属 Switch；W1/W2 两形完整恢复且整类 `javac --release 8` 通过、行为三方一致。
- [x] 2.2 Fallback 互斥划分（design 决策 3）：退化路径无重叠块集，诊断指向真实首个 FallbackReason；既有全部 quote 场景的诊断不劣化（抽查既有 fallback 测试期望）。

## 3. 回归与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 noCont、named-catch switch、string switch、labeled-loop 全部既有切片）、fmt、CI 完整 30 项 allowlist clippy、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [x] 3.2 W1/W2 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致（`70`/`36`/`70`）；记录输出 SHA。
- [ ] 3.3 root 独立复核出口分类边界、fallback 划分与三方行为，更新 CF-13 账本（勾销已证差距或收窄为剩余形态）与巡查记录。
