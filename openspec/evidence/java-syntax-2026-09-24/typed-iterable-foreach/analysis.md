# Typed iterable foreach 审计

## 结论

Jarde 现在只能写 `Object`，直接原因是现有 `iterable_for_each_candidate` 在证明遍历后固定构造 `ForEach { ty: Object }`，并把原 `next()` 或 `next()` 内层调用换成 `Object` 局部；原有 `String` 转换仍留在循环体。见 [`build.rs:5386`](../../../../crates/jarde-java/src/build.rs)、[`build.rs:5652`](../../../../crates/jarde-java/src/build.rs) 与 [`build.rs:5777`](../../crates/jarde-java/src/build.rs)。`ForEach` AST 把元素类型作为节点字段，[`emit.rs:668`](../../../../crates/jarde-java/src/emit.rs) 原样写这个类型；之后没有从迭代源或 `next()` 返回值重新算出元素类型。

这首先是呈现精度，不是行为恢复的必要条件。当前类源码仍保留原 `checkcast`，`while` 形态编译及运行正确；`for (Object item : rawIterable) { String value = (String) item; … }` 也合法并保留转换。冻结的 `iterable-raw-projection` 已证明 raw `Iterable` 与 Supplier 输入下，这种 Object 循环改写保留可观察结果和异常顺序。JADX 的 `for (String …)` 是更贴近 Java 原源码的显示；它的 LoopRegionVisitor 会把 `iterVar` 连同类型放入 `ForEachLoop`，RegionGen 直接声明该变量，所以元素类型不是 codegen 从 `Iterable` 现场推断出来的。JADX `fixIterableType` 会用 `iterableArg` 的泛型实参校验/修正变量类型；其 SignatureProcessor 也将方法 Signature 的泛型参数类型应用到方法类型。参见本地 [LoopRegionVisitor.java](/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/regions/LoopRegionVisitor.java) 与 [SignatureProcessor.java](/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/SignatureProcessor.java)。

## 最小证据与 Java 约束

本层已有足够的字节码证据在一个窄切片内把 `String` 局部提升为 foreach 元素：同一轮 SSA 已证明 `Iterator.next()` 的唯一消费者是紧随其后的 `checkcast String`，该转换的唯一消费者是循环绑定的 store；next 值、循环内用途及迭代器 SSA/phi 均已核对；iterator、hasNext、next 的 owner/descriptor 精确匹配；cast 与循环调用处于同一异常处理集合。当前候选已经覆盖这些必要边界的一部分，并从 `Operation::CheckCast { ty }` 按常量池目标类型构造真实 Cast 表达式（不是 LVT 或模糊 inferred type）。若要提升类型，应只使用这条直接 cast 的精确目标，并把 cast BCI 并入 foreach 的来源；只在 cast 紧邻 next、没有 body 前置效果、转换仍在每轮同一位置时，才可将原首条声明折成循环变量。非这种形状时仍保留 `Object` 加原 cast。

但 raw `Iterable` 的静态元素类型为 `Object`。单写 `for (String value : values)` 会被 Java 8 javac 拒绝（证据 [`RawTypedForeachFails.java`](RawTypedForeachFails.java)、[编译诊断](raw-typed-javac.stderr)）。探针显示，可由 bytecode checkcast 的直接目标构造参数化表达式 `(Iterable<String>) values`，再编译成 typed foreach；它是 unchecked source cast，但擦除为同一个 `Iterable`，javac 没有在该局部增加运行时指令，生成的 `next(); checkcast String; astore String` 与参数化 `Iterable<String>` 的 typed foreach 相同。此选择不依赖方法 `Signature`，但它会在输出中合成 unchecked 泛型断言。审计建议当前保留较小且直白的 `Object`+原 cast；若后续要求干净的 `for (String … : values)` 且不合成该断言，应先对方法参数的普通参数化 `Signature` 建立可呈现来源。

一次 `--release 8` 对照同时编译 `for (String value : Iterable<String>)`、raw `Iterable` 上的 `for (Object item …)` 加显式 cast、以及从显式元素 cast 目标形成 `(Iterable<String>) raw` 的 typed foreach。`-g` 和 `-g:none` 两组都通过 `javac` 与 `java -Xverify:all`，正常值、null 元素 NPE、错误元素 CCE 一致；后两种 raw 方法反射仍报告参数为 raw `Iterable`。方法 Signature 由方法声明决定，与循环体写法分离；typed 泛型方法反射为 `Iterable<String>`，typed raw 加参数化 foreach 表达式的反射仍为 `Iterable`。完整 [源码](TypedIterableForeachProbe.java)、[有调试表日志/`javap -v`](with-g/)、[无调试表日志/`javap -v`](no-g/) 固定在本目录。调试表只改变 class 哈希和局部变量元数据，不提供这个 foreach 元素类型证明。

这些输出的行为相等不代表所有 JVM 诊断文本相同：局部槽布局、变量名和 helpful-NPE 文字可能不同，正如先前 raw-projection 证据已记录的差异。元素类型升级的收益是源码可读性和对 JADX 参考的接近；当前循环体 cast/while 的行为正确性不等待它。

## 是否等待泛型 Signature 投影

不应把在途的 `recover-generic-method-signatures` 作为 foreach 行为的硬前置；它针对的 method-local type parameter 声明也不是普通参数 `Iterable<String>` 的参数化拼写。当前实现要求同轮 AST/SSA 直接返回参数并检查相邻调用绑定；实现入口 [`project_method_signature`](../../../../src/class_source.rs) 与 source shape 检查并未覆盖一般参数化参数。若只保留 `Object`+cast，foreach 本身无需任何 generic Signature 投影；若要输出干净的 String header，则需方法参数签名能提供可编译的 `Iterable<String>` 来源，这将需要单独补普通参数化 `Signature` 的 source 投影，且 2.3/3.1 即使完成也未必覆盖它。不要因 foreach 顺手扩大该 change。

若产品选择让方法声明本身保留 `Iterable<String>` 并从该签名推出元素类型，则需要另做普通参数化 `Signature` 的 source 投影及对应 erasure/source proof；这是呈现类型信息的架构债务，可单独记录。它不是当前行为所需。最小闭环可先保持 Object/cast，使用 `checkcast` 目标作为未来局部类型候选的证据，但不要直接使用模糊 inferred type 或 LVT 名称/类型。

`List` / `Collection` owner 支持也仍应按冻结证据限定：当前候选分别要求 invokeinterface owner 是 `java/util/List` 或 `java/util/Collection`，且 SSA 呈现类型精确匹配该 owner；自定义 `TextIterable` owner 不满足条件。不能由这两个标准接口推导任意传递子类型支持。参见 [`iterable-subtype-owners/analysis.md`](../iterable-subtype-owners/analysis.md)。
