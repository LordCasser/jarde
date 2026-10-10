# EM-17/18 数组 JavaInput 正向覆盖复核 v2

只读复核当前 JADX 数组测试与既有完整类记录；没有运行工具链，也没有改动 v1 清单。

`TestArrays.test2` 位于上游 [TestArrays.java:16-19](</Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/arrays/TestArrays.java:16>)：局部 `int[][] a = new int[i][i + 1]` 后返回 `a.length`。该文件的唯一 `@Test` 在 [23-28 行](</Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/arrays/TestArrays.java:23>) 只断言 `test1` 的字面量索引表达式，所以 `test2` 不是这个 JUnit 测试的独立字符串断言。

不过，这个分配形状已有真实完整类三方对照，不只是 `new int[a][b]` 的近似覆盖：EM-19 [Access.java:7-10](../java-syntax-2026-09-27/em19-array-access/input/em19/Access.java) 同样执行 `int[][] values = new int[i][i + 1]; return values.length;`，Runner 在 [Runner.java:6](../java-syntax-2026-09-27/em19-array-access/input/em19/Runner.java) 调用 `dimensions(2)` 并观察长度 `2`。其[报告](../java-syntax-2026-09-27/em19-array-access/report.md)记录原类、JADX、Jarde 完整源码均通过 Java 8 重编与 `-Xverify:all`，三方输出一致。差异仅为样例方法/接收者形态（EM-19 方法为 `static`，上游 `test2` 是实例方法）；没有证据显示它引入了不同数组分配语义，因此不建议为此重复建 fixture。若目标是逐字追平 JUnit 声明本身，仍需注意 test2 当前没有独立断言。

作为对照，[EM-17 Shapes.java:14](../java-syntax-2026-09-27/em17-array-dimensions/input/em17/Shapes.java) 的 `full(int a, int b)` 只覆盖两个独立参数；单独它不能证明 `i` 与 `i + 1` 形状。该缺口由上面的 EM-19 `dimensions(int i)` 记录补上。[EM-17 报告](../java-syntax-2026-09-27/em17-array-dimensions/report.md)仍可作为多维/嵌套初始化其他形状的完整类证据，不需要再重复 `TestNewArrayOfArrays` 已映射的四种方法。

EM-18 的活动 byte 字面量直接返回由已接受的 [byte-array-return baseline](byte-array-return-next/README.md) 覆盖；该基线的 `baseline-root-v2/manifest.json` 闭合原/JADX/Jarde 十个完整源码编译运行腿，root 的独立 verifier 已接受。用户指定已覆盖的 `TestArrays2.test4`、`TestArrayFillConstReplace`、`TestArrayFill4` 与 `TestArrayFillNegative` 不重复列为候选。`TestArrayFill2.test2` 标记 `@NotYetImplemented`，Smali/Dex 输入也不算 JavaInput 正向缺口。

71 单元账本仍将 EM-17/18/19 标为“部分已测、仍待扩验”：[summary.md:71-73](../jadx-feature-inventory-2026-09-27/summary.md:71)。这里确认的是本轮限定的活动 JavaInput 正向 shape 已有完整类对照，不把局部闭合扩大成单元完成声明；EM-18 的 Dex `fill-array-data` 等账本缺口不在本轮 JavaInput 范围。

本次没有找到值得新增的活动 JavaInput 正向目标；不提出新 fixture 或实现范围。
