## 1. 取证与基线

- [x] 1.1 重放固定 C1/C2（SHA 核对）：定位 ctor 呈现的语句序发射点（与枚举 ctor super 首句先例的关系）；确认合成字段事实来源（ACC_SYNTHETIC/名字模式）在 reader 层的可得性；记录两模式基线输出。
- [x] 1.2 构造并冻结至少三个 verifier 有效负例/变体：用户命名 `val$x` 字段（无合成标志，不重排）、pre-super 存与其它指令交错（保持逐字）、双捕获字段匿名类正变体；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 重排呈现

- [x] 2.1 ctor 呈现层实现判据与重排（design 决策 1–2）；C1/C2 family 联编 `javac --release 8` 通过、运行与 fixture 基线一致；负例边界正确。
- [x] 2.2 普通/枚举/委托 ctor 既有测试全绿（diff 级）；合成字段声明保留呈现。

## 3. 回归与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿、fmt、CI 完整 30 项 allowlist clippy、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。（root 代收尾于两个 glm-5.3-flash 通道配额耗尽后实测：全仓 2778/0、fmt/openspec 236/236、完整 30 项 allowlist clippy 干净；实现者的 in-crate p3_patterns 78/0 含 6 正负例。）
- [x] 3.2 C1/C2 family 三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [x] 3.3 root 独立复核判据、重排边界与三方行为，更新 DT 账本与巡查记录。（root 于合并主线 daa4fb31 复核：`C1$1`/`C2$Inner` ctor 首句 `super();`、合成字段写入后移按原序、合成声明保留；family 联编运行 25/6、10 与基线一致；非合成/交错/计算值负例不重排；全仓 2778/0〔首跑 1 例 engine flake 单跑与复跑均过〕、fmt/openspec 236/236。收尾由 root 在两个 glm-5.3-flash 通道配额耗尽后代完成——与 scv 切片同模式，见 impl-record.md。）

> （root 2026-10-04 账本补勾：实现者被配额终止于机械收尾段，其 1.1–3.2 工作实际完成并经 root 于 daa4fb31 验收，当时仅勾了 root 自身的 3.1/3.3，致账本失真。据 impl-record.md、results/ 基线与 in-crate 测试实证补勾；**变体以 `p3_patterns.rs` 的合成 class 生成器（`capture_ctor_class`）实现而非独立冻结 fixture 文件**，覆盖合成捕获/enclosing/双捕获正例与非合成同名字段、pre-ctor 写保持两负例，`-Xverify:all` 运行由 family 联编验收覆盖。）
