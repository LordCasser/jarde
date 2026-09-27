# EM-21：`this` 局部别名与分支值流对照

固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `usethis/TestInlineThis`、`TestInlineThis2` 和 `TestDontInlineThis`：前两者要求消去只指向 `this` 的别名，第三者要求保留可在 `this` 与新对象之间选择的局部变量。`TestRedundantThis` 的唯一 `@Test` 已被注释，不能算正向测试。四份测试文件的 SHA-256 由 [replay.py](replay.py) 核对。

[input/em21/](input/em21/) 是 Java 8、无 debug 信息的完整类与共同 Runner。原 class、固定 JADX、Jarde CLI (`8f1f0012324350e4fc65c7fef4b3e3835102e6d4fda00df280b466270c98e727`) 的完整源码均以 Java 8 重编，并在 `java -Xverify:all` 下得到相同结果。独立第二次重放的摘要及两份反编译源码逐字节相同；运行、编译日志与源码在 [acceptance/](acceptance/)：

```text
123:1
123:2
true:3
true:1
```

JADX 将单次 `this`→局部的 `inline`/`checked` 改写为直接 `touch()`、`this.field`、`Objects.isNull(this)`。Jarde 保留 `local1 = this`，并用 `local1.touch()`、`local1.field`、`Objects.isNull(local1)`；它更接近原源码，且行为正确，但比固定 JADX 多一个没有源级价值的别名。在 `choose` 中，局部可能来自 `this` 或 `new ThisUse()`：JADX 和 Jarde 都保留局部及两臂赋值。不能将它照搬简单正例全局替换为 `this`，否则会把返回对象身份和 `touch()` 的接收者改错。

这属于**源码简化质量差距**，不是 Jarde 不支持 `this` 语法，也不是本首片的编译/运行失败。Jarde 的 `jarde-java::build` 已有局部变量 `reuse` 身份、SSA 值和声明布局，`StmtKind::Declare/Assign` 与 `ExprKind::Local` 已能正确输出原始形态；优化应在同轮 builder 内证明某个非参数局部只有一次从槽 0 接收者赋值、全部读取仍指向该值，再原子省略该声明并在读取点呈现 `this`。不能通过文本替换、字段名或仅凭 `this` 出现在某个赋值中决定。无需新 pass、AST 或 JVM IR 机制；[窄 OpenSpec](../../../changes/inline-proved-this-local-alias/)记录正反边界。

EM-21 移到“部分已测”：这一首片尚未覆盖继承字段遮蔽、构造器中的 `this`、别名的逃逸/复赋值以及其它调用位置；`TestRedundantThis` 的禁用断言不当作已通过依据。
