## 1. 基线与负例

- [x] 1.1 重放固定 L1/L2/L3：核 fixture SHA、L2 的 BCI 66 残留与 L3 对照；定位标注循环尾段走查的语句核验点（与 17a 启用点的关系）并记录。（[tail-coverage README](../../evidence/java-syntax-2026-10-01/labeled-loop-patrol/tail-coverage/README.md) §走查点定位：fixture SHA 与 `results/fixture-sha256.txt` 一致，L2 残留与 L3 零残留复现；核验点**取证修正**为 `build.rs` 的 builder 逐指令走查（`discarded_evaluations` 的 pop 计划），region 层无 `statement_free*` 检查点；与 17a 的 `guard.rs::discarded_call_pop` 是同判据的两层所有者，guard.rs 未动）
- [x] 1.2 构造并冻结至少三个 verifier 有效变体：break-label 尾段、双层标注嵌套、尾段混合 void/链式调用语句；记录实现前后输出。（T1/T2/T3，javac 23 `--release 8 -g:none`，`java -Xverify:all` 通过且运行输出逐行确定；前后文本与 SHA 在 [tail-coverage/results](../../evidence/java-syntax-2026-10-01/labeled-loop-patrol/tail-coverage/results/)：残留 BCI 82/94+116/60 → 零残留，其余逐字节不变）

## 2. 尾段覆盖与标签拼写

- [x] 2.1 尾段走查接入 discarded-call 判据（design 决策 1）；L2 零 `@bytecode` 残留、行为逐字一致；17a 负例与 TWR 路径零变化。（`build.rs::discarded_evaluations`：把形状 (a) 独有的 `next` 前置守卫移入形状 (a)，形状 (b) 判据 `call_result_is_discarded` 一字未改、无第二份拷贝；L2 输出除标签与 BCI 66 残留移除外逐字节一致，重编运行 `00,10,` 逐行一致；TWR/17a 家族测试全绿，`guard.rs` 与四个 monitor/finally `statement_free` 调用点一字未动；回归 `tests/p3_labeled_loop_tail.rs`）
- [x] 2.2 标签源码式拼写（design 决策 2）；全仓 `jarde_loop` 期望同步更新并注明；L1/L3 除标签外逐字不变。（`Builder::loop_label`：语句写出序首次认领取 `loop`，其后 `loop2`…，随 `FinallyCheckpoint` 回滚；T2 钉死内 `loop`/外 `loop2`；`p3_loop_transfers`/`p3_for_add_store`/`p3_switch_loop_exits` 的 `jarde_loop_{4,8,5}` 期望同步为 `loop` 并在文件头注明改名缘由；L1 与冻结基线的 diff 仅标签拼写，L3 零 diff）

## 3. 验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿、fmt、CI 完整 30 项 allowlist clippy、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。（最终状态记录跑 **2739 通过/0 失败**、270 目标；clippy 30 项 `-A`+`-D warnings` 0 警告；strict 229/229；diff check 干净；整仓跑期间两次与负载相关的既有 flake（`bulk_recovery_delivery` 4-worker 预算断言、`p4_plugins` 的 `elapsed_millis` 0/1）隔离重跑全绿后按重跑判定规则排除因果并登记为独立观察项，见 [tail-coverage README §门禁](../../evidence/java-syntax-2026-10-01/labeled-loop-patrol/tail-coverage/README.md)）
- [x] 3.2 L1/L2/L3 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。（六类三腿逐路径：原 class==Jarde 全部一致（L1 `a3940f06…`、L2 `3e87c820…`、L3 `d5471484…`、T1 `13f3d359…`、T3 `90d88888…`）；T2 的 JADX 腿 `f5495d82…` 自身偏差——双层标注 `continue outer` 被改写为内层 break 而改变行为，原 class 为行为基准，Jarde 与原类逐行一致；run SHA 与腿输出见 [results/sha256.txt](../../evidence/java-syntax-2026-10-01/labeled-loop-patrol/tail-coverage/results/sha256.txt) 与 `results/run/`；测试内 compile+run 腿钉进回归）
- [x] 3.3 root 独立复核泛化面、标签规则与三方行为，更新控制流账本与巡查记录。（root 于合并主线 bfe8faf1 复核：L2 输出 `loop: for…` + `continue loop;` + 尾段链式 append 零 @bytecode 残留、重编行为逐字一致（`00,10,`）、L1/L3 除标签外不变、全仓 2739/0、fmt/openspec 229/229。落点修正复核认可：实据点为 build.rs `discarded_evaluations` 计划（`next` 前置守卫移入形状 (a)），与 17a guard.rs 判据同语义不同所有者层，无复制；标签按写出序 `loop`/`loopN`。JADX 对双层标注 continue 的行为改写登记为该工具自身反例。）
