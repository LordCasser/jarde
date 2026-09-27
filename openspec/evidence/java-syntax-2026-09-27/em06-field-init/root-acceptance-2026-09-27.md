# EM-06 静态初始化主线独立验收

root 审阅共享证明后，将实现合入主线；另让无 `<clinit>` 的普通类保持 `NotApplicable`，不把合法的默认值静态字段误报为待证明组，并将 module class 排除在普通类范围外。重新构建 CLI（SHA-256 `2d63a7da67cdacd1523e424d104170a8b6d74cb68ed8ff96dba253590b4e2c69`），独立运行固定 [replay.py](replay.py)。JADX 提交、五项测试和两份生产文件哈希一致；原 class、JADX、Jarde 的完整 Java 8 类源码均重编，`java -Xverify:all` 同为 `a:ab:abc:abc`、`sb2`。[root-replay/](root-replay/) 保存源码、日志和摘要。

Jarde 现按原 `putstatic` 顺序将 `trace/a/b/c/result` 五个 RHS 写到静态字段声明；依赖实例 `state` 的 `field = initField()` 仍在构造器，物理 `<clinit>` 方法和来源报告仍可查询。准入复用同轮候选、完整字段表、准确字段读、表达式效果与接口已有的原子 fragment/writer；缺失/重复写、额外顶层效果、前向读、ConstantValue、异常边和预算停止不投影部分字段。新加的无 `<clinit>` 默认静态字段测试确认不制造错误拒绝。

主线四项静态组测试、十项 `class_initializer_candidates`、85 项 `class_source`、完整 `jarde-java` 测试、workspace check、fmt 与 OpenSpec strict 均通过。接口原路径仍由候选测试和全套回归覆盖。实例字段共有初始化、异常初始化、混合 ConstantValue、继承字段与数组特例仍待各自验收。
