# switch 臂与外层循环出口的归属

冻结输入为本目录的 `SwitchLoopExits.java`、`SwitchLoopExitsRunner.java`、`SwitchLoopExits.class`。用 `javac --release 8 -g:none` 编译；class 是 Java 8（major 52），SHA-256 为 `279bb8be06bd8257c91d4c2e2a6807dc20d57ef190a8da49d7014ec1b372e03c`。`start` 与 `stop` 让普通 case、switch 自身 `break`、外层循环 `break` 都可执行，比 `tests/fixtures/p3-loop-transfers/SwitchLoopTransfer.java` 的固定 `i=0` 更能验收行为。

| 输入 `start:limit:stop` | 原 class `java -Xverify:all` |
| --- | ---: |
| `0:3:false` | 3 |
| `0:3:true` | 0 |
| `1:3:false` | 2 |
| `2:3:false` | 0 |
| `0:1:false` | 1 |
| `-1:3:false` | 0 |
| `3:3:false` | 0 |

同一 class 交给安装的 JADX 1.5.6，完整输出冻结为 `jadx.java.txt`。仅去掉它给默认包附加的 `package defpackage;` 后执行 `javac --release 8`，第 17 行第二条连续 `break;` 报“无法访问的语句”，退出码 1，不能作运行对照。Jarde 实施前 CLI SHA-256 为 `725082f7183ab6ed6451fb2a6facd89986212d4e25254f7b64b9d9f699cb70a1`；同输入 `class-source --policy single-class --release 8 --evidence all` 的 `run(IIZ)I` 为 explanation only，诊断先给 `jre_region_switch_arms_overlap`，另给 `jre_region_uncovered_blocks`，没有误发可编译却变更语义的 `switch`。

关键 CFG：BCI 11 的 `lookupswitch` 指向 case 0 的 36、case 1 的 49、default 的 55。case 0 的 BCI 40 和 default 的 BCI 55 跳到外层循环出口 64；两个正常完成的 case 在 BCI 58 汇合，先执行 `iinc 3,1`，再回到循环测试 5。64 是该 switch 在整方法 CFG 中的后支配点，但不是 Java `switch` 的正常完成汇合；58 只后支配留在循环内的臂。把 64 当 switch join 会把循环出口误收成臂汇合，把 58 的循环尾放进多个臂，正与 Jarde 的 overlap 诊断相符。

本地 JADX `2fb1b1638694` 的 `SwitchRegionMaker.calcSwitchOut` 用 case 的 dominance frontier 选 `out`，在多出口时再看循环边和 postdom；`insertBreaksForCase` 对可能离开 case 的嵌套 region 与整个 case 都尝试追加合成 `break`，后续 `SwitchBreakVisitor` 才清理。冻结输出说明这条链在本形状没有清理到可编译状态；不以“JADX 成功产生文本”作为正确性依据。可借鉴它对 case 出口和 loop exit 分开的调查顺序，但不能复用无目标身份的合成 `break` 或仅凭全方法后支配点认领 switch join。

JADX 自己的 `jadx-core/src/test/java/jadx/tests/integration/switches/TestSwitchBreak.java` 也在带标签的 switch 内循环退出旁留有 `TODO finish break with label from switch`，断言的是改写为 `return s + "+";`。这说明它的现有测试没有把保留循环目标的 `break` 当作已完成能力；此处既不能照搬其结构，也不能把重编失败归咎于测试 Runner。

Jarde 的实现落点仍是现有 `region.rs` 的 `Frame.loop_targets`、`switch_region`、分支臂走访与 `Region::LoopBreak`，以及现有 AST/Emitter。先将外层循环出口边视为带目标身份的终止边，只对仍在该循环内的臂证明一个局部 switch 汇合；case 0 的条件臂必须保留 `break outer`，case 1 与 case 0 的正常路径必须先执行 BCI 58，default 必须直接退出循环。发射时若一个 switch 臂已由 `break`、`continue`、`return`、`throw` 结束，不得再自动追加不可达的 switch `break`；含 `if` 的臂需按两条路径分别判断能否正常完成。任一归属、入边、作用域、或 Java 可达性不能证明时继续局部引用，不推测源码标签。无需新增全局 pass 或独立 CFG；这是当前区域边界和臂完成性的证明缺口。

验收至少包括：冻结源码/class 字节一致；Jarde 完整类无 `@bytecode` 且 Java 8 重编成功；原 class/Jarde 的 7 行 `-Xverify:all` 全同；switch 臂的 BCI 40/55 都映射到带循环目标的 `break`，BCI 58 只在 switch 后、循环体内出现一次；相邻普通 switch 穿透、嵌套 switch 与循环 continue 的正反例保持；低预算或取消不能发布半个 switch。JADX 在这一输入无法重编，必须单独记录，不能以三方运行相同作为通过条件。

## 实施后的独立验收（2026-09-24）

前文 Jarde SHA 与拒绝诊断是实施**前**的冻结基线。Agent 在现有 `region.rs` 中仅当循环内 switch 有唯一局部汇合时，才让其正常臂停在该点、让精确循环出口仍写成 `Region::LoopBreak`；`emit.rs` 仅给 Java 控制可正常完成的 case 补 `break`。root 用实施后 CLI SHA-256 `076c56649e04a4c84b8c10b03b13157cda8aa19e8c384f89997002f3e6e8ae88` 独立从源码重建冻结 class，SHA 完全相同；Jarde 整类无 `@bytecode` 且通过 `javac --release 8`，原 class/Jarde 各自 `-Xverify:all` 的七行逐字相同，就是上表的七行。输出在 `case 0` 的条件真臂和 `default` 写 `break jarde_loop_5;`，BCI 58 的 `local3 = local3 + 1;` 只出现在 switch 之后一次；source map 将 BCI 40/55 映射到各自循环 break、58 映射到该更新。

独立相邻反例 `tests/fixtures/p3-switch-loop-exits/SwitchLoopAdjacent.java` 同时含嵌套 switch 与循环 `continue`，外层 switch 仅一条正常路径，不能证明唯一局部汇合。初版投影曾生成可编译但错误的源码，收紧后现为 explanation only；`tests/p3_switch_loop_exits.rs` 钉住拒绝，`tests/p3_loop_transfers.rs` 将旧的 `SwitchLoopTransfer` 从拒绝边界提升为可恢复正例。`jarde-java` 121 项库测试、相关定向测试、`cargo fmt --all -- --check`、`git diff --check` 通过。全 workspace 测试在共享树 `bulk_recovery_handover` 的 `test-support` 特性配置处编译失败；Clippy 停在七条与本子集无关的既有警告。全量 all-features 编译因独立 target 占用快速增长中止，随后两次 `cargo clean` 合计回收约 10.6 GiB。3.1 的循环内 try/catch 与不可信转移边尚未验收，任务仍未勾选。
