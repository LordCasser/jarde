## Why

Java 8 的 `new int[][] {{f(1), f(2)}, {f(3)}}` 会产生逐层 `newarray/anewarray; dup; index; …; *astore`，当前 Jarde 对两层都没有认领，导致完整类的两个真实方法缺少返回语句。JADX 1.5.6 在该正例可重编并保持行为，但同一折叠算法会把乱序写入的有副作用元素重排，必须以 JVM 执行顺序而非 JADX 输出作为准入标准。

## What Changes

- 扩展现有数组初始化证明，让同块、按物理索引顺序、身份与组件类型均可证明的内层数组成为外层初始化器的元素，写出嵌套 `new T[][]{{…}, {…}}` 等等价 Java 表达式。
- 父子链各自完整认领 BCI、异常处理集合及预算；任一层证明不足时不提交部分嵌套表达式。
- 保留乱序索引、额外使用、跨块与无法证明的引用数组协变写入的拒绝边界。JADX 的排序实现仅作为参考，不复制其语义错误。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已证明的多维嵌套数组初始化链可恢复为 Java 8 数组表达式。

## Impact

只涉及 `jarde-java` 已有 `ArrayInitializers` 证明、`NewArray` 呈现和发射，以及对应的 Java 8 fixture、来源与执行对照；不新增 crate、全局别名/层级分析或 CLI 协议。以既有一维数组初始化与部分维度分配为前提；不宣称支持任意数组修改、未知组件转换或 JADX 的不保序折叠。冻结正反例及原/JADX/Jarde 对照见 `../../evidence/java-syntax-2026-09-25/nested-array-initializer/analysis.md`。
