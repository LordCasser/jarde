# 实施阶段验证

本文件是实施代理阶段记录；root 后续独立验收见 [verification-root.md](verification-root.md)。任务 1.3 的 fixture、verifier-valid 控制和本目录八个语料指纹条目已经落地，root 核对本目录应登记文件 8/8 与路径/长度一致后已勾选。当前共享工作树的全局指纹清单还有 121 个其他并行 fixture 未登记，已作为独立债务记录，不影响本任务的局部合同。

- 正例 `ArrayPostfixElement.class` 和 `iinc +2` 控制按冻结字节复制。不同槽控制以 `javac --release 8 -g:none -Xlint:-options` 编译后将唯一 `iinc 0,1` 补丁为 `iinc 1,1`；`java -Xverify:all` 原运行输出 `[1, 0, 0]`，而误写为 `a++` 会让第三元素为 2。八个新增语料指纹条目的 BLAKE3 与字节数均逐项核验。
- `CARGO_TARGET_DIR=/tmp/jarde-local-postfix-agent cargo test --test p3_local_postfix_array_elements -- --include-ignored`：5/5 通过。正例恢复正文含一次 `arg0++`，没有独立 `iinc` 赋值，所有真实 BCI 均有 source map；完整类以 Java 8 重编后在 `-Xverify:all` 下五行输出与原 class 一致，包括 `Integer.MAX_VALUE` 溢出。`+2` 与不同槽控制没有投影为 `++` 或部分数组字面量。低 IR 预算和预取消无部分 postfix 提交。
- 相邻回归：一维数组初始化 4/4、布尔数组 JDK 对照 2/2、求值位置 10/10、普通局部重写 7/7、多维数组及 JDK 对照 4/4、部分维度及 JDK 对照 3/3、既有后置字段/数组值及 JDK 对照 6/6、短路异常链 1/1，全部通过。

Root 已独立构建 CLI、复核三方运行、来源与拒绝边界，并完成 3.1、3.2；全局指纹清单漂移另案处理。
