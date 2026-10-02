# TWR 直接携带 finally 恢复（`recover-twr-direct-finally`）证据

`try (r) { … } finally { … }`（TWR 语句自身的 finally 子句）的锚点取证、变体前后对照、
三方一致与门禁记录。fixture 冻结于 [fixture/](fixture/)（SHA 见
[results/fixtures-sha256.txt](results/fixtures-sha256.txt)），`javac --release 8 -g:none`
编译；两个负例为单字段字节补丁（[fixture/patch_row.py](fixture/patch_row.py) 行表补丁、
[fixture/patch_code_u2.py](fixture/patch_code_u2.py) 指令操作数补丁），均
`java -Xverify:all` 链接+运行通过。

## 1. 锚点取证与 place 复用结论（tasks 1.1）

`twr-inner-finally` 的 `place_inner_finally` 只读 `parts.row.end_bci` 与 `parts.shape`：
要求 `at == row.end_bci`（副本恰在行保护区结束处出现）、走 `cleanup.len()` 条指令、
`cleanup_sequence` 等价（同一份判据）、退出读 goto/融合 goto/直落三形。direct 形同构：
**P3.voidBodySoloFin**（巡逻固定类，`t[c]f`）：

| 组成 | BCI |
| --- | --- |
| 资源 init `new P3; dup; invokespecial; astore_0` | 0–7 |
| 体 `aload_0; invokestatic touch`（体行 `[8,12)→19`） | 8–11 |
| 正常 close `aload_0; invokevirtual close`；`goto 35` | 12–18 |
| close 异常 handler（`[20,24)→27` suppression）`astore_1; aload_0; close; goto 33` | 19–26 |
| suppression `astore_2; aload_1; aload_2; addSuppressed@30`；`aload_1; athrow@34` | 27–34 |
| **finally 正常副本** `getstatic; ldc f; append; pop`；`goto 59` | 35–46 |
| **finally 异常副本** `astore_3@47; …同形…; aload_3; athrow@58` | 47–58 |
| 汇合 `getstatic; toString; areturn` | 59–65 |
| **finally 行**（any） | `[0,35)→47` |

判据差异在**锚点**而不在副本：inner 形行起点=体内资源 init 或最内层体起点；direct 形行
起点=整语句首指令（`resources[0].init.0`，P3 的 0）、行终点=close 链落点（35，即
`goto@16` 的目标=正常副本入口）。**实现**：`place_inner_finally` 重构为
`place_finally_copy(facts, row_end, shape)`（inner 两调用点改传 `parts.row.end_bci` 与
`&parts.shape`）——一份副本判据，两形共用；发现器平行（`trailing_finally_parts`，锚点
互斥由行起点几何保证，测试双向钉死：P1/P3/PD 恢复为 direct、T4/TF.v1 inner 文本逐字
不变）。split 保护行（bodyReturn 形的 `[34,51)→50`，覆盖 close handler+绑定 store）与自护
行（multiFin 形的 `[78,80)→78`）按"行全落语句自身机器内"（handlers/guards/close 链/
init/体/绑定 store）一并认领；三子句（TWR+catch+finally）显式拒绝。

主线基线（d1f835f9，实现前）：P3.voidBodySoloFin/P1.soloEach/TF.solo 整方法拒绝
`jre_guard_finally_copy`（P3@47/P1@52/TF.solo@54），finally 副本候选抢占失败（延迟保留
机制无 TWR 认领，最终以该拒绝呈现）。

## 2. 冻结变体与负例（tasks 1.2，逐一 `java -Xverify:all` 通过）

| fixture | 形态 | 原类运行 | 实现前 | 实现后 |
| --- | --- | --- | --- | --- |
| `PD.soloFin`（`t[c]f`） | P3.voidBodySoloFin 同形独立类 | pd-original.out 首行 | 拒绝（finally_copy@47） | **完整恢复**，重编运行逐字一致 |
| `PD.finReturn` | finally 自身 `return`（异常副本以 areturn 收尾，无重抛） | `t[c]` | 拒绝 | **仍拒绝**（无 rethrow 形副本对，登记现状） |
| `PD.multiFin` | 双层 TWR、外层语句带 finally（javac 平铺双资源；自护行 `[78,80)→78`） | `t[c][c]f` | 拒绝（finally_copy@78） | **恢复**：平铺 `try (a; b)` + finally（一形登记，未泛化多层特判） |
| `PD.bodyReturn` | 体内 `return` 穿过子句（split 行 `[34,51)→50`） | `t` | 拒绝（finally_copy@50） | **恢复**：声明与 `return` 在 try 内、finally 在其后（忠实求值序） |
| `PD.threeClauses` | TWR+catch+finally 三子句 | `t[c]f` | 拒绝 | **仍拒绝**（本片 Non-Goal；呈现 catch 会丢 finally 体） |
| `PDBroken.class` | soloFin 正常副本 `ldc@38` 指到 `E`（两副本不同形） | 首行 `t[c]E` | 拒绝 | **仍拒绝**（副本断链；同类 multiFin 仍恢复——拒绝精确到方法） |
| `PDOv.class` | multiFin 自护行 `[78,80)` end→92（压过异常副本全段） | 输出与原类相同（行在运行上惰性） | 拒绝 | **仍拒绝**（行越过语句机器；同类 soloFin 仍恢复） |

对照锚（回归）：`P3.voidBodyBranch`（纯 TWR）、`P3.voidBodyCatch`/`P1.twrCatch`（B 形态，
下一片）、T4/TF 全家族（inner 形）文本不变；`TF.solo` 由拒绝翻转为恢复（direct 形归本片，
`twr_inner_finally.rs` 断言同步更新）。

## 3. 三方对照（tasks 3.2）

原 class / 固定 JADX（dev，`--no-debug-info`）/ Jarde 重编（未恢复方法以抛
`UnsupportedOperationException` 的 stub 替换——切片惯例），全部 `java -Xverify:all`，
逐路径见 [results/pd-paths.txt](results/pd-paths.txt)：

- **恢复路径逐字一致**：`PD.soloFin`/`PD.multiFin`/`PD.bodyReturn`、`P1.soloFinally`、
  `P3.voidBodySoloFin`（及回归锚 `P3.voidBodyBranch`）三方同串。
- JADX（dev）对 direct 形不恢复（显式内联 close/suppression），输出语义等价、可编译运行
  且路径一致；**例外**：`PD.threeClauses` 的 dev 输出缺 catch 路径尾 return、不可编译
  （登记，双侧剔除）；`PD.finReturn` jadx 可运行、Jarde 侧 stub 剔除。
- 整类输出：p1-jadx/p3-jadx 与 orig 逐字；pd-jadx 前四行=orig 前四行（第五行
  threeClauses stub）；jarde 侧截断于首个未恢复方法（pd: finReturn@2、p1: twrCatch@2、
  p3: voidBodyCatch@1——均为本片前已拒绝的 B 形态，等待
  `recover-enclosing-catch-call-bodies`）。
- 输出与源码 SHA 见 [results/outputs-sha256.txt](results/outputs-sha256.txt)。

## 4. 门禁（tasks 3.1）

`cargo test --workspace --tests --locked --no-fail-fast`：**2818 通过 / 0 失败**（主线
2812 + 本片 6；本轮无 flake 复跑需求）；`cargo fmt --all -- --check` 干净；clippy（CI
完整 30 项 `-A` 清单 + `-D warnings`）零告警；`openspec validate --all --strict`
244 项全过。磁盘：全程 ≥61Gi 余量，报告前 `cargo clean`。
