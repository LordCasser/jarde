# 窄整数返回 fixture

`v8/NarrowIntegerReturns.class` 为 1,288 字节 Java 8 输入。源码将返回都编译为 `int`，再用
常量池描述符补丁把部分方法改为 B/C/S；Code 属性保持不变。它覆盖窄局部回读、直接返回、
字段前后自增和同步返回。source-only runner 检查越界值、字段更新及空锁触发的异常。

`actual-stack-join/ActualStackJoin.java` 使用 Java 8 语法，先由受控补丁改写为 major 49，
让 switch 各臂在操作数栈上汇合到同一 `ireturn`；随后每个版本只改一个 B/C/S 返回描述符。
三份永久 379 字节类位于 `actual-stack-join/v8/{byte,char,short}/`。复现过程使用
`regenerate_task12_fixtures.py` 调用 OpenSpec evidence 中已审核的 `patch_stack_join.py` 和
`patch_descriptors.py`；每个变体的 descriptor JSON 与 JVM 预期输出一并保存。

`v8/boolean-boundaries/BooleanReturnBoundaries.class` 保留 Z 参数并分别将方法返回描述符改为
B/C/S；另把 int identity 方法的返回描述符改为 Z。调用者
`BooleanRawTwoCaller.class` 在常量 BCI 0 把编译期的 `iconst_0` 改为 `iconst_2`，再通过
`(Z)B/C/S` 调用目标方法。`patch_return_boundaries.py` 可复现描述符和调用者改动，JSON 记录
输入输出 hash 和精确改动。`ReturnBoundaryRunner.java` 把 char 结果转成十进制整数后打印，
避免不可见字符成为测试 oracle。

运行 `cargo test --test p3_narrow_return_boundaries` 会用 `java -Xverify:all` 实际执行两组
边界输入，检查共享 join 的 BCI 映射、boolean 操作数拒绝来源、raw 2 返回值，以及 Z 返回的
最低位语义。
