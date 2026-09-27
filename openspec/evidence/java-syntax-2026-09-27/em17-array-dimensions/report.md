# EM-17：数组维度与元素类型首片对照

固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `TestNewArrayOfArrays`、`TestMultiDimArrayFill`、`TestArrays3`、`TestArrays4` 给出本次 Java 8 class 输入方向：`new long[n][]`、`new String[n][]`、`new int[n][][]`、`new int[a][b]`、嵌套不规则初值、`byte[]` 作为 `Object[]` 的单个元素，以及无调试信息时 `char[]` 局部类型。四个固定测试文件的 SHA-256 由 [replay.py](replay.py) 核对。

[input/em17/](input/em17/) 缩成一份完整的 `Shapes` 类和共同 `Runner`，以 `javac --release 8 -g:none` 编译。重放分别提取固定 JADX 与 Jarde CLI (`8f1f0012324350e4fc65c7fef4b3e3835102e6d4fda00df280b466270c98e727`) 的完整 `Shapes` 源码，与原始 `Runner` 组成编译单元。原/JADX/Jarde 都通过 Java 8 重编和 `java -Xverify:all`，运行输出逐字一致；独立第二次重放的摘要和两份反编译源码逐字节相同。源码、版本、编译与运行日志、哈希在 [acceptance/](acceptance/)：

```text
2:true
3:true
4:true
2:3
[[1], [4, 5], []]
true
2
```

Jarde 保留了四种维度/元素类型组合和完整嵌套初值；`wrapped(byte[])` 输出 `new Object[]{arg0}`，运行时内部元素仍与原 `byte[]` 同一引用。无调试信息的构造器正文仍输出 `char[] local2 = toChars(arg1)` 与 `new char[local2.length]`，不会把 `char` 当作 `byte`。这里已有 `Operation::NewArray` 的元素/总维数事实、`ExprKind::NewArray` 和 emitter 的维度写法；嵌套初值沿用 EM-18 的闭合数组链证明，不需新增生产机制。

EM-17 只移动到“部分已测”：动态维度表达式的副作用顺序、更多维度/类型、别名逃逸和其他控制流位置仍待逐项核验。本首片没有发现需要独立 OpenSpec 的实测差距。
