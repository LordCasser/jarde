# EM-06 实施验收

普通类的同轮 `<clinit>` 候选现在复用接口字段组证明及原子 fragment 发射。固定 `FieldOrder` 的 `trace/a/b/c/result` 按五次 `putstatic` 顺序成为字段声明初值；`state`、`field` 与构造器调用位置不变。字段与 `<clinit>` 的物理报告、BCI 来源仍保留。

使用固定 [replay.py](../../../evidence/java-syntax-2026-09-27/em06-field-init/replay.py) 验证原 class、固定 JADX 和 Jarde 的完整源码。三份源码均通过 `javac --release 8`，并通过 `java -Xverify:all` 输出相同两行：`a:ab:abc:abc`、`sb2`。加强的回放断言同时核验五个声明初值严格有序、无已消费的 `static` 块，实例赋值仍在构造器。Jarde 源码 SHA-256：`c624615d39f99c8fc3721ae0b8887eb6f2232c873b640e1bcdaf6539dd64a14a`。

定向 Rust 测试覆盖缺失、重复写、额外顶层调用、前向读、ConstantValue、异常边、输出/IR 预算及取消。接口投影的 6 项回归、类源码的 84 项测试、`jarde-java` 全套测试、`cargo check --workspace`、`cargo fmt --all -- --check` 与 `openspec validate ... --strict` 均通过。类名多定义及成员表截断继续由既有类源码测试覆盖。

剩余形态保持当前拒绝边界：实例字段初值移动、异常初始化、ConstantValue 混合、前向读、继承字段和数组特殊语法。本片没有为这些形态声明源码投影证明。
