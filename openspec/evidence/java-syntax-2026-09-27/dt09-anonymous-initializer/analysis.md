# DT-09：匿名类实例初始化块首个隔离切片

固定 JADX 测试 `inner/TestAnonymousClass4.java` 断言匿名 `Thread` 体中的 `{ f = 1; }` 位于 `run()` 覆盖方法之前，且分配表达式末尾仍能调用 `start()`。该原测试同时牵涉 JDK 父类、外层私有实例字段、编译器 accessor 和非返回位置的分配。本切片只隔离实例初始化块：同包可直接拼写的 `Base`、唯一 `return new Base() { ... }`、无捕获且无字段的匿名 child、构造器在 `super()` 后执行一次静态 `int` 写入，覆盖方法随后执行另一写入。因此本切片通过也不等于完整测试追平。

`input/p` 是精确 Java 8 输入；`replay.py` 使用固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 和 Jarde CLI SHA-256 `7bd7ebe1fd0837cd8ca552380361c23a006d4bb3a7818ed21850c66f5e290678`，将原 class、JADX 完整源码和 Jarde 根类加物理 child 完整源码分别以 `javac --release 8` 重编，并用同一源级 Runner 在 `java -Xverify:all` 下执行。三者均输出 `1`、`8`；第二次独立重放与 `baseline/summary.json` 完全一致。`baseline` 保存输入和生成源码的 SHA-256、所有编译/运行结果与诊断。

JADX 在 `Subject.make()` 中输出 `new Base() { { Subject.value = 1; } @Override public void run() { ... } }`。Jarde 根类保留 `new p.Subject$1()`，物理 child 的 `<init>` 中保留 `super(); p.Subject.value = 1;`，因此行为未错，但源级匿名初始化块未恢复。根类报告以 `anonymous_child_constructor_ast_missing` 拒绝投影：现有匿名父类路径要求 `GenericConstructorCandidate` 给出纯转发构造证书，而该候选明确只接受 `super(...); return;`，无法表示 `super();` 后的初始化效果。证据支持的是**投影证明边界**，不是字节码解码或运行语义故障。

架构上应沿用同次物理 child 的 AST/Code/SSA 与当前类级唯一分配点、准确 `EnclosingMethod`、owner 引用普查、整棵源码原子投影。对构造器另证一个有界的“准确 `super()` + 可呈现的初始化语句 + `return`”形态，再在匿名体中先输出初始化块、后输出覆盖方法；不能把带效果构造器谎报成现有无效果泛型构造证书。若任一构造指令缺失、存在额外捕获或字段、效果位置不明、AST 不完整、预算停止，应保留物理 child 路径。`Thread`、非返回分配与外层私有字段仍独立排队，不并入首切片。
