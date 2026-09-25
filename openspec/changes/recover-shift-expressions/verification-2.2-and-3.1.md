# 移位表达式：2.1–2.2 与整类执行 3.1（2026-09-24）

六条 `ishl/lshl/ishr/lshr/iushr/lushr` 指令现在解码为独立 `ShiftOp`，并沿现有 SSA 值呈现路径构造三个二元 AST 运算符。类型由左操作数单独做一元整数提升决定，右操作数只须是可整数提升的位移距离；build 再核对原 opcode 的 `int`/`long` 结果宽度。布尔、浮点或没有可写 Java 类型的输入保持引用。emitter 在原有统一优先级规则中加入移位层，并保持同级右子树的括号。未增加新的 CFG、IR 或类型推断机制。

Root 使用最终 CLI `/tmp/jarde-shift-ops-replay/jarde-cli`（SHA-256 `9467c73083d721a51abae84455175980bb7a2f2b38ed29c12c7f073d3672f6a9`）独立对三份冻结输入执行 `class-source`，均为 exit 0、`@bytecode` 计数 0，**未修改生成文本**地用 `javac --release 8` 编译完整类，再用 `java -Xverify:all` 执行：

| 输入 | 原 class SHA-256 | 原/Jarde 行数与输出 SHA-256 | 结果 |
| --- | --- | --- | --- |
| `ShiftSlice` 十方法小切片 | `1036af7d36484406373dc63259fb2bf3232cd5fcf289b22b488815eac90a438a` | 56 行，`63bdb0f4981100c8743d5b6c09e34d53dd3a12e54c84bbbf64705a152afbf2f7` | 原/JADX/Jarde 全部相同；六 opcode、short/char、同级嵌套、helper 两侧调用均在内 |
| `ShiftCore` root-core | `db5a3053fb7b38ddf22cff1e9ec93048d41ab29e94d5809c0f1f611f8ac86bba` | 723 行，`f34b05eb6a7ec009b2a6ce621f0479d5a9eb56e4c40ebdf350572408213a317d` | 原/JADX/Jarde 全部相同；原基线 Jarde 有 29 处引用 |
| `ShiftWidth` 宽度边界 | `0ead0ef6d8d26e594f9498e716630b4697bea11d78198d021c892e9d5f33e43e` | 3 行，`d92e366418388680cf24d4e436c05c905665ab05fdaa6e9d3db81278d5e7bfc4` | 原/JADX/Jarde 全部相同，`wide(1)=4294967296`、`narrow(1)=1` |

`ShiftWidth` 的 Jarde 文本分别写出 `return (long) arg0 << 32;` 和 `return (long) (arg0 << 32);`，宽度转换及其求值位置均保留。`ShiftSlice` 的 `nested` 输出 `arg0 << arg1 >>> arg2`，与 javac 的左结合一致；`callTarget` 仍按原顺序调用两侧 helper。原始、JADX 的 RED 基线和原/Jarde 的本次执行分别留在 [小切片](../../evidence/java-syntax-2026-09-24/shift-root-core-slice/analysis.md)、[原 root-core](../../evidence/java-syntax-2026-09-22/shifts/analysis.md)及[宽度边界](../../evidence/java-syntax-2026-09-24/shift-width-cast/analysis.md)。`ShiftAudit` 的位运算组合与 200 项 long 距离场景属于相邻边界，不计本项 3.1。

新增 `tests/p3_shift_expressions.rs` 的三项完整类测试通过；`jarde-java --lib` 138/138，受影响的位运算、显式转换、调用与延迟值顺序测试通过。代理的较宽测试另见两个无移位输入的既存失败：`p3_numeric_comparison::call_operands_keep_order_and_the_negative_boolean_merge_stays_quoted` 与 `class_initializer_candidates::initializer_sidecar_is_limited_to_non_annotation_interfaces`；它们分别留在数值比较和初始化器已有债务，不混入移位实现。`cargo fmt --all --check`、`git diff --check`、`openspec validate recover-shift-expressions --strict` 通过。

2.3 的新 [求值位置证据](../../evidence/java-syntax-2026-09-24/shift-effect-order/analysis.md) 用同一冻结 class 检查两侧调用、任一侧抛错、跨独立语句及旧局部左值：原/Jarde 三行完全一致，完整 Jarde 类可重编且无拒绝标记。JADX 1.5.6 在此合法输入上把移位后的 `getstatic events` 前移，生成能编译但三行执行结果全错的 Java；Jarde 保留临时结果及字段读取次序。这是以 SSA 求值位置优于 JADX 的具体案例，不靠额外移位专用机制。

1.2 的 JVM 合法负例已另行验收；2.4 的预算/来源及 3.2–3.3 总验收仍未关闭。
