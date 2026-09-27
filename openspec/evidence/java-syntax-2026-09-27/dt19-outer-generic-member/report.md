# DT-19：外层类型变量流入一个非静态成员类

固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `TestTypeVarsFromOuterClass` 要求 `Outer<String>.Inner` 不被写成 `Outer<Y>.Inner`，返回的泛型类型不退化为 `Object`。另一个 Smali 测试 `TestGenericsInFullInnerCls` 包含多层参数化成员声明，先不把它混入首片。[replay.py](replay.py) 将最小的 [Outer.java](Outer.java) 编为 Java 8、`-g:none` 的两个物理类：`Outer<T>` 与非静态 `Inner`，后者的 `T id(T)` 引用外层类型变量。[Runner.java](Runner.java) 是外部 API consumer，声明 `Outer<String>.Inner` 并调用 `id("ok")`。

原始与 JADX 的完整源码各自重编后，均以 `java -Xverify:all` 输出 `ok`。Jarde 的两个物理 `class-source` 请求均完成，但完整源码重编失败：根没有嵌套 `Inner` 声明，consumer 找不到 `Outer<String>.Inner`；扁平 child 的捕获字段赋值出现在 `super()` 之前，在 Java 8 下也不合法。Jarde 根报告中的家族关系已经是 `prepared`，捕获证明是 `proved`，仅家族调用门因“non-generic root and member headers”拒绝，导致整体投影拒绝。child 的 `(TT;)TT;` 方法签名因当前可见 scope 未声明 `T` 退化成 `Object id(Object)`；根的 `()Ldt19/Outer<TT;>.Inner;` 返回签名则因缺少选定 Java 成员路径拒绝。`outputs/` 保留源码、`javap`、三方编译/验证结果与精确拒绝状态；两次重放可检查文件逐字一致。

这说明首个闭环不需要新建泛型推断器。现有 `assemble-proved-member-class-family` 已有双向成员关系、捕获和原子家族 writer，reader 已解析 `Signature` 并核对擦除，`source_type_path` 已表达选定成员路径。需要把**已证明的外层泛型词法作用域**传给 child 的方法签名投影，再在同一家族证书内选定 `Outer<T>.Inner` 返回路径及构造调用；保留每个物理 owner/BCI 的 source-map 来源，所有声明、构造和签名一起提交。不能只删 `facade` 中的“non-generic root”门槛，否则 `T` 仍无 scope 且 `Outer$Inner` 仍是错误的 Java 位置。

JADX 的 `RootNode.initInnerClasses` 按父子关系建树，`ClassGen.addInnerClsAndMethods` 递归写声明，`SignatureProcessor` 识别带 enclosing segment 的类型。可借用这个处理顺序；Jarde 仍应使用已选物理定义与同轮证明，不能依 `$` 名或仅靠 JADX 的输出字符串。多 child、多层路径、成员类自有类型参数、无 debug 局部推断，以及 `TestGenericsInFullInnerCls` 的 Smali 图分别留待扩验。首片 OpenSpec 为 `recover-proved-generic-outer-member-family`。
