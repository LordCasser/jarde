# `recover-switch-arm-loop-exits` 实现证据（CF-13 第一片，2026-10-01）

[巡查](../../README.md)登记的两形固定复现在本片闭合：switch arm 直跳外层 loop latch 的边恢复为 arm 内
`continue;`，全部 arm 出口即 latch 时 join 即 latch；退化 Fallback 改为互斥划分。本文记录首个失败点
定位、变体前后行为、恢复输出与 SHA。基线一律为分片前主线 `f3452809` 的产物。

## 首个失败点定位（任务 1.1）

fixture SHA 与巡查一致（`../../results/fixture-sha256.txt`）。基线（`results/W1.baseline.java`、
`results/W2.baseline.java`）中 `mix`/`contNoJoin` 整方法 quote，诊断
`jre_region_ownership_overlap`@BCI 9；`noCont` 完整恢复。

按 for-header 是否被证明分岔（`region.rs` `for_header_candidate` 的共享 latch 守卫：latch 块含
body 效应且有早入边时拒绝 for header）：

* `noCont`：update 块同时含 `total += 10`，for header 被拒 → `continue_target` = header 测试块；
  switch 的 post-dominator 是 in-loop join 块 ≠ header → 走普通 join 路径恢复。与本片无关。
* `W1.mix`：for header 证明成立 → `continue_target` = update 块（BCI 66）。switch 的
  post-dominator 恰为 66 → 进入 local-join 证明；候选 63（`total += 10`）被拒，原因在
  `switch_loop_join` 的 per-arm 有界走访：default arm（entry 52）的 if 两支分别到达候选 63 与
  continue 66，旧判定 `reaches_join && reaches_continue → invalid` 把整个候选否决 →
  `SwitchShape`@9 拒绝。
* `W2.contNoJoin`：同上但无任何候选（latch 本身是 continue 目标被排除，其余块单前驱）→
  `SwitchShape`@9 拒绝。

次生缺陷即巡查根因 2：switch 拒绝的 `Fallback[9]` 与 loop 拒绝的合并 quote `Fallback[4,9]`
对块 9 双重认领，顶层 `overlapping_owner` 命中 → 整方法 quote 且诊断被置换为
`ownership_overlap`@9，真实首个失败（switch 出口分类）被掩盖。

## 实现（任务 2.1/2.2）

出口分类（design 决策 1–2，`crates/jarde-java/src/region.rs`）：

1. `switch_loop_join` 接受 mixed arm：到达 continue 目标的支路必须经自身单后继显式
   `goto`（既有 `explicit_transfer` 检查保留），同 arm 其余支路正常完成于候选 join 不再否决
   候选（W1 形态）。
2. 返回值改为三态 `SwitchLoopJoin`：唯一候选 → `Local`；**无候选且循环内不存在被 ≥2 个
   arm 自身路由共享的块**（`shared_tail` 侧信道，逐 arm 有效路由计数）→ `ContinueTarget`，
   join 即 latch、arm 自然落出（W2 形态，arm frame 不携带 `switch_continue`）；否则
   `Refused`（跨 case 路由、extra entry、双候选等保持既有拒绝——boundary 类四方法逐一对照）。
3. arm 终边呈现复用既有 `Region::LoopContinue`（labeled-loop 切片），latch/test 落点之外的
   arm 出边维持既有拒绝通道（break/return）。

Fallback 互斥划分（design 决策 3）：`loop_fallback` 与 `switch_region` 的 `quoted` 闭包按
walk 顺序先输出 body/arm 自身产生的 Fallback（真实首个失败），合并 quote 只保留此前未被
认领的块（header/branch + 被丢弃结构区域的块），空集不生成 region。顶层
`overlapping_owner` 检查保留不变：真正的局部决策冲突仍整方法 quote；构造失败仍保守
quote，只是账本真实、诊断指向该 walk 真实首个 `FallbackReason`。

## 变体族（任务 1.2，fixture/fixture `Cf13Exits`）

原类 `java -Xverify:all` 输出 `13 87 57 72 83 96 47`（七方法，main 逐行打印）。前后行为：

| 变体 | 基线 `f3452809` | 本片 |
| --- | --- | --- |
| `noJoinNoContinue`（全部 arm 出口即 latch，无 continue） | 整方法 quote（ownership_overlap@9） | 恢复；文本无 `continue;`——goto latch 保持自然落出，非 continue 语句 |
| `twoContinueArms`（两 arm 各含 `if (…) continue;` 且完成于 join） | 整方法 quote | 恢复；两 arm 各呈现一处 `continue;`，join 后 `+= 10` 保留 |
| `defaultWholeContinue`（default 整体 continue） | 恢复（前片 local-join 证书形态） | 逐字一致（回归不变） |
| `armBreaksLoop`（arm 内 labeled break 出 loop） | 恢复（`break loop;`） | 逐字一致（既有 LoopBreak 通道回归） |
| `whileFormContinue`（while 形态，continue 目标 = header 测试块） | 整方法 quote | 恢复；arm 内 `continue;` |
| `outerLabeledContinue`（arm 出边到外层外 loop 的 latch，非直接 latch/test） | 恢复为 `break loop;`（行为等价：外层体尾无语句） | 逐字一致；labeled continue 不在本片范围，等价通道未受影响，负例语义保留 |
| `stringSwitchContinue`（string switch 内 continue） | 整方法 quote（ownership_overlap@BCI 89） | 恢复；与整数 switch 同因（arm 出边分类缺失），顺带闭合，特此区分 |

`outerLabeledContinue` 是"arm 出边到非直接 latch/test 块保持既有拒绝"边界的 javac 可表达
形式：该边不经 continue 分类，仍走既有 break 通道/拒绝通道（boundary 类的 crossCase、
extraEntry、multipleJoins、`SwitchLoopAdjacent.nested` 四负例在 `tests/p3_switch_arm_loop_exits.rs`
中钉住仍拒绝且不再误报 overlap）。

## 恢复输出与三方对照（任务 3.2）

恢复输出（`results/*.recovered.java`，`cargo run -p jarde-cli -- class-source` 产物）：

* W1 全类恢复，无 `@bytecode`；`javac --release 8` 编译通过；`java -Xverify:all` 输出
  `70`/`36`，与原类一致（巡查冻结 `fixture/orig.out`）。
* W2 全类恢复；`noCont` 方法文本与基线逐字一致（`W2.baseline.java` 对应段）；`javac
  --release 8` 编译通过；`java -Xverify:all` 输出 `72`/`9`，与原类一致。
* Cf13Exits 全类恢复（0 个 quote）；`javac --release 8` 编译通过；`java -Xverify:all` 输出
  `13`/`87`/`57`/`72`/`83`/`96`/`47`，与原类逐行一致。

固定 JADX（`continue` 后留不可达 break、不可编译）仍按账本登记作结构参照，不计入行为
等价。

固定 JADX 腿（`testzone/jadx` 安装树，Java-input 无）逐类结构参照，2026-10-01 实测：

* `W1.class`/`W2.class`：该构建以"把 switch 后语句复制进各 arm"消除 `continue`（W1 的
  default 支呈现 `if (i4 <= 4) { i2 = i3 + 3; i3 = i2 + 10; }`，无 continue 语句），可编译，
  `java` 运行 `70`/`36`、`72`/`9`，与原类行为一致——结构改写与 Jarde 的 arm 内
  `continue;` 呈现不同，均行为等价。
* `Cf13Exits.class`：第 66/141/169 行 `continue;` 后留不可达 `break;`，`javac` 报"无法访问
  的语句"三处——账本登记的 JADX 自身反例模式在该类复现，不做行为对照。

## SHA-256

见 `results/sha256.txt`（本目录全部 fixture 与恢复/基线产物）。巡查原 fixture SHA 见
`../../results/fixture-sha256.txt`，未变动。

## 边界与剩余

* labeled continue（需要 label 的双层嵌套）不在本片：`outerLabeledContinue` 经行为等价的
  既有通道呈现，任何需要真实 `continue <label>;` 呈现的形态仍按既有路径拒绝。
* irreducible、跨 loop 出口重排不做。
* `W2.contNoJoin` 的 default arm 呈现为空 then 分支的 `if (cond) { } else { … }`（then 支
  到 latch 的边由 switch 结构表达）：可编译、行为一致；更贴近源码的否定式合并属呈现层
  打磨，未纳入本片。
