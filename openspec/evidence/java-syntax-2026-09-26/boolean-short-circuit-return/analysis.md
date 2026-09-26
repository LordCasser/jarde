# 已证明布尔值链的返回拼写

`BoolValue.java` 用 `javac --release 8 -g:none` 编成 `BoolValue.class`（SHA-256 `dfe43180f7a2cf32c3417c523e11091272f9e7c374dc0674b4094d441b653604`）。本目录保留同一 class 的 `javap.txt`、JADX 1.5.6 类源码 `jadx.java` 和 Jarde `jarde.java`；`Runner.java` 对四组输入分别记录两条带副作用链的调用次数。原 class、JADX 重编类、Jarde 重编类均使用 Java 8 编译和 `-Xverify:all` 执行，输出同 `original-run.txt`。JADX 自行加入 `package defpackage`，其 runner 编译时补同名包声明。

| 方法 | 原源码与 JADX | 当前 Jarde |
| --- | --- | --- |
| `and` | `return a > 0 && b > 0` | `return (arg0 > 0 ? arg1 > 0 ? 1 : 0 : 0) % 2 != 0` |
| `or` | `return a > 0 || b > 0` | `return (arg0 <= 0 ? arg1 > 0 ? 1 : 0 : 1) % 2 != 0` |
| `effectfulAnd` | `return positive(a) && positive(b)` | `return (positive(arg0) ? positive(arg1) ? 1 : 0 : 0) % 2 != 0` |
| `effectfulOr` | `return positive(a) || positive(b)` | `return (!positive(arg0) ? positive(arg1) ? 1 : 0 : 1) % 2 != 0` |

四个方法都没有 `@bytecode`；当前结果语义正确，差距是布尔表达式的构造与拼写。`and` 的两个测试在 BCI 1、5，`or` 在 1、5，两者的 1/0 生产叶都是 BCI 8/12、消费 `ireturn` 在 BCI 13；带调用的版本对应 4/11、14/18、19。`region.rs::short_circuit_value` 已封闭测试图；`build.rs::prove_short_circuit_value` 已验证双叶、Phi、唯一消费与预算；`build_short_circuit_statement` 随后把整图写成数值 `Conditional`，`adapt_return` 才套低位转换。修复应限于该证明后的 `Return` 值构造，不能更改普通语句位 join 的所有权。

JADX 在 `IfRegionMaker.mergeIfInfo` 中按继续走 then/else 选择 `IfCondition.Mode.AND/OR`，是极性与短路顺序的参考；它的区域合并适用范围更宽，不能直接替换 Jarde 的 CFG/SSA 证明。Jarde 还必须保留原始整数进入 `Z` 返回位的低位语义：生产叶改为 `2` 时，不能因为类名或方法返回描述符是 `Z` 就把它当成 `true`。
