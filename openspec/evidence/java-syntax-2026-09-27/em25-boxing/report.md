# EM-25：装箱/拆箱与引用 cast 边界审计

## 固定来源与有效断言

审计使用固定 JADX HEAD `2fb1b16386941660fda07e9017285aec40fcb37f`。`replay.py` 冻结并校验 `TestDeboxing.java`、`TestDeboxing2.java` 与 `DeboxingVisitor`、`MethodInvokeVisitor`、`TypeInferenceVisitor`、`InsnGen` 的 SHA-256；完整一次回放及三方源码快照保存在 `replay/`。

EM-25 inventory 列出 `TestDeboxing2.java`，其活动 `@Test` 确实断言 `Long` 参数方法签名、null 分支和默认值赋值、`test(null/0L/7L)` 调用文本；但没有断言 `.longValue()` 是否出现。固定 JADX 的源码保留 `return l.longValue();`，Jarde 也显式输出 `arg0.longValue()`，所以该拆箱子形态没有观察到源码或运行差异。`TestDeboxing.java` 在相同固定 HEAD 另有未被 inventory 列出的活动 `@Test`：明确断言六种 primitive 字面量结果分别写作 `1`、`true`、`(byte) 2`、`(short) 3`、`'c'`、`4L`，还断言两处 `use(true)`。这是一个 inventory 清单漏项线索；本报告记录但不改全局 inventory。JUnit `TestCls.check()` 引用 AssertJ，不能直接当成无外部依赖的可编译运行夹具；本审计改用下述纯 Java fixture 和共同 Runner。

固定 JADX 的 `DeboxingVisitor` 文档说明它删除 primitive boxing；其注册的 `valueOf` 只有 `Integer`、`Boolean`、`Byte`、`Short`、`Character`、`Long` 六类，并要求 invocation 有 result、参数是 literal，再按 SSA uses 决定是否能改成 primitive。`TestDeboxing` 的六条返回断言正覆盖本窄片。Jarde 的普通 `CallTarget` 在 `call_expr` 中统一构成 `ExprKind::Call`；普通方法没有相应的 source deboxing 规则，故保持 `java.lang.Wrapper.valueOf(literal)`。现有 `lambda::TypeConversion::{BoxPrimitive,UnboxPrimitive}` 只证明 LambdaMetafactory adaptation 边，不能当作普通方法转换已支持。JADX 的 `MethodInvokeVisitor`/`TypeInferenceVisitor` 对应重载实参 cast 插入；`InsnGen` 输出 `CAST`/`CHECK_CAST`，这些属于其它边界。

## 边界归属

- `TestCastOfNull` 在 EM-25 inventory 中列出，也与 EM-11 的 null 重载选择边界交叉：它验证保留数组、`String`、`List<String>` cast。这里记录其 EM-11 语义归属，但不将这些 cast 当作本装箱切片的正例。
- `TestCastInOverloadedInvoke` 活动断言检查泛型集合、`instanceof` 后 String cast 和 null 重载选择，归 EM-11。另一个 `testNYI` 带 `@NotYetImplemented`，不得计作通过；本切片不复制它。
- `TestDuplicateCast` 是 `checkcast` 及重复 cast 指令折叠，归 DT-28 引用 cast；`TestPrimitiveCasts` 的反断言检查 primitive cast 清理，也不构成本 OpenSpec 的装箱证据。
- `TestDeboxing2` 代表普通 `Long` 拆箱，并作为对照保留；不会把显式 `.longValue()` 擦除，也不会扩展到通用 `Number`、可空传播或泛型。

## 三方固定回放

`input/em25/BoxingAudit.java` 将六条固定 deboxing 正向断言缩成无第三方依赖的方法；同目录 `Runner.java` 观察包装类、值、缓存范围内重复 boxing identity、`Long` null/0/7 拆箱以及 `Boolean` true/false/null 行为。javac 原始 class SHA-256 为 `143b306e9a7e124d039a02f3247550acf7f10eda61cade92f4c261baefb4346a`，由回放脚本在临时目录计算，不复制 `.class` 到证据目录。一次完整日志、来源与三方编译/运行输出保存在 `replay/`。第二次独立运行的 `summary.json` 与首次字节完全相同，SHA-256 均为 `3812f2c44f6e97cc52f116b1b6166879e419f1ea407132b3debb825b6231a064`；原始/JADX/Jarde 源码 SHA-256 依次为 `73d36bf8f1528f4d54eb6880e917c8f8566f822133221d68e1f3b8dd46a6d15d`、`b46d40385a4a7f3a0ae3fa22ed4261cbfd3df0afc19c0c44057b4c8bd113d09a`、`24dfd32612c2239cfe5aff7ed0c7b768d3312c7fd3a92159db5fa00c788c16e0`，三者相同。第二次运行的完整重复日志未保留。`replay.py` 对 verbose Jarde 输出只保留源文件快照与字节/hash 元数据，便于重放并避免重复提交大日志。

两次运行中原、JADX、Jarde 都以 `javac --release 8 -g:none` 编译完整 `BoxingAudit` 与同一 `Runner`，并通过 `java -Xverify:all`。三方 stdout 逐字一致：六个包装类和值及 identity 都吻合，随后为 `0:0:7`、`true:false`、`null-unbox:NPE`。没有发现执行语义差距。

可确定的源码质量差距是六个固定形态：JADX 输出 `return 1;`、`return true;`、`return (byte) 2;`、`return (short) 3;`、`return 'c';`、`return 4L;`；Jarde 输出对应的 `Wrapper.valueOf(...)`。`javac` 对 primitive 在对应 wrapper/Object 返回上下文执行自动装箱，仍使用相同 wrapper boxing conversion；JLS §5.1.7 说明该转换的 identity 约束。fixture 还以缓存范围内相同 wrapper 引用检查这六种结果。Jarde 文本完整可编译且运行一致，因此这不是错误恢复，只是固定 JADX 明确覆盖的、可闭合的简化差距。

因此新建 [窄 OpenSpec](../../changes/debox-proved-wrapper-return-boxing/) 只提议：标准六种 wrapper 的 literal `valueOf(primitive)` 调用直接成为方法返回值，返回类型严格为对应 wrapper 或 `Object` 时，输出 primitive literal 并保留 `byte`/`short` 必要窄化 cast。没有为 overload、任意调用参数、变量流、条件合流、float/double、引用 cast、instanceof 或常规拆箱提出实现；这些形态需分别取得正向完整类证据后再立项。相关语言转换见 [Java SE 8 JLS §5.1.7](https://docs.oracle.com/javase/specs/jls/se8/html/jls-5.html#jls-5.1.7)。
