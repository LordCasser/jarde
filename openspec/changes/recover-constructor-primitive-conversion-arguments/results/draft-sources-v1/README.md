# 构造参数数值转换源码草稿 v1

这是待 root 审阅的完整 Java 8 源闭集，共两个顶层类：`ConstructorPrimitiveConversionControls` 与 `PrimitiveLongPair`。该目录仅为源码草稿；尚未编译、执行或放入永久 fixture。

主类覆盖 `Byte`、`Short`、`Long`、`Float`、`Double` 五类 wrapper 构造参数和 `Integer` 无转换对照；`boxedArray(int)` 另返回包含六种 Number wrapper 的 fresh `Number[]`，每个元素直接由带独立 tag 的构造参数产生，并由 main 逐下标显式观察运行时 wrapper 类型和值。另含普通 `return new` 与存储一次、作为两个构造实参读取的局部变量正控。`int`、`long`、`float`、`double` 四种 source-category mark 函数各自输出标签和值，供顺序及一次调用观察。十五个显式 primitive conversion opcode 都有对应 wrapper/`Character` 构造方法，并保留 `long → float → long`、`long → double → long` 和 `double → float → double` 的中间舍入链。

`main` 按固定顺序调用各方法，使用直写 `println` 观察 wrapper 值及 `instanceof` 类型结果；没有循环、条件表达式、字符串拼接、反射类名处理或运行时 evaluator。`storedLocalReuse` 只调用一次有副作用的 mark，随后两次转换同一已存局部值进入 `PrimitiveLongPair` 构造器。

所有 target/helper 源都在本目录：root 审阅后如需冻结，应将两个源作为一个完整输入闭集，以真实 Java 8 target 编译；不能借用原 class 或只抽取已恢复方法。
