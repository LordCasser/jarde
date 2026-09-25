# 匿名类合成捕获不能作 `new Base(...)` 实参

源码是自行编写的 Java 8 `AnonymousProbe.java`：方法内局部 `captured=base+1` 与外层 `state` 被匿名 `Action` 实现捕获，`main` 应输出 `10`。`javac 23.0.1 --release 8 -g:none` 产生本目录三份冻结 class；`java -Xverify:all` 输出 `10`。`run_audit.py` 在临时目录重编、核对每份字节哈希、执行原 class，并分别尝试编译冻结的 JADX/Jarde 完整外层类输出，结束自动清理临时文件。

2026-09-25 使用安装的 JADX 1.5.6（本地源码 `2fb1b16386941660fda07e9017285aec40fcb37f`）处理三份 class 的 jar。其输出虽然把匿名体放进使用点，却写成 `new Action(this) { ... }` 与体内 `this.this$0 = this`：`javac --release 8` 分别报“匿名类实现接口; 不能有参数”和匿名 Action 不能转为 `AnonymousProbe`。对照本地 `ProcessAnonymous` 的唯一构造/使用点条件及 `AnonymousClassVisitor.getArgsToFieldsMapping`：后者沿 SSA 一次使用追踪合成字段存储和 `super`，这一步值得借鉴；`startArg` 只因首参类型匹配外层类而跳过，不等于已证明源码中的外层 `this` 或构造实参可放到接口匿名类调用。

当时 Jarde CLI SHA-256 `fbd01aebdb6093800f95ed6b52f576aa3d7a9b61a757021ae373b828b50b1d61`（共享工作树，不能以仓库 HEAD 代替构建身份）在外层类写 `new AnonymousProbe$1(this, local2)`，内层类保留 `val$captured` 与 `this$0` 字段。Jarde 明确声明类源码“not claimed to compile”；若尝试用原 class 作 classpath 编译外层源码，`javac` 会把匿名构造器的合成参数隐藏为源级无参，拒绝该调用。这是尚未内联的诚实边界，不是 Jarde 的执行等价正例。

该反例推翻 [present-proved-java-structure 的原 5.3 条件](../../../changes/present-proved-java-structure/design.md)：无名、唯一使用点、方法体完整仍不足以输出匿名语法。修订后的 5.3 要求同一物理环境中调用者所有方法仅有一处分配点、全部待内联方法 `quality=structured` 且无引用缺口，并证明唯一构造器参数与真实 `super` 参数/合成捕获字段的映射、每次字段读取的稳定词法替换及创建时求值顺序，再决定能否内联。真实 `Outer.this` 只有在值流证明分配点捕获的就是当前词法外层接收者时才可写；同型参数 `other` 不能被当成外层 `this`。5.3 还要求独立构造基类实参、同型非外层对象、混合 fallback 正文、实例初始化副作用的 Java 8 正负例，避免只在本反例上拟合。
