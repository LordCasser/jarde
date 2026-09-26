# CF-01：布尔 eager 运算不能无条件短路化

本次只验收 CF-01 的一个反例，不据此判定整个 CF-01 已追平。输入是 [EagerBoolean.java](EagerBoolean.java)，用 `javac --release 8 -g:none` 编译；[物理指令](original-javap.txt) 中两个条件分别含 `iand` 和 `ior`。右操作数 `rhs()` 每次把 `calls` 加一，因此即使左侧已决定结果，也必须执行。固定的 class SHA-256、JADX commit 和三组运行结果见 [results.json](results.json)。[replay.py](replay.py) 在临时目录构建、完整源码重编并以 `java -Xverify:all` 运行三组，临时 Cargo target 随回放清理。

| 来源 | 条件表达式 | Java 8 重编 | 运行输出 |
| --- | --- | --- | --- |
| 原始 | `left & rhs()`；`left | rhs()` | 通过 | `1`、`102` |
| [JADX](jadx-EagerBoolean.java.txt) | `z && rhs()`；`z || rhs()` | 通过 | `0`、`100` |
| [Jarde](jarde-EagerBoolean.java.txt) | `arg0 & rhs()`；`arg0 | rhs()` | 通过 | `1`、`102` |

JADX `IfCondition.simplifyCmpOp` 在 `jadx-core/src/main/java/jadx/core/dex/regions/conditions/IfCondition.java` 的 `ARITH` 分支只检查布尔类型和 `AND/OR` 操作符，就将两次求值的位运算改成 `IfCondition.Mode.AND/OR`。此处没有检查右操作数的调用副作用，也没有证明 CFG 本来短路。这个测试直接证明该改写在一条完整 Java 8 程序上改变行为。Jarde 的 `BinaryOp::BitwiseAnd/BitwiseOr` 与 `LogicalAnd/LogicalOr` 分离，当前输出保留执行次数；此子形态不需要新增恢复机制。

CF-01 余下的真正短路 CFG、否定组合和多分支条件还需依固定账本逐项重放。后续实现不得以匹配 JADX 文本为目标把 eager `&`/`|` 改成短路运算；只有物理控制流确实跳过右侧求值时，才可输出 `&&`/`||`。此审计不修改 Jarde 代码。
