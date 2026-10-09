# 构造引用数组元素 fixture 计划 v1

本 fixture 只覆盖 fresh reference-array initializer 中的 inline constructor element 与 `aastore` 组合，不包含 primitive 转换、alias 控制或非法 Java 负例。整个闭集为 `Main`、`Base`、`Mid`、`DirectA`、`DirectB`、`TwoHop`、`LocalInterface`，所有 top-level 类均从源文件编译进每条腿的同一 classes 目录。

`Main` 提供六个 fresh initializer：`sequence()` 返回 `CharSequence[]` 并直接构造 `StringBuilder`、`StringBuffer`；`collections()` 返回 `Collection[]` 并直接构造 `ArrayList(Arrays.asList(...))`、`HashSet(Arrays.asList(...))`；`failures()` 返回 `Throwable[]` 并直接构造 `IllegalStateException`、`IllegalArgumentException`；`ownDirect()` 返回 `Base[]` 并存入直接子类 `DirectA`、`DirectB`；`ownTwoHop()` 返回 `Base[]` 并存入经 `Mid` 继承的 `TwoHop` 与直接子类；`ownInterface()` 返回 `LocalInterface[]` 并存入直接实现及两跳继承实现。每个方法恰有两个由 `new` 产生的数组元素，且不先写入共同上界局部变量。

每个数组元素的构造参数都调用 `Main.mark(tag)`，包括 Collection 构造器的内联 `Arrays.asList(mark(...))` 参数。mark 输出带标签的事件；自有层级构造器在 `super` 完成后输出带类名及同一标签的事件，因此原始运行能逐行核验参数先于构造器、每个数组的第一个元素先于第二个元素。平台构造器的执行次序由每个元素唯一、顺序的 mark 事件观察；运行输出不含 identity hash、时间或随机值。

`Main.main` 依次调用六个方法，并用一个 observer 读取每个已知非空数组的两个元素。observer 将每个 `values[index]` 先赋给 `Object` 局部，再以简单 `if/else` 打印 null 或运行时类名；它不使用循环、三元、数组长度、字符串拼接或异常路径。此 observer 仅让原程序效果和运行时类型可见，不承担构造元素证明。

每个 JDK 腿从同一十源文件集构建。Corretto 8 使用 `-source 8 -target 8 -g:none`；OpenJDK 23 使用 `--release 8 -g:none`。两个命令都显式传入空 `-classpath` 与空 `-sourcepath`。原始程序以 `-Xverify:all` 和该腿新建的完整 class 目录运行。runner 保存真实 argv、退出码、stdout/stderr 原始字节及 SHA-256、所有源/class 哈希，并保存 `javap -c -p Main` 原文及每个目标方法的 `new`、`dup`、`invokespecial`、`aastore` BCI 清单。
