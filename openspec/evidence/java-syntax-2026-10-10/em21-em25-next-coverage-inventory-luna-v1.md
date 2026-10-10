# EM-21～25 下一项活动 JavaInput 正向覆盖盘点

本盘点按 71 单元账本顺序，只找现有 JADX Java integration tests 中仍有活动源码断言、但该精确形状尚未进入完整类三方证据的候选。它不把测试代码本身当成行为差异，也不推断 Jarde 当前对此形状失败。

## EM-21：当前活动 `this` 局部别名用例已有完整类证据

- 账本：[expressions-misc.md](../jadx-feature-inventory-2026-09-27/expressions-misc.md) 第 61 行列出 `TestInlineThis`、`TestInlineThis2`、`TestDontInlineThis`，并说明其它消费者/槽复用仍待扩验。
- `TestInlineThis` 的活动 JUnit `@Test` 是第 23～33 行：要求直接 `this` 局部仅作实例调用/字段接收者时别名消失（第 27～32 行）。`TestInlineThis2` 第 30～41 行再覆盖 `Objects.isNull(thisVar)` 读取和同一别名的调用/字段写入（第 34～40 行）。`TestDontInlineThis` 第 31～38 行要求 `this` 与 `new TestCls()` 合流来源保留 `res`（第 35～37 行）。
- 这些形状都在既有完整 fixture [`ThisUse.java`](../java-syntax-2026-09-27/em21-this-alias/input/em21/ThisUse.java) 中：`inline` 第 9～13 行、`checked` 第 15～22 行、`choose` 第 24～34 行。其[JADX 基线报告](../java-syntax-2026-09-27/em21-this-alias/report.md) 第 3、5、9 行明确说固定重放包含上述三个测试，并对原/JADX/Jarde 完整类重编验证；[主线验收](../java-syntax-2026-09-27/em21-this-alias/root-acceptance-2026-09-27.md) 第 3 行记录独立重放成功。`TestRedundantThis` 的唯一 `@Test` 在源文件第 27 行被注释，不能算活动缺口。
- 因此 EM-21 账本所列活动 Java 断言已由同形完整类证据覆盖；账本写出的继承字段遮蔽、构造器别名、逃逸/复写/槽复用仍是扩验边界，但这不是本次从固定活动断言中找到的新候选。

## 首个清晰的新候选：EM-22 `TestRedundantBrackets` 的整数掩码条件

- 固定 JADX 测试源：`/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/others/TestRedundantBrackets.java`。`TestCls.method3` 在第 23～31 行包含 `if ((a & b) != 0)`（第 27 行）；活动 JUnit 测试第 45～62 行明确断言 `(a & b) != 0`（第 54 行）。这是 JavaInput 正向源码断言，不依赖 Smali/DEX-only 测试。
- 已有 EM-22 完整类证据在 [`Arithmetic.java`](../java-syntax-2026-09-27/em22-arithmetic/input/em22/Arithmetic.java) 第 22～28 行覆盖 boolean `|`/`&` 左结合返回式，但没有 bitwise AND 与整数比较组合成条件语句。Jarde 输出也对应这两个返回式（[完整源码](../java-syntax-2026-09-27/em22-arithmetic/acceptance/source/jarde-Arithmetic.java) 第 44～56 行）。EM-22 的重放脚本固定并校验 `TestRedundantBrackets.java` 哈希（[replay.py](../java-syntax-2026-09-27/em22-arithmetic/replay.py) 第 16～20 行），但现有 fixture / 报告只说从混合用例取括号方向，不是 `method3` 的完整类输入（[report.md](../java-syntax-2026-09-27/em22-arithmetic/report.md) 第 3、5、18～20 行）。因此不能把现有 `and(boolean,boolean,boolean)` 完整类复编当作第 27 行条件形状已测。
- 最小下一候选是完整类只保留同形 `method3(int a, int b) { if ((a & b) != 0) return ...; return ...; }` 与一个共同 Runner，对照原/JADX/Jarde 全类编译和验证运行，并断言输出中条件保留 `(a & b) != 0`。这能同时核实 bitwise 表达式位于 `if` 条件时的括号/比较绑定及原始行为。不要把 cast、`instanceof`、数组复合赋值等同一测试类中的其它断言一并扩进首片。
- 从现有架构看无需预设新机制：`build.rs:25229-25327` 已把 `Operation::Bitwise` 变为带类型证明的 `ExprKind::Binary`；`emit.rs:1415-1418` 输出二元表达式，`emit.rs:1787-1798` 集中定义 Java 二元运算符优先级；`emit.rs:945-954` 在 `if (...)` 中复用表达式 writer。下一片应先做完整类对照，若失败再用实际 javap/报告来源定位是表达式类型、条件 region 还是括号绑定，不先增 AST/pass。

## 顺序建议

优先只验证上面 EM-22 条件形状，避免同时重测 EM-18 数组、`TestArith2` 已覆盖的括号/左结合/boolean eager 形式，或扩成 EM-25 cast/重载任务。若该对照通过，账本仍可将 EM-22 标为部分已测；后续再从 EM-23 的活动 `TestFieldIncrement2` 接收者链或 EM-24 明确未覆盖的连续 `Object` 字段赋值中，按固定活动断言继续选一项。当前没有证据支持宣称本 EM-22 候选存在产品缺陷。
