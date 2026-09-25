# Primitive conversion opcode audit

2026-09-23，使用未重建的 `target/debug/jarde-cli`，SHA-256 前后均为 `7527b03abc1e487121d204672b52043ca5f3aa1148c6d65b136e36f2bbe1f4aa`。审计只写本目录和 `/tmp/jarde-numeric-conversions*`，没有修改生产代码、永久 fixture、Rust 测试、Cargo、总表或 OpenSpec。

`run_audit.py` 保存完整可重放流程。它用 `javac --release 8 -g:none` 编译三个互不干扰的被测主类、一个 overload helper 和一个共享 runner；原 class、JADX 和 jarde 每次都以完整 runner 为执行入口。JADX 对三个主类一次性反编译，避免 default-package 被放入 `defpackage` 后 runner 只编译单组而产生假失败。源码、原始输出、JADX/jarde 完整输出、`javap -c -v`、class hash 和每阶段日志均保存在本目录。

## 覆盖和 class 属性

| 主类 | 覆盖的实际 opcode | Code 数 | class 字节数 | jarde quote 数 |
| --- | --- | ---: | ---: | ---: |
| `ConversionIntegers` | `i2l`×1、`i2f`×1、`i2d`×1、`i2b`×2、`i2c`×2、`i2s`×2 | 10 | 615 | 18 |
| `ConversionLongFloat` | `l2i`×2、`l2f`×2、`l2d`×1、`f2i`×1、`f2l`×1、`f2d`×2 | 10 | 622 | 18 |
| `ConversionDouble` | `d2i`×2、`d2l`×1、`d2f`×1 | 5 | 358 | 8 |

三组合计覆盖 Java primitive conversion 的 15 个 opcode 家族：`i2l/i2f/i2d/l2i/l2f/l2d/f2i/f2l/f2d/d2i/d2l/d2f/i2b/i2c/i2s`。次数由保存的 `javap` 中真正的 instruction 行统计，不把 constant-pool 名称或方法名当作 opcode。class SHA-256 分别见三个主类目录的 `original-class-sha256.txt`。

被测 class 没有把浮点常量、位运算或 shift 放入方法体。runner 用 `Integer.MIN_VALUE`/`MAX_VALUE`、`Long.MIN_VALUE`/`MAX_VALUE`、负数和窄化边界，以及 `Float`/`Double` 的负无穷、最大值、负零、正零、最小值、NaN 和正无穷；浮点结果统一用 `floatToRawIntBits`/`doubleToRawLongBits` 输出。原 class 在 `java -Xverify:all` 下执行 180 行。

## 三方结果

- 原 class：`javac --release 8` 成功，`java -Xverify:all` 成功，180 行输出作为 `original.txt`。
- JADX：完整三主类输出通过 `javac --release 8` 和 `java -Xverify:all`。三组逐一运行完整 runner；JADX 与原 class 的差异文件各列出同样 13 行，全部来自 overload descriptor 语义：`longToFloatOverload(long)` 中 `(float)` 被 JADX 删除而选择 `take(long)`，`floatToDoubleOverload(float)` 中 `(double)` 被删除而选择 `take(float)`。这证明显式转换后的静态类型和 descriptor 选择不能以“数值可加宽”替代；long→float 与 float→double 在 JLS 分类中均为 widening conversion，前者仍可能丢失精度，不能把这两项称为窄化。
- 当前 jarde CLI：三个主类均返回成功载荷，但完整源文本分别有 18、18、8 个 `@bytecode` 引用；原样编译分别失败（`ConversionIntegers` 9 处、`ConversionLongFloat` 9 处、`ConversionDouble` 4 处“缺少返回语句”）。没有手改恢复文本，也没有把失败裁成可编译子方法。

JADX 完整源码和 jarde 完整源文本分别是各组目录中的 `jadx.java.txt` 与 `jarde.java.txt`；原/JADX/jarde 的编译、执行和报告日志均为实际命令输出。`jadx-differences.txt` 保留了逐行 raw-bit 对照，而不是只比较格式化数值。

## 代码事实和最小复用点

当前 `crates/jarde-java/src/decode.rs` 对这些 JVM conversion opcode 仍生成 `Operation::Other`，所以 Builder 不会把它们送入现有 `Operation::Arithmetic` 或 `Operation::Negate` 的表达式路径；这与三组 jarde 源文本在 conversion BCI 留 quote 相符。最小复用点是现有 operation/decode 的单条转换事实和 `render_value` 的值链：primitive conversion 是一元、无独立 Java statement 的值 producer，只有在 opcode、输入/输出 primitive 类型和最终位置已由同一事实证明时，才应作为现有表达式节点的一部分呈现。

`i2b/i2c/i2s` 不能仅按 JVM 的 int-shaped stack 事实输出裸 `arg0`：主类的 `byteOverload`、`shortOverload`、`charOverload` 证明窄化发生在 invocation descriptor 绑定前，分别选择 `(B)I`、`(S)I`、`(C)I` overload。`l2i`/`l2f`/`d2i`/`d2f` 等也必须保留实际 conversion 的输入边界；浮点到整数的 NaN、无穷和越界结果由 JVM conversion 规则决定，不能从 Java cast 的普通有限值样例外推。这个审计没有提出新 pass、全图类型推断或独立 conversion 框架。

root 已读实际源码、javap 和13行diff：三个组各自保存的是同一完整runner的180行，13行差异也是同一组差异的重复归档，不能累加成540行或39个独立错误。两种重载错误都位于ConversionLongFloat；其它两个主类不能被这份全runner差异误报为各有13个独立错误。尚未实施 primitive conversion；下一步需要把范围与显式Cast、调用实参类型及闭合浮点表达式的求值保存关系写成独立规划。

root 另以全新工作目录重编译、反编译及执行，`root/` 的三个class hash、180行原始结果和13行JADX差异与初审逐字相同。三份jarde完整源码仍分别javac失败，CLI前后未变化。没有用重新执行日志数量增加独立case计数。

## 显式转换链

root新增的`chains/`是独立的完整ConversionChain（601 bytes、9 Code，SHA-256 `8c1bd3a85cbe47829fa5ff85b77621e80531a3e21b78ef60356c35ad043d828a`）。73项包括Int/Long经过浮点后的往返、double→float→double、float→byte、double→char、int→byte→char，以及转换后long相加的左右调用与抛错。原class实际验证执行；JADX完整源码能编译，但7项因消掉中间浮点舍入而错，jarde24引用、完整javac失败。该7项与先前13项重载错误是不同输入。

`recover-primitive-conversions`四份OpenSpec已strict通过。最小方案是单条转换事实接到现有Cast，保持每个中间步骤，继续消费原descriptor；不新增求值器或转换消除。

## 无显式转换的窄整数返回

root 的 `narrow-locals/` 保存普通 javac 输入 NarrowLocals：297 bytes、5 Code，SHA-256 `833415dd5400ba5e477e659379b0bdadad8df0bc5c903cb7a9706467a9946368`。byte、char、short 参数先写入局部再回读返回，另有 byte 局部返回 int 的对照；20 项原 class/JADX 一致。方法体没有 conversion opcode，当前 jarde 将三个窄类型局部声明为 int，然后分别在返回处拒绝，合计3引用、完整 javac 失败；int 返回对照完整恢复。

[JVMS ireturn](https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-6.html#jvms-6.5.ireturn) 明确 B/C/S 返回分别按 i2b/i2c/i2s 窄化，Z 返回则取最低位。这是返回指令和本方法 descriptor 已陈述的语义，不需要先恢复窄局部声明或建立范围求解器。当前 `return_expr` 仅调用普通 Java 位置 widening 规则，缺少这一返回专属事实；不能把普通 assignment/field/invocation 一并放宽。合法 descriptor 变体的边界值实测另行进行，primitive conversion change 继续不包含此项。

root 补查 `return-sinks/`：ReturnSinks 472 bytes、5 Code，SHA `e3d6d5b7352fcce51909b4a970fc1bd199d27c5be9c93088fdcb088e44b4662a`。仅将三个唯一CP descriptor的返回I等长改为B，Code不变，31项原class通过验证执行。JADX生成`??`类型及不合法窄返回，完整javac失败；jarde字段前/后自增以零引用直接输出`return this.value++`/`return ++this.value`，javac报int不能窄化为byte，同步返回也被拒绝。条件选择另因未支持的stack phi拒绝，不能把这个独立原因归给ireturn。

为隔离phi，root另从源级移除该方法及对应调用，再重新编译和应用剩余descriptor补丁，得到 `return-sinks-core/`：411 bytes、4 Code，SHA `dbb7b8cecab3df4d6fcb207cf4db7bbe9337af73d8ceeba8df9f9926087002d5`。19项涵盖前后自增、完整字段值、同步返回和null锁异常。原class全部实际执行，JADX/jarde完整源码均javac失败（jarde1引用），没有声称它们运行一致。两个独立输入共用部分case，不能将重复的字段/同步case累加成50个独立语义场景。

新规划 `recover-narrow-integer-returns` 将转换限制在真实ireturn和B/C/S descriptor，并明确共用普通/同步/switch下推/字段自增的返回适配。既有switch入口的arm呈现位置不同于join的return BCI，需要分别传递求值与来源事实；不增范围求解器，不借此恢复一般stack phi或Z最低位转换。
