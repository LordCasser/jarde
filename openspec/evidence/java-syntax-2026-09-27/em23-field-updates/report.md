# EM-23：字段自增/自减与局部更新首片对照

固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `arith/TestFieldIncrement` 对实例 `field++`、静态 `field--` 和字符串 `result += s + '_'` 有活动的精确源码断言。`arith/TestArith` 的局部 `+= 2`/`++` 精确断言带 `@NotYetImplemented`，活动用例仅要求源码可生成；`arith/TestFieldIncrement2` 还要求嵌套接收者 `this.a.f += n`/`*= n`，`variables/TestVariablesDefinitions2` 则涉及循环作用域。四份测试文件的 SHA-256 由 [replay.py](replay.py) 核对，本首片不把未复现的嵌套/循环形态算作通过。

[input/em23/](input/em23/) 用 Java 8 无 debug 信息编译完整 `Updates` 类与共同 Runner，包含实例字段加一、静态字段减一、字符串追加及两个局部参数更新。原 class、固定 JADX 和 Jarde CLI (`8f1f0012324350e4fc65c7fef4b3e3835102e6d4fda00df280b466270c98e727`) 的完整类源码均通过 Java 8 重编、`java -Xverify:all`，逐字输出 `2:0:A_B_`、`5:4`。独立第二次重放的摘要及两份反编译源码逐字节相同；源码和编译/运行日志在 [acceptance/](acceptance/)。

JADX 对字段输出 `this.instanceField++`、`staticField--`、`result += str + '_'`。Jarde 的等效文本分别是 `this.instanceField += 1`、`Updates.staticField = Updates.staticField - 1`、`Updates.result = new StringBuilder()...toString()`。JADX 对局部参数 `value += 2`/`value++` 本身也简化为 `return i + 2`/`return i + 1`，Jarde 保留参数写入再返回，三方语义一致；由于原精确断言未完成，本轮不把局部拼写差异当作应追平目标。

现有 Jarde `CompoundAssignments`、`FieldIncrements` 和 `PostIncrement` 已有同次 SSA/字段 read-write 证明；实例 `++` 的可编译 `+= 1` 说明字段身份和求值顺序已恢复，差距在末端源码形式。静态 `--` 目前走普通字段读、减一、写回；若要输出源级 `--`，AST 需有受证明的后缀减一表达式，或将现有 `PostIncrement` 归一为带方向的同类节点。这是有限的表达式表示扩展，不需要新 JVM IR 或泛化反编译 pass。[字段单位更新 OpenSpec](../../../changes/spell-proved-field-unit-updates/)将二者作为同一小证明任务。

字符串 `+=` 涉及 StringBuilder 构造链与字符串拼接消除，归 EM-27 单独审计和任务，不能混入整数单位更新。`TestFieldIncrement2` 的 `this.a.f` 可能跨两次字段读取，接收者身份/求值次数必须单独闭合；`TestVariablesDefinitions2` 的循环局部则归 EM-20/CF 控制流交叉审计。EM-23 只移到“部分已测”，本首片不宣称全部 `++`/`--`、前后缀返回值或复合赋值已追平。
