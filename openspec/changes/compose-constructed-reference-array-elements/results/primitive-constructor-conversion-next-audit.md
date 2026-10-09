# 构造器参数 PrimitiveConversion 边界审计

## 结论

当前 EM-18 片的目标是把已证明的 fresh reference-array initializer 与数组元素中的已证明构造器站点组合起来。它不应顺带放行构造器参数里的数值转换。direct-v3 的 `boxedDirect()` 在五个 wrapper 元素中，分别在 `Byte`、`Short`、`Long`、`Float`、`Double` 构造器调用前执行 `i2b`、`i2s`、`i2l`、`i2f`、`i2d`；现有 `new` 参数区间扫描把这些操作归为 `StatementFree` 拒绝。拒绝点在表达式白名单，早于（且独立于）数组元素的引用兼容判断。

这里的 `PrimitiveConversion` 是纯值变换，没有独立调用、副作用或 JVM 异常；Builder 已能把它渲染为有明确目标类型的 Java 显式 cast。后续最小工作可在 `init` 既有构造参数证明中，有条件地接纳已属于真实参数依赖图、且 Builder 能按 opcode/source category 证明的 primitive conversion，继续复用现有表达式、顺序、effect 依赖与渲染路径。不需要新 pass、AST 节点、赋值兼容规则或通用运算白名单。该工作应另开 OpenSpec；本片仍只处理构造元素与 `aastore`/数组 initializer 的组合。

## 冻结输入与拒绝位置

- 输入源码：`tests/fixtures/p3-heterogeneous-array-initializers-v3/direct/Main.java` 中 `boxedDirect()`，源码写出六个元素：`new Byte((byte) mark(1))`、`new Short((short) mark(2))`、`new Integer(mark(3))`、`new Long((long) mark(4))`、`new Float((float) mark(5))`、`new Double((double) mark(6))`。`mark` 更新可观察 trace 并返回其参数。
- classfile 对照：`openspec/changes/compose-constructed-reference-array-elements/results/baseline-javap-direct-javac8.txt` SHA-256 `ae496a7fa34352d5cd68e49ce81e2e801b657752a76222785010b497aea24424`；javac 23 对照 `baseline-javap-direct-javac23.txt` SHA-256 `26fe4e5c64e66942e93fe9da460f9b1de44be63fd5221c6b2639e4666dbe11e8`。两者 `boxedDirect` 的五条链分别是 `new/dup/mark/i2b/<init>/aastore`、`new/dup/mark/i2s/<init>/aastore`、`new/dup/mark/i2l/<init>/aastore`、`new/dup/mark/i2f/<init>/aastore`、`new/dup/mark/i2d/<init>/aastore`。准确 BCI 是：Byte `new 7, dup 10, mark 12, i2b 15, <init> 16, aastore 19`；Short `22,25,27,30,31,34`；Long `51,54,56,59,60,63`；Float `66,69,71,74,75,78`；Double `81,84,88,90,91,94`。Integer 对照链没有 conversion：`new 37, dup 40, mark 42, <init> 45, aastore 48`。
- 冻结 v3 家族结果：`openspec/changes/recover-heterogeneous-array-init/results/candidate-v1-fixture-v3/manifest.json` SHA-256 `d3dd331861d79d32cc64b2fd2e2c3677606868d54767e157fa99e5db7f508b87`；root 核验 `openspec/changes/recover-heterogeneous-array-init/results/candidate-v1-fixture-v3-root-verification.json` SHA-256 `96087151f77d0dfcde9f4503c0479f686eb250e99b5fc3437ab1498346c193cb`。此处只引用冻结输入身份，不重跑目标或重写历史证据。direct 生成代码两个 javac legs 均以 exit 1 拒绝；不得把结构拒绝描述成 primitive 类型不兼容。
- 当前 `crates/jarde-java/src/init.rs` 的构造器参数验证先取物理参数 ValueId，在 `value_dependency_bcis_metered` 中沿定义和 `reads()` 构造依赖 BCI 集合。参数区间 `dup` 与 `<init>` 之间允许 Push、Load、Arithmetic、Negate，允许且仅允许处于真实参数依赖集内的 Invoke/InvokeDynamic；其余已解码 operation 进入通用 `Precondition::StatementFree` 拒绝。当前白名单不含 `Operation::PrimitiveConversion`，因此上述五条在 conversion BCI 被拒绝。`PrimitiveConversion` 已在参数依赖图上，但依赖成员身份尚不足以通过该区间 effect/statement gate。
- 拒绝与数组元素类型关系是两个证明面：数组候选先证明 store/value/site 的闭合关系，构造器参数 scan 再证明构造表达式区间可被内联。不能用 `Number[]` 对 `Byte/Short/...` 的兼容性替代任何一个结构证明；也不能把 `i2b` 等“转成 wrapper 参数类型”混称为 reference assignment compatibility。

## 最小组合路径与边界

建议的后续改动只在 `crates/jarde-java/src/init.rs` 的现有参数区间分类中增加一个有证明前提的 primitive-conversion 分支：conversion BCI 必须属于该构造器物理参数的 SSA value dependency 集，操作仍须由 decoder 精确识别，且通过现有 Builder expression closure 与源/目标类型检查。保持 `Push/Load/Arithmetic/Negate` 等原分支不变，不以“纯指令”概念扩大到所有 opcode。表达式由 Builder 现成的 `Operation::PrimitiveConversion` arm 递归渲染其唯一 stack read，并产出目标类型 `ExprKind::Cast`；`emit.rs` 已按 Java unary/cast 优先级输出 `(target) operand`。`primitive_conversion_source_matches` 会把 JVM int 类别限制为 Java byte/char/short/int，并明确拒绝 boolean；long/float/double 则要求精确类别。后续实现须调用/复用这些证明，不另造 cast 语义表。

效果和求值位置仍由原有验证负责。参数必须由 `dup` 后、constructor call 前的指令产生；构造器参数按调用 operand 顺序建立；`mark` 仅在调用的实际参数依赖图内才可出现在该 run 中。conversion 本身无调用/副作用/异常，因此允许它不会改变 dependency 中 `mark` 的调用次数或源码嵌套位置。Builder 递归表达式必须消费 conversion 的确切输入一次，不能仅因 BCI 同在区间就接受。这里不把既有 handler 证明夸大为一般的普通 invocation 参数异常边界证明：init 里 handler coverage 受 concat/dynamic/inline-array/nested-construction 等闭包分支控制；`mark` 的一般调用并未由“PrimitiveConversion 是纯操作”自动获得 handler split 证明。若后续对抗测试显示普通 invocation handler 边界还有缺口，应单独登记，不借本项扩围。

排除在本项之外的操作家族：

| 家族 | Builder/decoder 现状 | 为何不随本项放行 |
|---|---|---|
| Shift | decoder 有 `Operation::Shift`；表达式 renderer 支持移位 | 独立运算类别，含左右操作数、Java int/long shift distance 与掩码语义；不是 primitive conversion。可另立纯表达式构造参数边界。 |
| Bitwise | decoder 有 `Operation::Bitwise`；renderer 可按类型证据呈现 `&`、`|`、`^` | Java 的整型与 boolean 运算符重载语义不同；单独审计类型证据、优先级和 boolean/int 边界，不纳入 conversion whitelist。 |
| CheckCast | Builder 可呈现 reference `ExprKind::Cast`，但 JVM `checkcast` 可抛 `ClassCastException` | 不是纯值转换；需独立异常/handler 与检查位置证明，不能复用 primitive conversion 的纯 effect 结论。 |
| 其他未知、存储、调用或控制流操作 | 依现有 scan 拒绝或由专用嵌入证明接管 | 不因 conversion 分支而改变 `StatementFree` 原则。 |

## 15 个 JVM 数值转换的可呈现性与值语义

`crates/jarde-java/src/decode.rs::primitive_conversion` 已逐项映射 15 个 classfile opcode 到 source/target `Type`；`build.rs` 对 `PrimitiveConversion` 的渲染是一个统一 cast 路径；`emit.rs` 已输出显式 cast。表中的 Java 表达式成立仍以源类型证明通过为前提，尤其 JVM verifier 的 int 类别不等于 Java `int` 语义，boolean 必须拒绝。

| Opcode | Java source → target | Java 可表达形式 | JVM/JLS 值语义要点 |
|---|---|---|---|
| `i2l` | int → long | `(long) x` | 有符号扩展，精确。 |
| `i2f` | int → float | `(float) x` | binary32 最近值舍入；大整数可丢精度，例如 `16_777_217`。 |
| `i2d` | int → double | `(double) x` | 所有 int 值可精确表示。 |
| `l2i` | long → int | `(int) x` | 保留低 32 位，按 int 二补码解释。 |
| `l2f` | long → float | `(float) x` | 最近值舍入，可能丢精度/范围信息。 |
| `l2d` | long → double | `(double) x` | binary64 最近值舍入；超过精确整数范围时可丢低位。 |
| `f2i` | float → int | `(int) x` | 有限值向零截断；NaN 为 0；越界及 ±Infinity 饱和到 int 边界。 |
| `f2l` | float → long | `(long) x` | 有限值向零截断；NaN 为 0；越界及 ±Infinity 饱和到 long 边界。 |
| `f2d` | float → double | `(double) x` | binary32 值可精确扩展到 binary64；保留符号零、无穷和 NaN 类别。 |
| `d2i` | double → int | `(int) x` | 有限值向零截断；NaN 为 0；越界及 ±Infinity 饱和到 int 边界。 |
| `d2l` | double → long | `(long) x` | 有限值向零截断；NaN 为 0；越界及 ±Infinity 饱和到 long 边界。 |
| `d2f` | double → float | `(float) x` | 最近值舍入；可能溢出到同号 Infinity、下溢到零；NaN 仍为 NaN。 |
| `i2b` | int → byte | `(byte) x` | 保留低 8 位并按 signed byte 解释。 |
| `i2c` | int → char | `(char) x` | 保留低 16 位，unsigned UTF-16 code unit。 |
| `i2s` | int → short | `(short) x` | 保留低 16 位并按 signed short 解释。 |

这些转换本身不抛运行时异常。精度、窄化、NaN、无穷和溢出细节来自 JLS 5.1.2/5.1.3；classfile 指令定义来自 JVMS Chapter 6。Jarde 当前 decoder 的 15 项映射单测 `all_fifteen_numeric_conversion_opcodes_keep_their_source_and_result_types` 证明 opcode 分类，不是运行时语义验收。Builder 的 `explicit_conversion_source_categories_accept_int_family_but_reject_boolean` 验证 int-family 类型条件，同样不能替代端到端值测试。

## JADX 参照边界

本机 JADX 是 DEX 输入的反编译器，以下仅作为 cast 建模/渲染与回归设计的旁证，不是 Jarde classfile 算法的等价证明：

- `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/instructions/InsnDecoder.java` 的 CAST decode 将 DEX 数值转换映射成 source/target 类型 cast；同文件相邻的 `CHECK_CAST` 是另一种 operation。
- `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/codegen/InsnGen.java` 的 `CAST`/`CHECK_CAST` 共享 `(type) arg` 文本代码生成。文本相同不意味着两者 effect 相同。
- `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/SimplifyVisitor.java` 的 cast simplify 在 `ArgType.isCastNeeded`、duplicate、outer-shadow 条件允许时消除冗余 cast；`isArithWideUpCast` 特意在算术父指令需要时保留 wide cast。该算法说明 cast 类型对后续运算语义有意义，但不能直接推导 Jarde 的 constructor interval 安全性。
- `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/types/TestPrimitiveConversion.java` 以 smali 测试要求保留 `z ? (byte) 1 : (byte) 0`，不能把 boolean 值直接作为 byte 实参；`/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/core/dex/visitors/typeinference/PrimitiveConversionsTests.java` 测 numeric type comparison 并排除 boolean。它们是 Java 类型/cast 表达的测试，不覆盖全部 JVM 数值边界值。
- `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/types/TestPrimitiveConversion2.java` 是 bitwise/bool-int 独立族测试参考，不作为本项放行依据。

## 后续独立 OpenSpec 的最小验收

范围应明确定义为“在已通过 `new` 与数组组合结构证明的 constructor argument 中，允许 Builder 已支持且来源/目标可证明的 `PrimitiveConversion` value expression”。验收需覆盖以下闭环：

1. 五个 direct-v3 wrapper case 在 javac 8 与 javac 23 两套真实输入上进入 initializer，保留 Byte/Short/Integer/Long/Float/Double 实际对象与值；生成源必须完整编译、运行 `-Xverify:all`，stdout 和 stderr 原始字节逐字匹配原程序，不能将只编译或空体当成功。
2. 单测覆盖全部 15 个 decode mapping、每个 opcode 的 source category/target cast 以及目标 AST/render。对 `i2b/i2c/i2s` 检查负数和高位截断；`l2i` 检查低 32 位；`i2f/l2f/l2d` 覆盖精度边界（如 `2^24+1`、超过 `2^53` 的 long）；float/double 到 int/long 覆盖正负小数向零截断、NaN→0、±Infinity 与越界饱和；`d2f` 覆盖精确值、溢出、下溢、±0 与 NaN；`f2d` 验证拓宽结果。通过 `Float/Double.isNaN`、`isInfinite`、符号零检查及明确数值断言验证规范保证的性质；不要要求 NaN payload 保持，因为规范不保证其 payload。
3. 观测 argument effect 次序与唯一求值：多参数构造器中给每个参数 effect 添加有序 trace，并使 conversion 前后都依赖该 effect；构造器内记录最终 primitive 值。输出应证明每个 effect 恰好一次、左到右顺序不变、构造器收到的值保持转换语义。额外对照只有纯 cast 时无额外输出的 case。
4. 负例继续拒绝：boolean verifier-int 被当 numeric source、conversion 不在物理参数的 dependency 图内、转换有多个/不明 stack read、参数来自 dup 前/调用后、存在不相关 effect 或第二 consumer、参数/constructor descriptor 次序不吻合。`checkcast`、shift、bitwise 不属于此 OpenSpec 的正例。
5. 针对 handler split 做独立对照：primitive conversion 本身不会抛异常；包含 `mark()` 的普通参数仍须保留原异常区域及副作用顺序。若普通 invocation handler coverage 需要新规则，应单独以失败分析提出，不将“cast 纯”视作已证明 handler 可重排。

规范参考： [JLS 5.1.2 Widening Primitive Conversion](https://docs.oracle.com/javase/specs/jls/se21/html/jls-5.html#jls-5.1.2)、[JLS 5.1.3 Narrowing Primitive Conversion](https://docs.oracle.com/javase/specs/jls/se21/html/jls-5.html#jls-5.1.3)、[JVMS Chapter 6 Instructions](https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-6.html)。

## 适用范围声明

本报告只记录构造参数中的 primitive conversion 缺口和后续验收建议，不要求本片实现、不更新当前 change 的 planning/spec/tasks，也不修改产品或 fixture。数组组合完成后若五个 boxedDirect 条目仍因这些 BCI 拒绝，应将其登记为独立 whitelist/render composition 缺口；不要归因为引用赋值兼容。
