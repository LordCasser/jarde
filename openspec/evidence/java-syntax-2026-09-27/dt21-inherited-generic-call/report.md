# DT-21：泛型继承与 bridge 的两种不同失败

固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `TestTypeVarsFromSuperClass` 在无调试信息时断言继承调用结果前没有 `(String)`；`TestGenericsMthOverride` 要求四种泛型 override 头；`TestSyntheticOverride` 来自 Smali 且明确禁用编译，不能作为可重编正例。[replay.py](replay.py) 用 Java 8 `-g:none` 编译 [Hierarchy.java](Hierarchy.java)：`C2<B> extends C1<B>`、`C3<C> extends C2<C>`、`Hierarchy extends C3<String>`，方法 `test()` 调用继承的 `call()`；[Runner.java](Runner.java) 同时验证运行值和 `Hierarchy` 的泛型父类反射。

原始完整源码重编并在 `-Xverify:all` 下输出 `true`、`dt21.C3<java.lang.String>`。Jarde 四个物理类源码也能 Java 8 重编，运行值仍是 `true`，但 `C2/C3/Hierarchy` 的参数化父类签名全部退化为 raw，反射变为 `class dt21.C3`；`test()` 保留 `(String)`，child 的泛型 `B call()` 因类头未投影而没有作用域。生产拒绝原因明确是 `class_generic_source_unproved: parameterized or nested parent needs a separate inherited-member proof`，因此这是声明与继承绑定的实质差距，不能凭运行值通过计分。

这次隔离 jar 中，固定 JADX 能写出 `Hierarchy extends C3<String>`，却把原 class 的 synthetic bridge `Object call()` 也写为显式方法。其源码与泛型父类的 `String call()` 冲突，完整 Java 8 重编失败；`outputs/results.json` 保留英文编译诊断。这个结果不能误记成“JADX 已完全追平 DT-21”，也不能把 Jarde 的可编译 raw 源码记成语法恢复完成。JADX 测试对单个 `TestCls` 文本的断言与此处四个顶级 class、当前 javac 版本的完整重编不是同一证据级别。

架构上应拆成两个切片。先对一个准确选定的普通 `Parent<T>`/`Child extends Parent<String>` 恢复参数化父类头，利用 reader 已有 class `Signature` 解析/擦除和选定物理父定义证明泛型形参与实参，完整源码重编和反射验证；不借方法 body 猜 T。再对本例的多层继承、`call()` 类型替换和 bridge 做原子闭包：替代显式 bridge 只有在完整证明其物理转发与 Java 编译器会由泛型父类自动再生成时才可隐藏；同时保留物理来源、方法分派和异常/效果。此顺序可参考 JADX `TypeUtils.replaceClassGenerics`、`TypeBoundInvokeAssign`、`OverrideMethodVisitor` 的类型变量映射，但 Jarde 应以已选 classfile 及调用/bridge 证据证明，不照搬 JADX 的错误 bridge 输出。当前仅冻结审计，不把复杂多层形态塞入参数化父类首片。
