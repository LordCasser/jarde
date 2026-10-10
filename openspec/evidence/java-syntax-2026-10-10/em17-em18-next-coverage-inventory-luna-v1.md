# EM-17 / EM-18 下一步覆盖盘点（只读）

本盘点只对照 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的数组测试源码、71 单元账本和已归档的完整类证据。没有执行 JDK、JADX、Jarde 或构建命令。本文件列的是最多两项值得继续确认的测试形状，不声称它们已经失败，也不提出新的生产机制。

## 已有边界

- 71 单元账本把 EM-17 标为部分已测：已有维度/类型 Java 8 完整类三方重编运行证据，更多维度和副作用表达式待测；EM-18 同样部分已测，字符串、计算元素、对象数组实参以及自读数组写入已有完整类证据，Dex fill-array-data、别名和其他求值次序待测。见 [`summary.md:71-72`](../jadx-feature-inventory-2026-09-27/summary.md:71)。
- EM-17 的完整类 [Shapes.java](../java-syntax-2026-09-27/em17-array-dimensions/input/em17/Shapes.java) 已覆盖 `new long[n][]`、`new String[n][]`、`new int[n][][]`、`new int[a][b]`、不规则嵌套初值、`byte[]` 包装和无调试信息下的 `char[]`。其[报告](../java-syntax-2026-09-27/em17-array-dimensions/report.md)记载原/JADX/Jarde 完整源码均 Java 8 编译并 `-Xverify:all` 运行一致。
- EM-18 的完整类 [Arrays.java](../java-syntax-2026-09-27/em18-array-initializers/input/em18/Arrays.java) 已覆盖字符串/int 完整字面量、int 计算元素、后缀自增元素、自读逐项写入和 `Object[]` 中的引用。其[报告](../java-syntax-2026-09-27/em18-array-initializers/report.md)记载语义三方一致；`TestArrayFill2.test2` 对应的后缀形在上游仍标 `@NotYetImplemented`，不能当作 JADX 正向合同。
- `TestArrayInit.test2` 的数组写入实例字段已有 array-field-store 证据；纯字面量 static/instance 字段另有 [ArrayFieldLiteral baseline](../java-syntax-2026-10-10/array-field-initializers-literal/README.md)。近期跨构造器实例字段初始化覆盖属于独立 slice，不作为本清单的新数组语法。
- `TestArrayFill4` 的 long 极值、`TestArrayFillNegative` 的依赖写入与边界反例、`TestArrays2.test4` 的 primitive 分支，以及多维/嵌套数组既有工作不重复列为候选。`TestArrays.test1` 的 `new int[]{...}[i]` 已由 EM-19 的 [Access.java](../java-syntax-2026-09-27/em19-array-access/input/em19/Access.java) 和[报告](../java-syntax-2026-09-27/em19-array-access/report.md)覆盖。

## 建议顺序

1. **EM-17：多维分配长度表达式的可观察求值次序。** 上游锚点是 [TestNewArrayOfArrays.java:23](</Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/arrays/TestNewArrayOfArrays.java:23>)：`multiAnewarray(int a, int b)` 只用普通参数，未让维度求值顺序可观察。建议以该方法为锚扩成一个完整小类：两维分别调用 `dimension(1)`、`dimension(2)`，helper 记录 trace 并返回给定长度；Runner 对照 trace、两维长度和零长度分配。它直接补上账本写明的“副作用表达式”空白，不改变目标类型/嵌套初值。首选复用现有维度/`multianewarray` 恢复路径；只有真实完整类对照显示拒绝或次序不同，才决定是否需要实现工作。当前证据不能证明该路径缺失。

2. **EM-18：活动的 byte 字面量直接返回。** 上游正向锚点是 [TestArrayFill3.java:11](</Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/arrays/TestArrayFill3.java:11>)：`byte[] test() { return new byte[] { 0, 1, 2 }; }`，且 `@TestWithProfiles` 明确覆盖 `ECJ_J8` 与 `ECJ_DX_J8`。现有 EM-18 完整类有 String/int 初值，byte 的新证据集中在字段初始化或 `TestArrays2.test4` 分支返回 `Object`，没有找到以此独立方法签名为主的完整类三方重编记录。最小输入就是该 `TestCls` 和 runner，runner 检查返回字节、重复调用得到的数组 identity 不同。预期复用现有 typed `NewArray`/有序元素初始化；profile 本身不表示 Smali/Dex 输入，也不应据此扩展到 fill-array-data。

`TestArrayInit.java:13-17` 还有一个“创建后未使用”的局部 byte 数组正断言，但该分配不可从运行行为观察；是否保留 dead local 是另一种源码保真合同。若目标仅是语义与可编译性，它不应优先于上面两项，也不能把省略它记为数组行为缺陷。

## 边界判断

`TestArrayInitField2`、`TestFillArrayData` 和 `TestArrayFillWithMove` 是 `SmaliTest`/DEX 输入，不是 Java 8 class 文件测试。当前 classfile 回放不能直接把它们当成新的 Java source/class 对照；账本已把 Dex fill-array-data 留作 EM-18 待测，但那需要另一个明确的输入边界，而不应混进上述最小 Java 语法覆盖。

此次检索没有证据支持宣称 EM-17/18 已无缺口，也没有证据支持把这两项候选宣称为 Jarde 失败。推荐先闭合第 1 项的完整类行为次序，再闭合第 2 项的类型/签名正控制；之后依据实测输出决定是否需要规格或实现变更。
