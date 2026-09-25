## Context

`TernaryCore.returned(Z)I` 的真实 Code 为 `iload_0; ifeq 10; invokestatic a; goto 13; invokestatic b; ireturn`。`Region::If` 已给出测试、双臂和 join；SSA 在 join 的栈深 0 给出 Entry/Phi，但 `render_value` 目前只接受指令定义，因此两个调用在臂内被引用，BCI 13 的返回值也被引用。根在独立复制的源码上重放：424 B/6 Code，SHA-256 `9c867c925fb0f42b7fe11af18bb4e90645fe6ac8fcd2718114100161378b6ce4`；原/JADX 各两行相同，jarde 完整 `javac` exit 1。完整样本 1414 B/15 Code、SHA-256 `a2e96a4bb220fc3a61f4635827c628b84ad4571d8e71fd9da42a4c0e94c9a80f`，原/JADX 16 行一致，jarde 20 个引用、七个缺少返回语句。

相邻 `assert` 独立实证见 `../../evidence/java-syntax-2026-09-22/assert-syntax/`：原/JADX 整类在 `-ea` 与 `-da` 下逐项相同，基线 jarde 的合成 final 字段 `$assertionsDisabled` 未初始化。`recover-class-literals` 后 root 用冻结 CLI `25bf181ccf0d879351a16710818e27efba0df6b3851931a29eb98d9f41adb1e0` 重放，类常量已恢复，剩余唯一引用是 `<clinit>` BCI13：`desiredAssertionStatus()` 分支各推 `0`/`1`，汇合栈值由 `putstatic Z` 消费。现有输出是空 `if` 加引用，整类 javac 仍失败。该场景与返回/局部值一样是双臂 Phi 的消费者，不构成新的 assert 指令或专用 pass。

`../../evidence/java-syntax-2026-09-24/assert-core/` 把该问题缩成独立 runner 驱动的普通类：`guard`、detail 和 `check` 的条件/抛错已完整写出，Jarde 整类只因 `<clinit>` 的 BCI 13 汇合未写 `static final Z` 而编译失败；原/JADX 启停断言结果分别为 `1|0;bad|2|1` 和 `0|0;0|0`。另一个仅将 `<clinit>` 的类字面量从 `AssertCore` 改为 `StringBuilder` 的合法补丁，在 `-ea:AssertCore -da:java.lang.StringBuilder` 下从前一输出变为后一输出。因而本 change 必须保留条件值原有类字面量与消费者身份，不可把相似控制流专门改写成源级 `assert`；是否恢复该语法及 synthetic 字段元数据是独立跨成员问题。

`tests/fixtures/p3-assert-core/AssertCore-non01-arms.class` 另将同一 `<clinit>` 的 BCI 8/12 两条常量从 `1/0` 改为 `2/3`，其余指令和字段 descriptor 不变，SHA-256 为 `b52d39dd6ae7943d70b510c4925f23faadcc2fcc64c5cb7a2c309ae450de4e2c`。该 class 仍经 `-Xverify:all`，但 `-ea` 得 `0|0;0|0`、`-da` 得 `1|0;bad|2|1`，与原类正好反转。这是有效的 JVM 布尔字段写入，不是合法的 Java 布尔字面量分支；不能仅凭目标 `Z` descriptor 把非 0/1 整数 Phi 拼成两个布尔字面量。

上述断言相关失败是冻结 CLI 的历史起点。当前 CLI `a3a29b59aa85c2d174b06033f874eb7d60a104dbccf1d20f5d88908273e92dd5` 已经复用条件值表达式和既有字段收窄，普通 `AssertCore` 输出 `(!AssertCore.class.desiredAssertionStatus() ? 1 : 0) % 2 != 0`，非 0/1 补丁输出对应 `2 : 3` 再收窄；两份均重编并与原类的 `-ea`/`-da` 行为一致，错误来源补丁的选择性启停亦一致，见 `evidence/assert-core-current/`。因此这里不再预设必须新写字段布尔机制；尚需独立验收真实 `putfield` 消费及一般整数 Phi 的拒绝边界。

## Goals / Non-Goals

**Goals:** 恢复单一 `if` 两臂的直接值汇合，可被 return、局部、算术、调用实参或已证明的字段写入消费；保留测试和恰好一条臂的调用、异常与静态目标。

**Non-Goals:** 不将所有 Phi 都变成条件表达式；本项不把循环、switch、多前驱、不规则分支、跨异常处理器或臂内含独立语句的汇合认作双臂 `?:`，这些形状仍可由已有的独立路径恢复；不引入外部类层级求解器，也不改变一般局部类型推断。不把编译器断言协议直接投影为源级 `assert`，也不宣称普通 Java 源码能保留显式 synthetic 字段标志。

## Decisions

1. 利用现有 `Region::If` 的分支/测试/汇合身份与 SSA Phi 的前驱输入，证明汇合值的两条输入正好对应 true/false 两臂，join 没有外部入口，每臂的产生值在该臂执行一次且只流向这个消费者。审查已存在的区域/栈值记录，按现有预算、深度、取消逐项计费；未知就拒绝，不靠 BCI 邻近猜测。
2. 在已有表达式树增加一个 `Conditional { test, when_true, when_false }` 形状，使局部、算术、实参与返回共享一次值构建；不合成伪 JVM 槽。每个子表达式仍经现有求值位置和 producer 绑定规则，未选臂在 Java 中不执行。臂内若另有不能放入表达式的语句，则不折叠该 region。
3. 从现有 SSA/descriptor 类型事实和消费位置证明条件表达式的 Java 静态类型；基础同型 primitive、同型引用与 `null` 可直接准入。`Z` 字段的整数条件值先保留原整数语义，再由既有字段写入收窄规则在真实 `putstatic`/`putfield` 消费处转换；对 `0`/`1` 可在证明等价时简化，不能把任意 int Phi 直接认作 boolean，更不能把有效 `2`/`3` 补丁误写成布尔字面量。静态 final 左值沿既有字段写入的合法简单名规则。需要提升、装箱或外部继承关系而未证明时保留拒绝。实际调用 Methodref 必须由现有 `invocation_argument`/目标 cast 锁定，不能因条件表达式的 Java 重载解析选错方法。
4. 测试分支、两臂值、跳转及最终消费者的 BCI 归属到同一表达式/语句及各子节点；被认领的调用不得再生成独立可执行语句。失败时引用完整来源，不允许只有 `if` 空臂、没有 return 的半成品冒充恢复。默认/all 证据共享同一正文。

## Risks / Trade-offs

- Java 条件表达式只执行选中臂；提早保存两臂或把 helper 写在 `if` 外会错改 trace 与异常优先级。原始 16 行包括两个抛错分支与调用实参顺序，必须逐项重编译执行。
- Java `?:` 的静态类型可能改变重载选择；仅靠 JVM 栈形状不能准入不同引用类型，必须保留实际 Methodref 的目标类型或拒绝。
- 全局 Phi 解析会把循环/多入口汇合误判为条件值；本 change 只扩展已经有双臂结构证明的局部区域。
- 断言开关的来源类并不必然是当前类；仅改变一个 `ldc` 就可让选择性 `-ea`/`-da` 结果相反。条件值恢复只保留原测试和值流，绝不由字段名猜断言语句；新最小正负例逐项核对该来源。

依据：[JLS 8 §15.25 条件运算符](https://docs.oracle.com/javase/specs/jls/se8/html/jls-15.html#jls-15.25)规定先求值测试、只执行选中的臂并完成条件表达式类型转换。
