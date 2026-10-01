## 1. 取证与基线

- [ ] 1.1 重放固定 C1/C2（SHA 核对）：定位 ctor 呈现的语句序发射点（与枚举 ctor super 首句先例的关系）；确认合成字段事实来源（ACC_SYNTHETIC/名字模式）在 reader 层的可得性；记录两模式基线输出。
- [ ] 1.2 构造并冻结至少三个 verifier 有效负例/变体：用户命名 `val$x` 字段（无合成标志，不重排）、pre-super 存与其它指令交错（保持逐字）、双捕获字段匿名类正变体；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 重排呈现

- [ ] 2.1 ctor 呈现层实现判据与重排（design 决策 1–2）；C1/C2 family 联编 `javac --release 8` 通过、运行与 fixture 基线一致；负例边界正确。
- [ ] 2.2 普通/枚举/委托 ctor 既有测试全绿（diff 级）；合成字段声明保留呈现。

## 3. 回归与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿、fmt、CI 完整 30 项 allowlist clippy、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。（root 代收尾于两个 glm-5.3-flash 通道配额耗尽后实测：全仓 2778/0、fmt/openspec 236/236、完整 30 项 allowlist clippy 干净；实现者的 in-crate p3_patterns 78/0 含 6 正负例。）
- [ ] 3.2 C1/C2 family 三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核判据、重排边界与三方行为，更新 DT 账本与巡查记录。
