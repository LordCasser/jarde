## 2.3/3.1 限定投影验收（2026-09-24）

当前准入一个或多个方法局部类型变量，界可以是非参数化、非嵌套、可拼写的 class 和 interface 组合；非泛型参数可以是原始类型或简单 class 类型。返回类型须为其中一个变量，正文须是同一次恢复构建出的唯一 `return` AST：直接返回该变量对应的参数，或由未改型的 `boolean` 参数作条件、两臂均返回该变量对应的参数。每个局部引用的主来源须对应 SSA 的真实 load，参数槽在全方法 SSA 中不得写入；条件节点附带的来源只可指向 JVM 条件分支。其它赋值、调用、局部合流、表达式与 fallback 都没有本次证明，整项拒绝。当前类常量池中任何对该方法名的同类 `Methodref`（包括相邻重载）也整项拒绝，以免改变已知调用绑定。

reader 对同一成员唯一 `Signature` 的完整语法、方法局部作用域、逐参数/返回 descriptor 擦除与 `Exceptions` 核对仍是前提。class-source 对 varargs、任一方法/参数/type-use 注解、参数化或嵌套界、数组/通配符/复杂参数形状和无同次正文证明均记录拒绝，保留物理 descriptor 声明。`Signature` 的 `^T` 异常后缀因现有头部只拼写物理 `Exceptions` 类型而明确拒绝；空后缀不覆盖物理 `Exceptions`。签名侧车只在成员有 `Signature` shell 时由同一次 `build::Program` 和 SSA 产生，不加入独立方法 `RecoveryReport`，无 XRef 依赖。

新增定向验证命令：`CARGO_TARGET_DIR=/tmp/jarde-generic-method-impl-target cargo test -p jarde --test generic_method_projection --locked`，9/9 通过。除冻结正例、五个 JVM 可验证反例、无调用相邻重载的准入、同类调用的保守拒绝、type-use 注解及 varargs 整项拒绝外，还覆盖两个局部变量及普通参数、`throws T` 整项拒绝、正文赋值及正文调用的拒绝。两个局部变量的 Jarde 完整类由 `javac --release 8 -g:none` 重编，并用 `java -Xverify:all` 对照原完整类的普通调用、`getTypeParameters`、界、`getGenericParameterTypes` 和 `getGenericReturnType`；擦除匹配但正文不兼容的冻结反例也以相同编译和 JVM 选项验收 `7|0`。

使用独立临时目录、`javac --release 8 -Xlint:-options` 与 `java -Xverify:all` 重编/运行原、JADX、Jarde 三方完整类。`GenericMethodProbe` 三方一致：变量 `T`，界 `java.lang.Number`，泛型参数 `T,T,boolean`，泛型返回 `T`，普通调用值 `3`。`GenericThrowsProbe` 三方一致：变量 `T`，界 `java.lang.Number`，参数/返回 `T`，`throws java.io.IOException`，普通调用值 `3`。五个冻结反例均保留 descriptor 声明、带拒绝标记，Jarde 完整源码可用 Java 8 重编。

尚未完成 2.3/3.1 的全量范围：当前证明只覆盖上述静态单返回形状；赋值到类型变量局部、多个返回语句、其它表达式合流、正文调用目标与同类调用绑定均尚无跨方法源级类型环境可证明。这些情况保持整项拒绝，任务框保持未勾选。3.2 的低预算、取消、essential/all 专项由 [Root 预算/取消及 CLI 复核](verification-root-3.2.md) 独立覆盖。严格 Clippy 在别处现存 `enumswitch.rs` 的 `useless_conversion`、`region.rs` 的 `too_many_arguments`/`type_complexity`、`report.rs` 的 `needless_option_as_deref` 及 `facade.rs` 桥方法段的 `collapsible_if` 处失败；这些不属于当前泛型变更。
