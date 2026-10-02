# TWR 正文内显式 finally 恢复（`recover-twr-inner-finally`）证据

落点取证、三层降低记录、变体前后对照与三方一致性。fixture 冻结于 [fixture/](fixture/)
（SHA 见 [results/fixtures-sha256.txt](results/fixtures-sha256.txt)），行为输出与源码 SHA 见
[results/outputs-sha256.txt](results/outputs-sha256.txt)。T4 为巡查固定类
（[../fixture/T4.class](../fixture/T4.class)，`d0507434…`）；`javac --release 8 -g:none` 本机
重编与冻结件逐字节一致（SHA 复核）。

## 1. 三层降低取证与 dispatch 判定（tasks 1.1）

`T4.nested`（`try (a) { try (b) { body } finally { mid } }`）的完整降低
（[results/T4-nested.javap.txt](results/T4-nested.javap.txt)，块头
`0 36 44 50 52 64 78 85 93 99 101`，无行号表——`-g:none`）：

| 层 | 组成 | BCI |
| --- | --- | --- |
| **TWR-a** | init `new T4("a"); astore_0`；主行 `[10,78)→85`（Throwable）；正常 close `[78,85)`；handler `[85,101)`（close+suppression `[86,90)→93`+rethrow） | 0..10 / 78..101 |
| **TWR-b** | init `new T4("b"); astore_1`；主行 `[20,29)→36`；正常 close `[29,36)`；handler `[36,52)`（close+suppression `[37,41)→44`+rethrow） | 10..20 / 29..52 |
| **finally-mid** | 主行 `[10,52)→64`（any）+ 自护行 `[64,66)→64`；正常副本 `[52,64)`（mid+goto 78）；异常副本 `[64,78)`（astore 4; mid; aload 4; athrow） | 52..78 |

（对照：**纯内层 try 的 finally 无自护行**——TF.v1 表 `[10,19)→31` 单行；**返回形**的正常
副本无尾 goto、直接落入 a.close——TF.v3；**TWR 自带 finally（solo）**的 finally 行从 BCI 0
起、覆盖资源 init 本身——见 §2。）

**dispatch 尝试序与失败环**（主线 a8996529，`resources()` 的候选行扫描先于链级证明）：

1. `finally_copy(row2)`＝`Some(64)`（catch-all、astore/mid/aload/athrow、含 Invoke、无
   close/addSuppressed/monitor）→ **抢占点**：`prove_finally_copy` 失败即整方法拒绝
   `jre_guard_finally_copy`@64。具体失败环（按序）：①行前存在 Invoke（a 的构造调用 @6——
   该切片"不在可能抛出的调用后推断 try 边界"）；②正常副本末位是 `goto` 而非 `Return`
   （该证书只认 return 形）；③受保护区 `[10,52)` 含内层 TWR 的分支与行、且被外层行
   `[10,78)` 覆盖（"无其它行拦截"检查不成立）；④副本等价/所有权同不成立。
2. 即使放行到链级证明（`twr()`，内层行 row0 起链 `[row4,row0]`），b 级 close 后
   `normal_close(52, slot a)` 于 mid 副本处失败（CloseOrder）；且 **a 级 close 组与其单前驱
   继续块被 canonical 图融合**（块 `78=[78,79,82,101,104,107]`，goto→101 无块边界）——
   主线对无守卫 close（`new` 资源）多层 TWR 的继续块读取本就无法完成
   （TF.v2 基线即 `jre_guard_close_order`@52 拒绝）。

**接入层结论（取证驱动）**：按 design 决策 1 落在 **TWR 证书**（`resources`/`twr`）——
finally-copy 候选的独立证明失败改为**延迟**（不改变无 TWR 语境的原拒绝），由 TWR 链级证明
先行认领，内层 finally 在其正文语境证明（子证书）；同时补上两处取证暴露的必要环节：
a 级无守卫 close 的**融合继续块**读取（`normal_close` 读块内 transfer 目标）与语句
**尾迹**（trail：语句自渲染被融合块中 close 链之后的普通语句，仅当该块终结方法且全部可
呈现）。

## 2. 冻结变体与负例（tasks 1.2，`java -Xverify:all` 逐一通过）

[fixture/TF.java](fixture/TF.java)（`--release 8 -g:none`，`TF.class 5e0a928c…`）与两个
单字段字节补丁负例（[fixture/patch_row.py](fixture/patch_row.py) 生成，均链接+运行验证）：

| fixture | 形态 | 原类运行 | 实现前（主线 a8996529） | 实现后 |
| --- | --- | --- | --- | --- |
| `T4.class`（巡查固定） | `try (a) { try (b) { body } finally { mid } }` | o4.out（首行 `body[b]mid[a]`） | `nested` 拒绝：`jre_guard_finally_copy`@64 | **完整恢复**，重编运行=o4.out 逐字 |
| `TF.v1` | 外层 TWR + 纯内层 try/finally（无内层 TWR） | `bodymid[a]` | 拒绝（finally_copy@31） | 恢复：`try (a) { try { body } finally { mid } }` |
| `TF.v2` | 双层 TWR、无 finally（回归锚） | `body[b][a]` | 拒绝：`jre_guard_close_order`@52（**主线缺口**，见 §1.2） | 恢复：平铺 `try (a; b) { body }`（融合继续块读取修复的直接收益） |
| `TF.v3` | 内层 finally 的正文含 `return` | `4`（length 在 mid 前求值） | 拒绝（finally_copy@41） | 恢复：`int local1 = log.length(); return local1;` 在内层 try 内、finally 在其后 |
| `TF.nested` | T4 形（独立类复核） | `body[b]mid[a]` | 拒绝（finally_copy@64） | 恢复（与 T4 同文） |
| `TF.solo` | TWR 自带 finally（无外层 TWR 正文） | `body[b]mid` | 拒绝（finally_copy@54） | **仍拒绝**（本片不认：finally 是该 TWR 自身的子句而非某正文形态；split=0 无外层资源） |
| `T4Ov.class` | row2 `[10,52)→64` 的 end 52→90（内层行压过 a 清理与 suppression 行 `[86,90)`） | 链接+运行通过 | 拒绝 | **仍拒绝**（`row_f.end ≠ E`，内层 finally 不自洽） |
| `T4Self.class` | row3 自护行 `[64,66)` 的 start 64→37（压过 b 的 suppression 机制 `[37,41)→44`） | 链接+运行通过 | 拒绝 | **仍拒绝**（自护行必须恰覆盖绑定 store） |

## 3. 实现（tasks 2.1/2.2）

- **guard.rs**：`Shape::Resources` 增 `inner_finally`/`trail`；`InnerTwrFinally`
  （split、行号、自护行、副本 span）+ `inner_finally_parts`（发现与自洽：S 锚定内层资源
  init 起点或最内层行起点；内层层级 handler 全在 `[S,E)` 内；自护行恰覆盖绑定 store）
  + `place_inner_finally`（副本等价 `cleanup_sequence` 复用、退出汇合下一外层 close 组——
  goto/融合 goto/直落三形）；close 链在 split 处精确插接一次；副本/处理器 span 计入
  pieces/cleanup（所有权与 slot 消亡分析）；覆盖不足时恰一 companion 行（同型同 handler，
  精确 span）——T4 形由外层行覆盖、v3 形由 companion 承载；`finally_copy` 独立证明失败
  改为延迟保留（无 TWR 认领时原码原 BCI 拒绝）。`normal_close` 无守卫形增融合继续块
  读取；`twr()` 增 trail（块终结方法、全可呈现、非语句自身首块）。
- **build.rs**：`Shape::Resources` 呈现分支——`inner_finally` 时内层 `StmtKind::Try`
  （内层资源、正文、`finally_body`=mid 副本）作为外层 try 的唯一正文语句；`returns` 归
  内层正文；trail 在语句后 `range` 渲染。无新呈现函数（复用 `StmtKind::Try`+`body_range`）。
- **测试**：`crates/jarde-java/tests/twr_inner_finally.rs`（T4.nested 整文断言、四变体、
  负例保持、纯域 byte-identity：`Guarded.one` 与 `FinallyNormal.run`）；guard.rs 增复合形
  预算/取消停止传播探针。纯 TWR/finally 家族经全量套件零回退；预算/取消原子性不变。
- 环境附记：`tests/p3_boolean_short_circuit_return.rs` 的深链渲染栈余量随本片代码布局再次
  翻转——第二个测试按既有惯例（nested-mon 片先例）改在显式 16MiB 栈线程上运行同一组
  断言（断言零改动）。

## 4. 三方对照（tasks 3.2）

原 class / 固定 JADX（dev，`--no-debug-info`）/ Jarde 重编，`java -Xverify:all` 逐路径一致：

- **T4**：三方输出 SHA 同为 `0d6a8997…`（= o4.out：`body[b]mid[a]`/`3:42:null`/`10`）。
  JADX（dev）对本形不恢复（显式内联 close/suppression），其输出可编译运行且路径一致。
- **TF**：v1/v2/v3/nested 四路径三方逐字一致（[results/tf-paths.txt](results/tf-paths.txt)）；
  `solo` 双方（Jarde 不认、JADX 显式内联）按惯例剔除后对比——Jarde 重编件以抛
  `UnsupportedOperationException` 的 stub 替换该未恢复方法（记录于
  [results/tf-jarde-recompiled.out](results/tf-jarde-recompiled.out) 尾行）。
- Jarde 重编源即 [results/after-T4.jarde.java](results/after-T4.jarde.java)、
  [results/after-TF.jarde.java](results/after-TF.jarde.java)（`javac --release 8` 整类通过）。

## 5. 门禁（tasks 3.1）

`cargo test --workspace --tests --locked --no-fail-fast`：**2810 通过 / 0 失败**（`p4_plugins`
一次运行计时性翻牌、单测复跑通过——已知 flake 清单内；清单其余成员单测复跑全绿）；
`cargo fmt --all -- --check` 干净；clippy（CI 完整 30 项 `-A` 清单 + `-D warnings`）零告警；
`openspec validate --all --strict` 242 项全过（含本 change）。磁盘：全程 ≥2.9Gi 余量运行，
完成即 `cargo clean`。

## 6. 边界与遗留

- `solo`（TWR 自带 finally）保持拒绝（非本片正文形态）；TWR 内 catch（具名/多 catch）不做；
  内层 finally 内抛错路径呈现忠实序（athrow 保留为 finally 语义）；三层嵌套不做；
  monitor×TWR 复合不做。
- 正文在内层语句**之前**另有语句的形态（`try (a) { x(); try {…} finally {…} }`）不认
  （S 锚点不成立）——登记为后续切片候选。
- trail 仅认"融合块终结方法"的形态；带分支尾迹的继续块保持 `jre_guard_continuation` 拒绝。
