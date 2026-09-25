# Java 8 字符串 switch 的源码形态审计

`run_audit.py` 用 `javac --release 8 -g:none` 编译本目录源码，冻结 class 为 635 B、SHA-256
`258fea472bfd8f80ee49726dc25f0c3d46dce8f61db57d43d5f63e52e6c2c53c`，并以 SHA-256
`908c472560d6146354e1fd679c595e884afadf5ec7051727b51e79a1f95c6570` 的 CLI 生成
jarde 完整类。原 class、JADX 完整源码、jarde 完整源码都实际通过 Java 8 重编译、
`java -Xverify:all` 与 10 行运行对照；两份 diff 为空。输入覆盖 `"Aa"` 与 `"BB"` 的
相同 hash、第三个普通 case、default、null 与一次有副作用的 selector 包装。

JADX 恢复 `switch (str)`，jarde 则忠实写出编译器的 `switch (local1.hashCode())`、
`String.equals` 判别、整数 discriminator 赋值，以及第二个 `switch (local2)`。
因此这里没有已证实的执行错误或编译错误；缺口是源码语法质量。不能把两级文本简单
合并：碰撞分支、null 的 `NullPointerException`、selector 只求值一次及每个整数 case
的目标均必须证明。`javap.txt` 显示同一方法内的两个 switch 与三次 equals，所有映射
证据都在本方法的 SSA/常量池/CFG 内，不需要外部类路径或新全局类型系统。

最小可行架构是给现有 switch 区域做一个**局部、可拒绝的 lowering 形状认领**：确认
String selector 的单次保存、`hashCode()` 的 null 行为、各 hash bucket 中完整且互斥的
字符串常量比较，以及 discriminator 的唯一赋值到末级 switch 的 case 目标。认领成功后
现有 switch 语句可继续承载各 arm；AST 的 case label 从纯整数扩展到整数或已验证的
字符串常量，来源合并两个分派及原 arm。原 Region 的原始整数键可以保留供证明使用。
这需要新的**局部形状证明与标签类型**，但无需通用反混淆器、外部继承求解或第二套
控制流框架。未证明的变体继续保留目前可执行的两级 Java，不以“看起来像 javac”
作为改写依据。

依据：[JLS 8 §14.11](https://docs.oracle.com/javase/specs/jls/se8/html/jls-14.html#jls-14.11)
允许 `String` selector，禁止 null case，并规定引用 selector 为 null 时抛异常。
