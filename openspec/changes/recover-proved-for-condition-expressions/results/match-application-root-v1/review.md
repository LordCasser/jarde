# 私有候选审查

候选只改变 `for_header_candidate` 现有 `test` 指令 match：在原 `Push`、`Arithmetic`、`Comparison` 之外，允许 `Negate`、`NumericComparison`、`Invoke`、`FieldAccess::Read`、`ArrayLength`、`ArrayLoad` 与 `ArrayElementLoad`。Load 角色分支保持原样，仍单独标记计数变量读取并记录其它局部不变量。

## 准入前提与效果位置

唯一生产调用路径是 `header_tested_loop`：它先对同一 header/test BCI 调用 `test_is_pure(header, test_bci, true)`，成功后才调用 `prove_for_header`，后者进入候选。该先行检查要求这些非局部 producer 属于 branch 条件值；数组长度与元素读还保留原单读者约束。调用和字段读取可以有副作用或抛异常，但它们必须是条件表达式的 producer，未被条件消费的调用/读取仍由 `test_is_pure` 拒绝。这里没有增加一般 purity 声明。

候选只决定这个已接受的 header 是否也满足计数循环的局部角色证明。Builder 仍以原 branch BCI 调用 `test_expr`，并在现有位置写条件；初始化语句仍由 `init_bci` 消费，更新仍由 `update_bci` 消费。没有缓存或搬动条件 producer，因此每轮检查的调用次数、stack operand 顺序和异常位置保持字节码对应的头部求值位置。所有其它 ForHeader 条件保持在 while 中。

## 保持的拒绝边界

更新指令仍要求既有尾部形状，且其写入必须是计数局部；preheader 仍须唯一且初始化 store 位于末尾，初始化 stack producer 仍须单消费者；phi 仍须精确两个输入并由 init/update 提供。唯一 latch、完整回边、循环内 induction/invariant 局部写、循环外访问检查均未变。match 没有接受 `Store`、`Increment` 或 `Copy`，Builder、条件证明、CFG、计费与停止传播没有改动。`ArrayLength` 和元素读继续受此前消费检查约束，避免表达式树重复执行抛异常读取。

## 规范化差异证据

对当前生产 `region.rs` 与候选执行 `diff -u`，差异恰有两个局部 hunk：一个在现有 `for_header_candidate` test 指令扫描前增加两行 caller 契约注释；另一个只扩展该 match 的 operation alternatives。`Operation::Load` 两个角色分支、后续 `reads_induction` 检查、ForHeader 构造及其后 CFG/初始化/phi/invariant/external-access 检查逐字保留。完整规范化 unified diff 见 `region.rs.patch`。

未运行构建或测试；本任务仅交付待父任务接受的静态私有候选。`pins.json` 中记录的源码、候选和 patch SHA-256 均从当前字节读取计算。生产 `region.rs` SHA 与 root 给定基线相同。
