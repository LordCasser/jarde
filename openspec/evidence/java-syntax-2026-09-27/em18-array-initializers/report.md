# EM-18：Java 8 数组初始化首片对照

固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `TestArrayFill`、`TestArrayFill2`、`TestArrayFillNegative` 与 `TestArrayTypes` 提供本次 Java class 输入的正反形态。`TestArrayFill2.test2` 明确标为 `@NotYetImplemented`，不能当作 JADX 已恢复 `a++` 的正向证据。`TestArrayFillWithMove`、`TestFillArrayData` 是 Smali/Dex 形态，尚未由这次 Java 8 class 重放覆盖。

[replay.py](replay.py) 校验固定测试哈希，从 [input/em18/](input/em18/) 以 `javac --release 8 -g:none` 构建原始 jar，再分别用固定 JADX 和 Jarde CLI (`8f1f0012324350e4fc65c7fef4b3e3835102e6d4fda00df280b466270c98e727`) 生成 `Arrays` 完整源码。各版与同一个原始 `Runner` 重编，并以 `java -Xverify:all` 运行。源码、编译/运行日志、工具版本及 SHA-256 均在 [acceptance/](acceptance/)；独立第二次重放的摘要与两份反编译源码逐字节一致。三方输出逐字一致：

```text
[1, 2, 3]
[1, 3, 2]
[1, 2, 6]
[1, 2, 3]
1
```

Jarde 对 `new String[]{...}`、`new int[]{1, a + 1, 2}`、`use(new Object[]{e})` 均写完整初始化表达式；`arr[1] = arr[0] + 1` 继续保留逐步写入，没有错误折叠自读数组。固定 JADX 也保留自读数组写入。对于 `new int[]{1, a++, a * 2}`，Jarde 保留 `arg0++` 的源级副作用位置，JADX 输出 `new int[]{1, i, (i + 1) * 2}`；两者运行语义一致，但 Jarde 在这个尚未由 JADX 测试完成的语法点上保留了更直接的源形态。

Jarde 现有 `ArrayInitializers::prove` 消费 SSA、操作和效果事实，在同一块内检查从分配到有序写入的闭合链、索引、别名、异常处理和唯一消费者；`prove_local_postfix_element` 是其中受限的 `int` 局部后缀证明，不需要为本首片新增生产机制。JADX `ReplaceNewArray.processNewArray` 按数组用途收集、排序写入，并用 `verifyPutInsns` 检查单块/数组实参使用；这解释了其正例方向，但不能替代 Jarde 对元素求值顺序与副作用的证明。

EM-18 只移动到“部分已测”：Dex `fill-array-data`、稀疏/缺项初值、移动后的数组别名、多维/嵌套数组和更多求值次序仍按固定队列逐项核验；本次没有观测到需要开新 OpenSpec 的 Java 8 class 差距。
