# DT-14：枚举用户初始化与三元构造实参

## 结论

DT-14 的两个子形态都存在 Jarde 源码重编差距，但根因不同，已分别提窄 OpenSpec。

- **自定义静态初始化后缀**：Jarde 对普通 enum 和无参数常量前缀有可用闭环，但 `values()` 遍历后向 `Map` 写入的用户静态后缀不在当前枚举投影门内。Jarde 保留原物理常量字段和 `<clinit>`，把物理字段写在 `enum` 体开始处，Java 8 编译器报 `enum constant expected here`。
- **三元构造实参**：当前 int 实参闭集只接受整数 literal、已认证 `getstatic:I`、以及加法；javac 将 `condition ? left : right` lowering 为 `<clinit>` 内条件分支。整个 enum 常量组不能进入源码投影，保留物理字段后同样无法编译。JADX 的实际 `TestEnumsWithTernary` 还把 `String` 字面量实参与三元组合在一起，因此实际测试同时触及 DT-11 的普通 String 构造实参边界；我们另以 int 三元 fixture 隔离三元 lowering。

普通 enum 与 int 字面量构造实参控制均在 Jarde 中完整 Java 8 重编并通过 `-Xverify:all`；所以失败不是普通枚举声明或基础 int literal 投影。

## JADX 固定输入及断言边界

- 固定 checkout：`/Users/lordcasser/workspace/testzone/jadx`，HEAD `2fb1b16386941660fda07e9017285aec40fcb37f`。
- `enums/TestEnumsWithCustomInit.java` 定义 `TestCls`，三个常量各传一个 String literal，随后 `MAP = new HashMap<>()` 并在 static block 中遍历 `values()` 逐项写入。JUnit 断言仅检查一次 `ONE("I"),` 与不出现 `new TestEnumsWithCustomInit$TestCls(`；没有运行时行为断言，也没有确认 map 初始化循环的完整输出。String 构造实参与 static suffix 是组合形态，本次用无参 enum fixture 隔离后者。
- `enums/TestEnumsWithTernary.java` 在 `FIRST`、`SECOND`、`ANY` 三项传入 `useNumber() ? String : String`，用 `noDebugInfo()` 在 `DX_J8` 与 `D8_J8` profile 检查一次 `ANY(useNumber() ? "1" : "2");` 和不存在 `static {`。这是正向文本断言，但 `useNumber()` 恒为 false，测试本身没有原 class/JADX 全源码 Java 8 重编与运行对照。String 参数又与 DT-11 非 varargs String 参数重叠；int 三元对照用具有效果的条件调用单独定位三元表达式差距。
- `enums/TestEnumsWithStaticFields.java` 来自 Smali，而不是可由本测试源码编译出的 Java enum；测试明确调用 `disableCompilation()`。它断言混淆输入中的 `sB` 字段保留、`sA`/`sC` 与指定构造器文本消失。其 `<clinit>` 还含插桩调用和实现接口状态字段，不证明 Java 源级 static initializer 可重编/运行，故只记作字段处理弱证据，不扩充本次 DT-14 正例。

JADX `EnumVisitor.convertToEnum` 从线性 class initializer 中寻找 `$VALUES` 写入与 enum 字段，构造 `EnumClassAttr` 并把 enum 常量构造实参交给 `processConstructorInsn`；常量实参由 `ClassGen.addEnumFields` 经 `InsnGen.generateMethodArguments` 输出。处理 enum 后仍有字节码的 `<clinit>` 会保留为静态代码。JADX 侧仍须以本报告三方重编对照为语义标准，不把文本断言当作运行语义证明。

## 冻结 fixture、字节码及重放

本目录的源码将两个被测子形态与两个正向控制分开：

- `CustomInit.java` / `CustomInitRunner.java`：无参 `RED`、`BLUE`，完整用户 suffix 分配 map 并逐一存放 enum 实例；期望 `map=2:true:true`。移除 String 构造参数避免 DT-11 混入。
- `TernaryInit.java` / `TernaryInitRunner.java`：三项 int ternary，`useNumber()` 每次递增计数并交替返回 true/false；期望 `ternary=1:20:1:3`，观察每个条件只执行一次且两分支都被选择。
- `StringTernaryInit.java` / `StringTernaryInitRunner.java`：与 JADX 测试同样的三个 String ternary 和恒 false `useNumber()`；期望 `string-ternary=A:B:2`。
- `LiteralInit.java` / runner：int 常量实参控制，期望 `literal=1:20`。
- `PlainInit.java` / runner：无自定义 static suffix 的普通 enum 控制，期望 `plain=2:true`。

在仓库根执行 `python3 openspec/evidence/java-syntax-2026-09-27/dt14-enum-init/replay.py baseline`。脚本要求固定 JADX HEAD 且 checkout 干净，使用 `TemporaryDirectory` 建原始 Java 8 class、jar、JADX/Cargo 构建和各自编译目录；Rust `CARGO_TARGET_DIR` 与其余中间产物自动删除。未来两份提案分别用 `fixed-custom`、`fixed-ternary` 模式验收，合并后用 `fixed-both`；每个模式输出到独立同名目录，不覆盖基线，未修的子形态仍须保持其冻结失败边界。结果 `results.json` 含所有 fixture 的输入 SHA、class SHA、三方编译/运行出口与稳定化编译诊断；`*-javap.txt` 保存路径/日期归一化后的 enum `javap -v -p`，`*-jadx.java.txt` 和 `*-jarde.java.txt` 保存最小完整物理类源码结果。

固定环境为 `javac 23.0.1 --release 8` 与 OpenJDK 23 的 `java -Xverify:all`。所有原始类及 JADX 完整源码均成功编译、验证运行，且每组输出逐字节相同。Jarde 普通 enum 与 int literal enum 都编译、验证运行且输出相同；CustomInit、int ternary、实际 String ternary 三个 Jarde 输出都在 Java 8 编译时失败，首条诊断分别为：

```text
CustomInit.java:6: error: enum constant expected here
    public static final dt14.CustomInit RED;
TernaryInit.java:6: error: enum constant expected here
    public static final dt14.TernaryInit FIRST;
StringTernaryInit.java:6: error: enum constant expected here
    public static final dt14.StringTernaryInit FIRST;
```

## classfile 形状与 Jarde 生产入口

`CustomInit.<clinit>` 在 enum 标准前缀之后继续：`new HashMap` / `putstatic BY_NAME`，调用 `values()`，取得数组长度，然后以 `if_icmpge` / `goto` 形成循环；循环体读取 enum `name()`，调用 `Map.put` 并回跳。原顺序在 `custom-init-javap.txt` 中可按 BCI 39（map 字段写）、42（`values()`）、循环分支及 `Map.put` 找到。Jarde 文本确实保留了这个 `<clinit>` 循环和所有物理 origin，但没有把物理 enum 常量转换为 enum grammar 中的 `RED, BLUE`，所以 enum 体先出现字段声明并不能通过 javac。

`TernaryInit.<clinit>` 每个常量都是 `new/dup/name/ordinal` 后调用 `useNumber()`，再经 `ifeq`、两个 literal 值分支和汇合点到 `(String,int,int)` 构造器与对应 `putstatic`。第一个常量是 BCI 0–22；条件在 BCI 7 调一次，BCI 10 分支到 17，BCI 19 调构造器。Jarde 目前的 `prove_initializer_prefix` 调用 `prove_int_source_argument`，该语法不包含分支/合流，所以整个常量组无法投影；在物理 `<clinit>` 中能看到点名 BCI 的 fallback 标记。

公开生产路径是 `Engine::class_source_with_evidence` → `prepare_physical_class_source` → `enum_constants::prove_group` → `prove_initializer_prefix` → `class_source::prepare_enum_constant_source_projection`。用户静态尾部由 façade 在 enum 组已证后检查：`has_only_terminal_initializer_return` 是普通无后缀路径，另一路 `prove_static_assignment_suffix` 只接受现有专门化的 `LOW(2), HIGH(5), totalUnits = sumUnits()` 证明。它不接受本次 Map 初始化循环。int 实参 proof 的 `prove_int_source_argument` 只覆盖限定叶子和加法；conditional branch 不能凭 code 邻近重建。两个根因分属 suffix statement proof/projection 与 constructor expression proof；不应合并成一个放宽 enum 门。

## 窄提案

分别提交 `recover-enum-user-initializer-suffix` 与 `recover-ternary-enum-arguments`。前者只在完整 enum prefix 后的用户 `<clinit>` suffix 能由同次结构证据完整表达时，原子投影 enum 常量与用户 static field/block；首个验收形状是上面的 `Map` 初始化与 `values()` 遍历，不扩成任意 class initializer decompiler。后者只接受完整、有界的 conditional diamond 构造实参，保留条件的单次求值/分支次序和对应构造器实参。凡未解释指令、额外效果、入口/汇合歧义或停止都拒绝整个 enum 常量组，不剥除物理来源。两项都不实现生产代码；普通枚举、DT-11 String-varargs 和 DT-12/13 均保持独立边界。
