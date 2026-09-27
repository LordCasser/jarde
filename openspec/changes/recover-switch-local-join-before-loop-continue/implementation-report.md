# 实现与验收记录

## 控制流与归属

在当前循环的 `continue_target` 恰好是 switch 全方法 post-dominator 时，`switch_loop_join` 对各 case 做有界走访。局部 join 必须在循环作用域内、受 switch 支配、没有外部前驱，并由至少两个正常 case 到达；至少一个独立 case 必须经自身单后继 `Transfer` 精确到达当前循环更新块。跨 case、嵌套 switch、混合 join/continue 路径或多个候选 join 均不授予此证书。原有 loop-break 路径沿用既有规则。

只有获得该证书的 switch arm 的私有 `Frame.switch_continue` 能把到当前循环更新块的转移写成 `LoopContinue`。CF-13 的 BCI 49 保留为 `continue` 来源；BCI 55 的共享语句留在 switch 后，BCI 58 的更新留给外层 `for`。完整恢复结果经过 `region.rs` 的 `overlapping_owner` 检查，若 completed Region tree 重复认领物理 block，会整体引用；本例 `walk(I)I` 已通过该检查。定向测试核对 BCI 9/12/49/55/58 对应的 source-map 文本，并确认共享语句与更新各出现一次，且没有 `continue; break;`。

## 负例与中止

新增 Java 8 固定边界类包含普通 loop 尾、跨 case、额外入口和两个候选 join。普通尾部仍完整恢复且没有虚构 `continue`；后三者保持引用。既有 `SwitchLoopAdjacent.nested` 继续引用。原有和 CF-13 class 均通过低分析预算、预取消请求的“不发布完整类”断言。

## Java 8 对照

原始输入和 Jarde 修后完整源码分别通过 `javac --release 8 -g -Xlint:-options`；固定原 class、重编原输入、重编 Jarde 源码的 runner 均以 `java -Xverify:all` 运行，输出精确为 `38`。修后完整源码 SHA-256：`31d8374396c39b604bdc09923c6fa556b9823f72d3a1e0b68c15b3f48b363b2c`。固定 JADX 源码按相同 Java 8 编译命令在第 16 行因 `continue;` 后的不可达 `break;` 失败，此失败是既有对照事实。

## 检查

- `cargo test --test p3_switch_loop_exits --test p3_loop_transfers --test p3_switch_forward_join --test p3_switch_fallthrough --test p3_loop_test_values`：19/19 通过。
- 新边界类以 `javac --release 8 -g -Xlint:-options` 编成测试夹具；其 class SHA-256 为 `052fb805b7bacaa8695d892a0ac3b80d49a82e1471f1644745f64b23a89d3f3b`。
- `cargo test -p jarde-java --lib region::tests`：4/4 通过。
- `cargo check -p jarde-java -p jarde`、`cargo fmt --all -- --check`、`git diff --check` 均通过。
- `openspec validate recover-switch-local-join-before-loop-continue --strict` 通过。
- 隔离 target `/tmp/jarde-cf13-impl-target` 已执行 `cargo clean`，清理 1450 个文件、1.7 GiB。
