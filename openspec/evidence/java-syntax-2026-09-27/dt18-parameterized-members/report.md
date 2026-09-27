# DT-18：参数化字段、参数与返回值的首个独立切片

固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `TestGenerics2` 混合了嵌套类、参数化成员、字段读取和局部变量类型推断；`TestGenericFields` 主要断言局部变量的推断结果。这里先用顶级 `GenericSlots` 把字段、方法参数和返回值拆开，避免把成员类位置和数据流问题误算为签名问题。无 `Signature` 的 raw `List` 字段及方法是负控制。[replay.py](replay.py) 固定原 class、JADX、Jarde 三份完整类型源码和同一个外部 [Runner.java](Runner.java)，分别用 Java 8 重编并以 `-Xverify:all` 运行。两次重放的五个输出文件逐字相同。

原 class 与 JADX 在反射中均保留 `List<String> names`、`List<String> id(List<String>)`、`List<String> empty()`；Jarde 已正确保留无同类 `Fieldref` 的字段和同轮参数返回 `id` 的完整泛型类型。三方 raw 控制和 `id`/`empty` 实际调用结果一致。唯一差距是 Jarde 将无参 `empty(){return null;}` 的返回声明写成 raw `List`；它仍能编译运行，但反射从 `java.util.List<java.lang.String>` 变为 `interface java.util.List`。真实 class 的物理 descriptor 是 `()Ljava/util/List;`，`Signature` 是 `()Ljava/util/List<Ljava/lang/String;>;`，因此这不是无签名的合法擦除。

reader 已解析方法 `Signature` 并校验擦除，`class_source::ordinary_parameterized_declaration` 能拼写参数化 `List<String>`；现有 `id` 正例也证明同轮候选与原子方法投影工作正常。`empty` 的正文已经恢复为 `return null;`，但 `jarde-java::generic_return_candidate` 未向此方法提供同轮 null 返回候选，class-source 据此记录 `ordinary_generic_source_unproved`。DT-16 的独立实现正在为精确无效果 `aconst_null; areturn` 增加同轮候选；DT-18 应在该实现验收后复用此事实，仅为普通参数化 `List<String>` 返回添加独立的声明门，不能把 DT-16 的方法自有类型变量边界顺手扩大。

此切片只确认顶级普通类中的未使用参数化字段、参数直接返回和参数化 null 返回。`TestGenerics2` 的嵌套 `WeakReference<V>`/`Map<Object, ItemReference<V>>`、字段读取、局部变量推断，以及 `TestGenericFields` 的链式字段读取仍在 DT-18 后续队列，不能由本例判定已追平。

## 修后验收

`replay.py fixed <absolute-path-to-jarde-cli>` 使用冻结的 JADX `2fb1b16386941660fda07e9017285aec40fcb37f`。原 class、JADX 完整类型、Jarde 完整类型分别与同一 `Runner.java` 以 `javac --release 8` 重编，并以 `java -Xverify:all` 运行。三方逐字输出相同：`empty` 的返回反射类型为 `java.util.List<java.lang.String>`；未使用参数化字段、`id(List<String>)` 和三个 raw 控制保持一致；调用结果为 `ok:true`。Jarde 输出中不再出现泛型 Signature 拒绝诊断。三方源码、原 class 的 javap、逐字结果、输入和输出哈希保存在 [fixed 输出](outputs/fixed/)。

Rust 集成测试还覆盖其他参数化返回、物理 descriptor 与 Signature 擦除不符、正文额外效果、同类调用绑定、输出预算停止和取消；所有拒绝路径都保留物理方法 descriptor，完整的有效声明仅在原子投影通过时发布。嵌套参数化成员、字段读取和局部变量推断仍留在后续队列。
