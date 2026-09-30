# `recover-preceded-statement-catches` 实施复放记录

实施分支上的复放记录（2026-09-30）。基线 = `b3a01e24` 的恢复代码（`1170c943` 与其在代码上相同，只加文档）；实施 = 本切片的双修复。所有诊断来自 `jarde-cli recover --policy single-class --format json`，恢复文本来自 `class-source --format text`，重建二进制为 fresh `cargo build --workspace --locked`。

## 1.2 近邻（`fixture/near/`，SHA 见 [near-sha256.txt](near-sha256.txt)）

| 近邻 | 形状 | 基线（修复前） | 实施（修复后） |
| --- | --- | --- | --- |
| `P1FieldWrite.putfieldPrefix` | `putfield` 前置具名 catch | 完整恢复（既有字段赋值门槛已覆盖） | 完整恢复（不变） |
| `P1FieldWrite.putstaticPrefix` | `putstatic` 前置具名 catch | 完整恢复（同上） | 完整恢复（不变） |
| `P2GetstaticRead.run` | `getstatic` 消费语句（`System.out.println`）前置 | `jre_guard_resource_init` + `jre_region_uncovered_blocks`，整方法回退 | 完整恢复 |
| `P3StorePrefix.main` | store 前置降级（`r = open();`，N1 的第二例） | `jre_guard_handler` | `jre_guard_handler`（不变） |
| `P4LateSplit.g` | 真分支切断链（head 33）之前存在更早同 owner `toString`（BCI 18） | `jre_concat_split` 误指 **BCI 18**（入口块链 1 的尾） | `jre_concat_split` 指向 **BCI 68**（该链自己的消费点）；链 1 照常呈现 |
| `P5TwoResources.two` | 双资源 TWR（调用初始化、体内 void 调用、保存返回） | 完整恢复 | 完整恢复（不变） |

M5（`invokestatic helper()` 前置）与原始 `FinallyOnce.main` 的基线/实施行为见 [README.md](../README.md) 的拆分表；实施后两者均完整恢复，M5 与 M3 的 class-source 逐字等于冻结期望 [M5.exp2.java](M5.exp2.java) / [M3.exp2.java](M3.exp2.java)，原始类逐字等于 [fo.exp.java](fo.exp.java)。M1/M2 的四个方法文本与修复前构建的输出逐字节一致（`tests/p3_preceded_catches.rs` 以常量逐字钉死）。

## 三方重编与运行（固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f`，dev）

原 class、JADX-on-class、JADX-on-Java-input（冻结 `.java` → `javac --release 8` → JADX → 重编）、Jarde class-source 四条腿各自 `javac --release 8` 重编后 `java -Xverify:all` 运行：

- `M5`、`P1FieldWrite`、`P2GetstaticRead`、`P5TwoResources`：四条腿 stdout 逐字节一致（正常与异常路径由各自的 `mode` 驱动覆盖；M5 单异常路径）。
- `M3` 与原始 `FinallyOnce`：Jarde 腿与原 class 逐字节一致（`normal:2/caught:arg:2/state:1` 与 `normal:1/caught:arg:1/state:1`）；两个 JADX 腿把 `handled` 的 finally 渲染为正常路径多计一次（`normal:3` / `normal:2`）——这是固定 JADX 对 finally 的既有渲染属性，与本次修改无关（JADX 腿不经过 Jarde 代码），按任务 3.2/4.1 的口径以“main 与固定 JADX 结构一致 + Jarde 腿行为与原 class 一致”验收。
- Jarde 腿 `main` 与固定 JADX 输出结构一致（同三条语句、同唯一 `try/catch`）。

运行 stdout SHA-256（Jarde 腿 = 原 class 腿）：

```
99be8352b79ab0d5504ba57a1784ab2b41a5616a8d8b49e7b6b6be11db5a3a2b  FinallyOnce jarde-run
43459b1d3944c1b661f0813c7cec22bf188cdf8ea85d4ec7175a6b1658a2263b  M3 jarde-run
836a2244bead408a670866b5e046e9fa54aedf5c97e84078c599948ad5b9c2e8  M5 jarde-run（= M5 orig-run 同值）
c9bf7bfc467e8201d5c90f4b9de8562266205047e8c01639da8e8beb2be9ec20  P1FieldWrite jarde-run（四腿同值）
c145dc6389732f9f1750de34948d24be9d47029f4c42c1cace85480b67fbe043  P2GetstaticRead jarde-run（四腿同值）
6295b85e51f3c4760d78abfa007ff2a5d6f1937aa51a8456f3266dd8c87588b3  P5TwoResources jarde-run（四腿同值）
```

## 门禁

- `openspec validate --all --strict`：217/217（修复前后各跑一次）。
- `cargo test -p jarde-java --tests --locked`：24 个测试二进制全绿（基线 23 + 新增 `p3_preceded_catches` 11 项；guard 单元测试含新增 2 项）。
- `cargo fmt --all -- --check`、CI 同款 `cargo clippy --workspace --all-targets --all-features --locked -- -A … -D warnings`、`cargo check --workspace --all-targets --all-features --locked`：全部通过。
