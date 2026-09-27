# 固定 Test12 `runTest` 主线独立验收

主线 `4c6301ad` 合入实现后，root 用独立 `/private/tmp/jarde-switch-root-target` 新建 CLI，复放 `replay.sh` 到 `/private/tmp/jarde-switch-root-replay`。固定 `TestTryCatchFinally12$TestCls.class` SHA-256 为 `d9b9cb676203d943ee3cf97d66e62dda2a637125d7f737be1e22ca275c209185`；pinned JADX checkout 为 `2fb1b16386941660fda07e9017285aec40fcb37f`。固定类与同 BCI/opcode/异常表的最小完整类，各自原 class、JADX、实时 Jarde 全源码均通过 `javac --release 8` 和九路径 `java -Xverify:all`；两套三方运行输出逐字一致。root 的 `results.txt`、方法形状、六份运行轨迹以及两份 Jarde 源码与提交的 `after/` 冻结副本逐字一致。

root 审阅了 Guard 的实例字段完整赋值证明：只把栈闭合的 `putfield` 前缀排出 TWR 假候选，不改变局部 `Store`/真实 TWR refusal。Region 仅在异常行 exclusive end 有单条无效果 transfer、唯一词法后继、所有正常前驱由当前 switch body 拥有、且无竞争异常/子程序入口时续走；`split` 将 BCI 61 作为 Try 后独立 sibling，而不是并入保护范围。Builder 只为直线 case arm 的末条、直达当前 switch join 的消去 `goto` 加 derived 来源。固定方法的 `Try→Switch→Catch`、全部物理 BCI 来源、五个 Try 块加 BCI 61 与 79 的唯一 owner 由产品测试逐项断言；61、45、53 均有派生来源。

三个部分 case 保护 class 与真实 TWR class 的 SHA-256 清单通过 root 重算；均为 Java 8 编译、`-Xverify:all` 有效，未被误提升为完整外层 switch catch。固定布局中不能直接生成 verifier 有效的额外边，受控 canonical edge 测试覆盖竞争入口、额外/重复出口和异常边。独立 `cargo test -p jarde-java --tests --locked`、`cargo check --workspace --locked`、`cargo fmt --all -- --check`、OpenSpec strict 与 diff check 全部通过。专用 Cargo target 随后清理。

验收范围是固定 `runTest` 与相同证书形状；Test12 的 `test1/2/3` 已有单独验收。完整 InnerClasses family 的其它投影、Test13 五行多段 finally 和 `FinallyOnce` 的其它保护形态不因此算作 CF-16 全单元追平。
