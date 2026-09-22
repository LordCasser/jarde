# §5 G4：重排与专项交付（主 Agent 记录）

## 5.1 测量后的重排与停止决定

在后准入的候选上重测的剩余耗时占比（`g0-instrumentation.md` 的存量数字 + `g3-regression.md` 的后准入读数；**占比未沿用旧热点**）：

| 排名 | 剩余项 | 后准入状态 | 决定 |
| --- | --- | --- | --- |
| 1 | **窗口的有序交付**（`take_front`，W6a w=4 的 ~90% 请求段） | 已由 bulk 的二级额度修复并**裁定为默认**（bcprov w8 1.86 s、s2-009 w8 6.51 s discard；w8/w1 1.79–2.05×） | 无剩余工作；不再进下一轮 |
| 2 | **发现与准备**（W1 的 `prepare` 相 252–268 ms，属调用方发现成本） | 不在本专项范围（它量的是调用方如何发现目标，不是引擎内部） | 移出本专项，记为其证据里的既有说明 |
| 3 | **编码与写出**（39 MB 输出 +307 ms / 写盘 +145 ms） | 上界 <5%（编码 +2.8%、写盘 +3.6%） | 本轮**否决**，不再重排进来 |
| 4 | **查询 unit 级停止**（一页 1 条仍付整个 unit；2 条/页时 265 页边界 ≈278 ms，默认页型 ≈0.3%） | 子 spec 未立，归 demand-driven 的 D4 收尾面 | **暂缓**，触发条件=子 spec + 真实小页负载 |
| 5 | **O2 的读复用** | ✓ Complete（168/187 命中、工作维度 0 移动） | 已交付，退出排名 |
| — | O5 更高层 store/淘汰、O6 预热、O7 的 W6b、O8 的 CLI 长度固定点 | 各自带触发条件 | **暂缓**，不因已有专项而自动实施 |

**停止决定**：本专项的准入面已经走完一轮——唯一准入项已交付并回归，其余项要么已由其它 change 交付、要么被有证据地否决或暂缓。**不再有下一轮自动重排**：任何重启都需要新的宿主需求证据（W6b/预热的触发条件）或子 spec（O3）。

## 5.2 可复现交付

**命令与原始样本**

| 交付 | 命令 | 原始数据 |
| --- | --- | --- |
| G0 基线（W1–W5、W6a） | `python3 evidence/run-baseline.py --tag {fixture,bcprov} --repeats 10 --features test-support --out baseline-*-raw.jsonl --run …`（配置清单见 `baseline-*-summary.txt`） | `baseline-{fixture,bcprov}-raw.jsonl` |
| 后准入回归 | 同上，`--tag *-postadmission` | `postadmission-{fixture,bcprov}-raw.jsonl`、`postadmission-*-compare.txt` |
| O1–O8 调查 | 见 `o{1..8}-*.md` 各自的命令段 | `investigation-*-raw.jsonl` |
| G1/G2 准入实验 | `evidence/g1g2-o2-campaign.py`（两臂同一二进制、唯一变量为开关、11 格 × 2 臂 × 10 次交错） | `g1g2-o2-raw-*.jsonl` |
| 对账脚本 | `compare-campaigns.py`（按 (configuration, sample) 分组，harness 增长单列） | — |
| 代码检查 | `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`、`cargo test --workspace --all-targets --all-features --locked` | `g0-*-summary.txt`、`investigation-workspace-test.log`、本轮 **1603 passed / 0 failed / 18 ignored** |
| OpenSpec | `openspec validate --all --strict --no-interactive` | **25/25** |

**分项调查结论与子 change 状态**：见 `o-investigation-summary.md` 与 `g1g2-summary.md`；子 change 状态见 `g3-regression.md` §4.1。

**收益与代价、回退记录**：见 `g1g2-summary.md` 的实验表与 `g3-regression.md` §4.3；每项的暂缓/否决理由在其 `o*.md` 的第 7–8 段。

**一处诚实边界**：G0 的进程内 harness 与协议驱动（`ba2076a` 轮）**不可互换**，同一单元格相差 1.125×（10/10 同向），差值定位在 G0 自己的 `prepare` 相与其 `request` 相开销。因此本文件的所有读数只在 G0 体系内做前后对照，**不与协议行并列**（见 `openspec/benchmark-protocol.md` 的对应段落）。

**调查完成 vs 产品交付**：本专项的全部任务（1.1–5.2）均为**调查/准入/跟踪**，其产品交付分别落在 `reuse-selected-class-read`（已 Complete）与其它已 Complete 的 change；本专项自身不含生产实现。
