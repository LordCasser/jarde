# Root 独立验收（3.1–3.2）

Root 在 `6360f94d` 的实现上审读 `region.rs` 的头/闩锁选择、天然环外出口桥和 `build.rs` 的空臂来源。头块有比较并不直接决定头测：只有两条真实后继留在迭代中，或一条经独占普通桥到同一闩锁出口时才优先尝试体内分支。桥要求唯一 Normal 入/出边、源块比较、桥块仅一条 `Transfer`，并在循环覆盖核账中单独认领；任一额外/异常/Call 边不获此证明。闩锁体前缀在条件前执行，提前 `break` 不执行后续闩锁调用。`continue` 的无语句 `goto` 作为所属 `if` 的派生来源，`break` 的桥 `goto` 锚在转移语句，均不伪造物理 BCI。

Root 独立构建当前 `jarde-cli`，借用已冻结的三方审计驱动，但将输入、JADX、Jarde 产物与日志都写入隔离临时目录，未覆盖基线证据。`javac --release 8 -g:none` 原样编译各源码；原 class、JADX 1.5.6 原样源码和当前 Jarde 完整类均通过 Java 8 编译、`java -Xverify:all`，运行结果逐字节相同，Jarde `@bytecode` 引用数均为 0：

| 样本 | 输出行数 | 原/JADX/Jarde 输出 SHA-256 |
| --- | ---: | --- |
| `DoWhilePositive` | 4 | `28e0e52277783b453f2a615767eef04cc213afe30f1e17a74027293651e4b6bc` |
| `DoWhileCore` | 7 | `5f3fa46c4a5fd6eebc38027bb8926e5e8500869574c35e4fbef01df2c057bee5` |
| `DoWhileAudit` | 9 | `8783e96b04133881325205d57b52b6538cd8a14cd13d48ec4c70bcd264a1257a` |

Root 独立运行 `cargo test --locked --all-features --test p3_loop_boolean_do`（当时 7/7；代理随后加入闩锁调用早退对照并重跑为 8/8）和 `cargo test --locked --all-features -p jarde-java`（全套通过）。代理在隔离 target 下另跑相邻的 `p3_loop_boolean_exit` 3/3、`p3_loop_test_values` 4/4、`p3_loop_transfers` 5/5、`p3_loop_try_handler_entry` 6/6、`p3_switch_loop_exits` 3/3。永久负例继续拒绝嵌套 `switch` 的无标记外层 break、闩锁链第二入口和含独立效果的混合条件；低预算与预取消不发布未证循环。源码默认/all 一致，核心 BCI 7、10、13、21、24、29、32 在两种证据选择下均可追溯。目标文件格式、`git diff --check` 和 OpenSpec strict 已通过。

本项只证一个 do-while 的体内转移；多层循环、跨层带标记转移和 `switch` 臂截获的无标记 break 仍由 `present-proved-java-structure` 的相邻任务处理，不能沿用这里的单层桥证明。Reader 全量语料计数目前因另批已加入的 class 夹具与冻结 census 不符，已作为独立维护切片核对，不修改本项 Region 语义；3.3 待 census/fingerprint 和全仓最终门禁复核后完成。Root 验收用的隔离 Cargo target 已清理 4.1 GiB。
