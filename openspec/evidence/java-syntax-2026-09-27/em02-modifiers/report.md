# EM-02：接口修饰与覆写可见性的首片审计

固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的四个测试及哈希见 [replay.py](replay.py)。`TestInterfaceDefaultMethod` 对 `default`、`static` 和抽象接口方法有活动文本断言；`TestOverridePrivateMethod` 不允许给私有同名方法写 `@Override`。`TestOverridePackagePrivateMethod` 是 Smali 测试，明确要求同包真实覆写有 `@Override`、跨包包私有同名方法没有。`TestBadMethodAccessModifiers` 也是 Smali 的非法可见性修复测试，不能把它当作合法 Java 8 `javac` 正向例子。

[input/](input/) 构造合法 Java 8 的六个完整目标类及一个共同 Runner：接口抽象/default/static；子类与父类私有同名方法；同包覆写父类包私有方法；跨包同名但不覆写的方法。使用固定 Jarde CLI SHA-256 `8f1f0012324350e4fc65c7fef4b3e3835102e6d4fda00df280b466270c98e727`，原 class、固定 JADX、Jarde 六份目标类的完整源码均通过 `javac --release 8` 和 `java -Xverify:all`；三方 stdout 相同：

```text
7:5
1:2:1
2:1
```

源码、编译和运行日志以及 SHA-256 保存在 [baseline/](baseline/)。Jarde 正确保留接口 `default`/`static`，且没有给私有或跨包包私有同名方法误加 `@Override`。唯一已证质量差距是同包 `PackageChild.onlyHere()`：固定 JADX 写 `@Override`，Jarde 不写。`@Override` 是 SOURCE 保留注解，class 文件不记录原始拼写，因此这不是丢失物理注解，而是 JADX 根据父方法关系提供的安全源码提示。原 class 的合法动态派发与 Jarde 运行结果已一致；不应把没有注解说成运行错误。

架构上，`src/class_source.rs` 的物理方法声明只表达本 class 的 flags/attributes，不应直接猜一个跨类关系。`src/facade.rs` 已有请求内依赖类读取与成员家族证明入口，可在同一物理定义选择与预算下，只为**同 jar 直接父类、同包、准确同名同描述符、父方法为非 private/static/final 的可覆写实例方法**作一个窄投影证书，然后在最终类文本中给该方法写 `@Override`，同时保留原始方法事实。父类缺失/歧义、跨包包私有、private/static、构造器、桥/合成、签名替换、未知字节码或预算停止都保持原文本。接口实现、泛型/协变返回、多级继承、非法 access flags 和 Android/Smali 路径各自需要额外证明，不由此首片顺带覆盖。JADX `OverrideMethodVisitor` 的 `isMethodVisibleInCls` 可作可见性条件参考，但 Jarde 应坚持精确父定义与成员身份，不依赖方法名碰撞启发式。
