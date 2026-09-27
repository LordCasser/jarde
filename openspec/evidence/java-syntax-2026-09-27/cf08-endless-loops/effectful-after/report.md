# CF-08：带效果双出口验收

输入源码、Runner 与固定 class 的 SHA-256 分别为 `1d8d906d5bb3bf10d7220d46af6453ef2baaa096a228f4d1a7a5ad3d4f857eaf`、`ba71a9f1a2b557e5dc63f9163c446de60076273ce3d804fd4ca5d6ec26cbec0d`、`2e27cffb9361cbd78a689b404457b68d6bc31f1739362659ce71c0e276c0bae8`。固定 JADX checkout 为 `2fb1b16386941660fda07e9017285aec40fcb37f`，重放脚本还校验了七个固定源码 pin；见 [summary.json](summary.json)。

## 1.1 可行性门槛

实际 Jarde SSA 的 BCI 35 `Local(1)` 是 `ValueId(4)` φ，两个输入为 BCI 13 `istore_1` 的 `ValueId(16)` 与 BCI 20 `istore_1` 的 `ValueId(20)`。φ 没有被替换，仅在 BCI 35 `iload_1` 使用一次；效果臂的写入值仅被这个 φ 使用，体内写入值还在 BCI 21 被比较读取。新测试从 `MethodIr::ssa()` 直接断言这些定义、输入与消费。

在拒绝式 walker 探针中，体内首次进入 BCI 2 后得到 `If@5`：越界臂持有 BCI 8 和 `LoopBreak@14`，继续臂得到 `If@23`，其中命中臂持有 BCI 26 和 `LoopBreak@26`，未命中臂持有 BCI 29；BCI 32 回边只停止迭代，不再次领取 BCI 2。唯一后续为 BCI 35。探针本身没有发布源码，正式实现用相同的有限走访和区域归属检查。

## 证书和边界

证书仅接受三个自然循环块（头、体内比较、单一 latch）加两个独占出口块；头部为纯比较，效果块仅有常量、调用、同一局部写入、纯 goto，体内出口仅有纯 goto，latch 为自增与纯 goto。检查全部候选块的 Code/SSA 指令对应、canonical 正常入出边、无 handler/异常或子程序边、共同后继唯一的两个入边，以及上述 φ 和唯一呈现消费。扫描先按边、指令和 φ 收费并检查取消，再扩展 body scope。未闭合时沿既有安全拒绝路径；负例的额外调用、不同后继、额外 latch、handler 均未发布 Endless。预算耗尽和预取消都不留下部分源码或 source map。

Endless 的 header 由体内 `If` 唯一拥有；循环语句是派生锚点。物理 BCI 5、8/10/13/14、17/20/23/26、29/32、35/36/39/40 均有来源。BCI 14、26 对应各自 break，BCI 32 的纯回边由循环派生来源保留；BCI 35 在循环外只消费一次。

## 完整源码与回归

[重放脚本](../replay-effectful.py) 用 `javac --release 8 -g:none` 重编原始、固定 JADX 和 Jarde **完整类源码**，再以 `java -Xverify:all` 运行 Runner。三者对空数组、命中 3、未命中至越界均为 `8:1 / 3:0 / 8:1`，Jarde `pick` 无 `@bytecode`；源码、编译及运行日志均在本目录。纯双网关、普通 while/do-while 与单出口终止循环的定向测试通过。单独重放 `NotIndexedLoop` 仍在外层分支/局部汇合处安全拒绝，完整 Jarde 源码缺少返回语句；本变更没有覆盖该形态。

定向 Rust 测试：`p3_effectful_exits` 3/3、`p3_loop_exit_gateways` 4/4、`p3_loop_terminal_return` 3/3、`p3_java_recovery` 37/37。另有 `cargo fmt --all -- --check`、`cargo check --workspace --locked`、`git diff --check` 与 `openspec validate recover-effectful-dual-loop-exits --strict` 通过。
