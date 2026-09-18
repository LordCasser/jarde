# layer-jarde-crates 验证记录

约束：本 change 是**结构重组**，不兼带语义修复。每条记录携带**精确 commit** 与当时的全量数字；未绿不搬迁。

## 1.1 基线（2026-09-18）

**精确基线**：`0f3134b`（`docs: accept the call-context slice`）。前置条件已满足——P2 的 `0.3`/`0.3b`/`3.4` 及其依赖全部验收完毕（0.1–0.5 六项与 3.4 各有独立复核；3.4 经**四轮**复核后 Approve）。

**测试结果（该 commit 上实测）**：

| 项 | 结果 |
| --- | --- |
| `cargo test --workspace --all-targets --all-features --locked` | **628 passed / 0 failed / 1 ignored** |
| `call_context` 单测 | 36 |
| `cargo test --test p1_xref_golden --locked` | 5 passed |
| `cargo fmt --all -- --check` | 干净 |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 干净 |
| CI（`24ae87e`） | run [`35341736858`](https://github.com/LordCasser/jarde/actions/runs/35341736858) 四 job success |

**生产依赖树**（`cargo tree -p jarde --edges normal --depth 1`）：`blake3`、`flate2`、`noak 0.7.0`、`petgraph 0.8.3`、`rawzip`、`serde`、`thiserror`。按 design 的归属：`noak`/`rawzip`/`flate2`/`blake3` → reader；`petgraph` → jvm；`serde`/`thiserror` 按实际使用声明。**拆分不顺手升级任何第三方版本或 feature**。

**待拆分的 `src/` 模块**（19 项）：`artifact`、`budget`、`call_context`、`cfg`、`classfile`、`dispatch`、`engine`、`environment`、`error`、`ir`、`lib`、`members`、`model`、`multi_release`、`passes`、`providers`、`query`、`resolver`、`view`、`xref/`。

**本机环境限制（如实记录）**：验证期间本机链接器失效（Xcode 许可未接受，`xcrun --sdk macosx --show-sdk-path` 失败、链接报 `library 'System' not found`）。全部 cargo 命令均在 `SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk` 且 `PATH` 以 `/Library/Developer/CommandLineTools/usr/bin` 开头的环境下执行。CI 侧（Linux runner）不受影响。

**未做**：跨包私有访问与测试辅助消费者的盘点（1.1 的第二半，进行中）；未开始任何文件搬迁。
