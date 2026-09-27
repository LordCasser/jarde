# EM-05：构造链首片

固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的五项相关测试和 `ConstructorVisitor`、`PrepareForCodeGen`、`InsnGen` 源码哈希由 [replay.py](replay.py) 锁定。`TestConstructorBranched`、`TestInsnsBeforeSuper2`、`TestInsnsBeforeThis`、`TestConstructorWithMoves` 是 Smali 输入；其中 Java 8 源码不能表示任意对未初始化 `this` 的早期操作，不能把测试文本断言直接当成可重编的 Java 8 正向证据。`TestDefConstructorNotRemoved` 的多成员声明又与 EM-01 家族装配交叉。本首片隔离为合法的顶级 `Base/Child/Factory` 三类，覆盖无参/有参 `new`、分支构造、`super()`/`super(String)`、调用静态 `Integer.toString` 后的 `this(String)` 链。

[baseline/summary.json](baseline/summary.json) 记录原 class、JADX、Jarde 三份**完整三类源码**均用 `javac --release 8` 重编，并以 `java -Xverify:all` 运行同为 `a`、`b`、`c`、`5`。Jarde 保留了两条分支各自的构造调用、正确的 `super` 与 `this` 链；额外显式 `super();`、构造器尾 `return;` 和静态调用结果的 `(String)` cast 不改变此输入行为。JADX 把简单的分支局部合流写成条件表达式，这是控制流/表达式呈现的交叉质量点，不应混进 EM-05 构造目标证明。

Jarde 已在 `init@1` 证明构造器首个 `this`/直接父类 `<init>` 身份，在 `new@1` 对分配、复制、准确构造调用及有序实参作同轮证明；本首片没有发现需要新机制的构造链差距。EM-05 仍需扩验 Smali 的异常栈形、独立于 EM-01 的多成员默认构造器，以及构造链前副作用能否被 Java 8 合法表达。没有具体三方失败前不发实现任务。
