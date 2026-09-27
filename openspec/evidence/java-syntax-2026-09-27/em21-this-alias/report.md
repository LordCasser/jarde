# EM-21：`this` 局部别名与分支值流对照

固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `usethis/TestInlineThis`、`TestInlineThis2` 和 `TestDontInlineThis`：前两者要求消去只指向 `this` 的别名，第三者要求保留可在 `this` 与新对象之间选择的局部变量。`TestRedundantThis` 的唯一 `@Test` 已被注释，不能算正向测试。四份测试文件的 SHA-256 由 [replay.py](replay.py) 核对。

[input/em21/](input/em21/) 是 Java 8、无 debug 信息的完整类与共同 Runner。修前重放的原 class、固定 JADX、Jarde 完整源码都以 Java 8 重编，并在 `java -Xverify:all` 下得到相同结果；Jarde 基线保留别名的源码、编译日志和摘要保存在 [baseline-before-fix/](baseline-before-fix/)。

实现后，声明计划以现有 `LocalVariable` 身份建立一次窄证书：只接纳一个非参数非槽 0 局部、一次 `Store`、直接 `aload_0` 的 SSA 来源，以及逐一覆盖的直接读取；读取只能是受支持的实例调用/实例字段接收者或精确 `Objects.isNull(Object)` 实参。读取闭合还容许 SSA 已证明并替换为同一值的 trivial phi；多写入、非平凡 phi、额外用途、复用身份不明、不可呈现区域和不支持的消费者都保留旧投影。计划在生成 AST 之前完成，预算和取消错误直接终止构造；输出阶段只跳过证书中的 store，并把证书中的 load 表达为保留原 BCI 的现有 `ExprKind::Local("this")`。没有文本替换或新增 pass/AST。

固定脚本 `python3 replay.py fixed ...` 对原 class、固定 JADX 与 Jarde 完整源码执行 `javac --release 8`，再用 `java -Xverify:all` 运行共同 Runner。Jarde 在 `inline`/`checked` 消去两个只读 `this` 别名并保留字段写入、调用、分支和空值检查；`choose` 的 `this`/新对象双来源仍保留两臂赋值、调用及返回局部。三个版本的 stdout 完全一致：

```text
123:1
123:2
true:3
true:1
```

固定验收摘要、三方编译/运行日志和生成源码位于 [acceptance-fixed/](acceptance-fixed/)；修后 CLI SHA-256 为 `82a7f6633cae3c2fc2a58af52f3cd4fe9c20dec687f577806dfd3bb22f736df0`。Rust 回归测试另验证直接字段读取也恢复为 `this`，并用 Java 8 反例覆盖复赋值、参数/返回值逃逸、无读取的写入，以及复用槽位的拒绝；后者在身份/类型布局不足时沿用原拒绝。固定 JADX `TestRedundantThis` 的 `@Test` 仍是注释状态，只核对哈希和注释，不作为通过测试。固定回放也把 `output_bytes=1` 的停止作为负例：CLI 不返回源码，避免发布部分类文本。

边界仍限定在当前证据的普通实例方法读取形态。继承字段遮蔽、构造器、别名逃逸或复赋值、未覆盖的调用/字段位置、任意复制传播和槽复用的独立夹具尚未验收；这些形态没有因此获得支持。
