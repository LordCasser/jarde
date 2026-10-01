# 三方行为对照：组合形态折叠（任务 3.2）

三方 = 原冻结 class / 固定 JADX（`/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx`，1.5.6-dev，Java-input）/ Jarde 本 change 后 `class-source` 重编。每方 `javac --release 8` 编译后 `java -Xverify:all` 运行，逐路径比对 stdout。

## `fam.jar` — `p.Combo`（SHA 见 sha256.txt / [sha256-mix.txt](sha256-mix.txt)，fam.jar 未改动）

| 方 | `ADD.apply(3,4)` | `MUL.apply(3,4)` | `ID.apply(3,4)` |
| --- | --- | --- | --- |
| 原 class | `7` | `12` | `0` |
| JADX（`ADD(1) { … } MUL(2) { … } ID(0);`） | `7` | `12` | `0` |
| Jarde 重编（[combo-mix.jarde.java](combo-mix.jarde.java)，同形折叠 + `private Combo(int arg0) { this.code = arg0; }`） | `7` | `12` | `0` |

行为基线 [orig.out](orig.out)（`7`/`12`/`0`）逐路径一致。

## `variants-mix.jar` — 三个变体（SHA 见 [sha256-mix.txt](sha256-mix.txt)）

| 变体 | 原 class | JADX | Jarde 重编 |
| --- | --- | --- | --- |
| `p.Trio`（三参带体） | `ONE:10 0` / `TWO:20 1` / `ONE TWO` | **其输出 `javac --release 8` 拒编**——byte 实参拼写为裸 `4`，丢失 `(byte)` 窄化（`不兼容的类型: 从int转换到byte可能会有损失`） | `ONE:10 0` / `TWO:20 1` / `ONE TWO`，逐行一致（`(byte)` 窄化按参数描述符拼写） |
| `p.Holder$Op`（嵌套+混合） | `7` / `12` / `0` | `7` / `12` / `0` | `7` / `12` / `0` |
| `p.Gs`（getstatic 参带体） | `11 A` / `22 B` | `11 A` / `22 B`（JADX 折叠） | 本 change 保持逐字段（边界登记见 [variants-before-after-mix.md](variants-before-after-mix.md)），无重编方 |

JADX 与 Jarde 对 `p.Combo`/`p.Holder$Op` 的折叠形状一致；`p.Trio` 差异点为 Jarde 按 ctor 参数描述符做 int 族窄化拼写（`B` → `(byte) v`），JADX 丢窄化导致其输出不可编译——JADX 为有界参照，非正确性证书（与既有证据立场一致）。

## 门禁汇总（任务 3.1）

- `cargo test --workspace --tests --locked --no-fail-fast`：272 个 suite，**2766 通过 / 0 失败**（含本片新增 5 测试与 enum 全家族、member-family 装配、旧 constant-bodies/DT-12 全部既有测试）。
- `cargo fmt --all -- --check`：通过。
- `cargo clippy --workspace --all-targets --all-features --locked` + CI 完整 30 项 `-A` 清单 + `-D warnings`：通过。
- `openspec validate --all --strict`：**234 项全部通过**（含本 change；任务说明中的 235 为上游计数，本 base 上全部项有效）。
- 磁盘纪律：全程 `df -h` 监控，完成即 `cargo clean`。
