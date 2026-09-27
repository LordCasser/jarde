# 声明与类型：JADX 测试驱动的特性单元

基线、计数口径见 [README.md](README.md)。本组归属 `types` 47、`inner` 39、`enums` 26、`names` 22、`generics` 21、`java8` 11、`annotations` 7，共 **173 个测试文件**。下面的 **31 个 DT 单元**是可构造、可验证的 Java 源码形态及恢复边界，不是测试通过率；每行给出 JADX 的代表测试和生产实现入口。测试路径均相对于 `jadx-core/src/test/java/jadx/tests/integration/`，实现路径均相对于 `jadx-core/src/main/java/jadx/core/`。同一个文件可支撑多个单元；未逐行覆盖的测试仍属于上述文件范围。

| ID | 应恢复的语法单元 | 代表测试 | JADX 生产入口 | 证据边界 |
| --- | --- | --- | --- | --- |
| DT-01 | 非静态具名成员类声明、外部实例隐藏字段消除 | `inner/TestInnerClass.java`, `inner/TestInnerClass5.java` | `dex/visitors/ClassModifier.java`, `codegen/ClassGen.java` | 断言类层级和无 `this$`；多级捕获需另测行为。 |
| DT-02 | 静态成员类声明与实例化 | `inner/TestInnerClass2.java` | `dex/visitors/ClassModifier.java`, `codegen/ClassGen.java` | 不能把仅凭 `$` 命名的独立类误纳入。 |
| DT-03 | 限定外部实例的成员构造 `a.new AA()` | `inner/TestInnerConstructorCall.java`, `generics/TestOuterGeneric.java` | `dex/visitors/ConstructorVisitor.java`, `codegen/InsnGen.java` | `TestOuterGeneric` 中泛型成员构造用例标 `@NotYetImplemented`。 |
| DT-04 | 词法封闭实例接收者 `Inner.this` | `inner/TestAnonymousClass2.java` | `dex/visitors/ClassModifier.java`, `codegen/InsnGen.java` | 代表断言为 `Inner.this;`；不外推所有 `Outer.this` 变体。 |
| DT-05 | 匿名接口实现 `new I() { ... }` | `inner/TestAnonymousClass.java`, `inner/TestAnonymousClass18.java` | `dex/visitors/ProcessAnonymous.java`, `dex/visitors/AnonymousClassVisitor.java`, `codegen/InsnGen.java` | 默认内联与禁用内联两种策略分别断言。 |
| DT-06 | 匿名父类及其显式构造实参 | `inner/TestAnonymousClass10.java`, `inner/TestAnonymousClass15.java` | `dex/visitors/ProcessAnonymous.java`, `dex/visitors/AnonymousClassVisitor.java`, `codegen/InsnGen.java` | `TestAnonymousClass10` 精确断言多实参表达式。 |
| DT-07 | 匿名类内再声明匿名类 | `inner/TestNestedAnonymousClass.java`, `inner/TestAnonymousClass12.java` | `dex/visitors/ProcessAnonymous.java`, `dex/visitors/AnonymousClassVisitor.java` | 内联图有依赖顺序；不可仅靠类名识别。 |
| DT-08 | 匿名类捕获局部变量或封闭实例 | `inner/TestAnonymousClass7.java`, `inner/TestAnonymousClass8.java` | `dex/visitors/AnonymousClassVisitor.java`, `dex/visitors/ClassModifier.java` | `TestAnonymousClass7` 断言 `final double d` 和体内 `d`；构造签名匹配另验。 |
| DT-09 | 匿名类实例初始化块 | `inner/TestAnonymousClass4.java` | `dex/visitors/AnonymousClassVisitor.java`, `codegen/ClassGen.java` | 断言匿名体 `{ f = 1; }` 与重写方法顺序。 |
| DT-10 | 空枚举、普通枚举常量 | `enums/TestEnums.java`, `enums/TestEnums7.java` | `dex/visitors/EnumVisitor.java`, `codegen/ClassGen.java` | [已实测 0/1/4 常量差距](../java-syntax-2026-09-27/enum-arity/report.md)：不将编译器 `$VALUES` 当源码字段；两常量既有证明不代表本单元完成。 |
| DT-11 | 带构造参数、字段及构造器的枚举 | `enums/TestEnums3.java`, `enums/TestEnums4.java` | `dex/visitors/EnumVisitor.java`, `codegen/ClassGen.java` | 可变参数构造器见 `TestEnums4`。 |
| DT-12 | 含匿名常量体的枚举 | `enums/TestEnums2a.java`, `enums/TestEnums6.java` | `dex/visitors/EnumVisitor.java`, `dex/visitors/ProcessAnonymous.java` | [双匿名体 String 实参修后验收](../java-syntax-2026-09-27/dt12-anonymous-enum-audit/implementation-report.md)：准确两常量构造前缀、纯构造转发和唯一接口抽象方法获证后，原/JADX/Jarde 全源码在 `-g`/`-g:none` 下 Java 8 重编验证运行一致；错 ordinal/桥转发、非 ASCII 和三常量拒绝。`TestEnums6` 无匿名常量体，归 DT-11；其它匿名枚举形态仍待扩验。 |
| DT-13 | 嵌套枚举及 `enum implements I` | `enums/TestInnerEnums.java`, `enums/TestEnumsInterface.java` | `dex/visitors/EnumVisitor.java`, `codegen/ClassGen.java` | [固定 DT-13 对照](../java-syntax-2026-09-27/dt13-enum-shapes/report.md)：简单 interface header 三方重编/运行通过；嵌套 enum 的 Jarde `$` 顶层布局使原嵌套 API consumer 编译失败，窄 OpenSpec 待审。`TestInnerEnums` 另含 DT-11 构造实参；`TestEnumsInterface` 常量专属匿名体归 DT-12，均未由此清项。 |
| DT-14 | 枚举自定义静态初始化及三元构造实参 | `enums/TestEnumsWithCustomInit.java`, `enums/TestEnumsWithTernary.java`, `enums/TestEnumsWithStaticFields.java` | `dex/visitors/EnumVisitor.java`, `codegen/ClassGen.java`, `codegen/InsnGen.java` | [DT-14 三方对照](../java-syntax-2026-09-27/dt14-enum-init/report.md)：同类静态布尔调用接双 int literal arm 的 ternary 构造实参，以及普通 enum/int literal 控制，已在全源码 Java 8 重编与验证运行中追平。values→Map 用户 suffix 和 String ternary 仍是源码编译差距，分别属于静态后缀 OpenSpec 与 DT-11 交叉边界。`TestEnumsWithStaticFields` 来自 Smali 且明确 `disableCompilation()`，不是正向重编证据。 |
| DT-15 | 类/接口泛型形参与上界 | `generics/TestUsageInGenerics.java`, `generics/TestGenericsMthOverride.java` | `dex/visitors/SignatureProcessor.java`, `dex/nodes/parser/SignatureParser.java`, `codegen/ClassGen.java` | [顶级薄切片](../java-syntax-2026-09-27/dt15-generic-class-bounds/report.md)三方 Java 8 重编与反射上界一致；原两个 JADX 测试只直接断言方法头，成员类组合未验收。有效 Signature 是前提；损坏签名属容错。 |
| DT-16 | 方法泛型形参与上界 | `generics/TestUsageInGenerics.java`, `generics/TestGenericsInArgs.java` | `dex/visitors/SignatureProcessor.java`, `codegen/MethodGen.java` | [实例泛型 null 返回修后验收](../java-syntax-2026-09-27/dt16-generic-null-return/report.md)：固定 `<T extends Number> T value()` 已由同轮 AST/Code/SSA 精确 null 事实与 Signature 擦除证明投影；原/JADX/Jarde 完整源码与外部显式 `<Integer>` consumer Java 8 重编、验证运行及反射结果一致。类级变量、效果和错误擦除等拒绝；`TestGenericsInArgs` 复杂正文仍待拆分验收。 |
| DT-17 | `?`、`? extends`、`? super` 通配符 | `generics/TestGenerics.java`, `generics/TestGenerics3.java` | `dex/nodes/parser/SignatureParser.java`, `codegen/TypeGen.java` | [单 List 参数空 void 方法验收](../java-syntax-2026-09-27/dt17-wildcard-void/report.md)：五种通配符在原/JADX/Jarde 全源码 Java 8 重编、验证运行及反射中一致；无 Signature raw 控制不被推断。参数使用、额外效果、嵌套界、type-use 注解、错擦除与不完整 Code 均拒绝；含正文和其他成员界仍待扩验。 |
| DT-18 | 字段、方法参数/返回值中的参数化类型 | `generics/TestGenerics2.java`, `generics/TestGenericFields.java` | `dex/visitors/SignatureProcessor.java`, `dex/visitors/GenericTypesVisitor.java`, `codegen/TypeGen.java` | [顶级成员首切片](../java-syntax-2026-09-27/dt18-parameterized-members/report.md)：Jarde 的未使用 `List<String>` 字段与参数直接返回在三方 Java 8 全源码重编/反射中通过；无参 `List<String> empty(){return null;}` 编译运行但返回泛型反射退化为 raw `List`。raw 无 Signature 控制正确。嵌套参数化成员、字段读取和局部变量推断仍待测。 |
| DT-19 | 外部泛型实参流入非静态成员类 | `generics/TestTypeVarsFromOuterClass.java`, `types/TestGenericsInFullInnerCls.java` | `dex/visitors/SignatureProcessor.java`, `dex/visitors/GenericTypesVisitor.java` | 测试显式拒绝 `Object` 和错误的 `Outer<Y>.Inner`。 |
| DT-20 | 构造表达式菱形语法 `new HashMap<>()` | `generics/TestConstructorGenerics.java` | `dex/visitors/GenericTypesVisitor.java`, `codegen/InsnGen.java` | [debug/no-debug 定向对照](../java-syntax-2026-09-27/dt20-diamond-local/report.md)：三方完整源码 Java 8 重编及验证运行一致；`-g` 中 JADX 保留 `Map<String,String> map = new HashMap<>()`，Jarde 退化为 raw `HashMap`/cast。`-g:none` 时 JADX 自身也写 raw 构造；真实 `LocalVariableTypeTable` 是本片泛型实参的必要证据。 |
| DT-21 | 泛型继承、方法覆盖及编译器 bridge 处理 | `generics/TestGenericsMthOverride.java`, `generics/TestSyntheticOverride.java`, `generics/TestTypeVarsFromSuperClass.java` | `dex/visitors/GenericTypesVisitor.java`, `dex/visitors/ClassModifier.java` | 需要检查声明与调用分派一致，不能只隐藏 bridge。 |
| DT-22 | 注解类型声明及元素默认值 | `annotations/TestAnnotations.java`, `annotations/TestAnnotations2.java` | `codegen/AnnotationGen.java`, `codegen/ClassGen.java` | 精确断言 `float value() default 1.1f;`。 |
| DT-23 | 类、字段、方法、参数上的注解使用 | `annotations/TestParamAnnotations.java`, `annotations/TestAnnotationsUsage.java` | `codegen/AnnotationGen.java`, `codegen/MethodGen.java` | `TestAnnotationsUsage` 主要检查 use 图，源码放置仍需独立重编。 |
| DT-24 | 注解中的标量、类字面量、枚举、数组、嵌套注解 | `annotations/TestAnnotationsMix.java`, `inner/TestReplaceConstsInAnnotations2.java` | `codegen/AnnotationGen.java` | `TestAnnotationsMix` 对复杂值的源码断言很弱；需要补完整输出/运行时反射。 |
| DT-25 | Lambda 表达式及参数列表 | `java8/TestLambdaStatic.java`, `java8/TestLambdaArgs.java` | `dex/instructions/invokedynamic/CustomLambdaCall.java`, `codegen/InsnGen.java` | `TestLambdaResugar.java` 的 lambda 方法内联标 `@NotYetImplemented`。 |
| DT-26 | Lambda 捕获局部值/实例 | `java8/TestLambdaExtVar.java`, `java8/TestLambdaExtVar2.java`, `java8/TestLambdaInstance.java` | `dex/instructions/invokedynamic/CustomLambdaCall.java`, `codegen/InsnGen.java` | `TestLambdaExtVar` 明确仍期待较冗长的 block lambda。 |
| DT-27 | 静态、实例、构造器方法引用 | `java8/TestLambdaStatic.java`, `java8/TestLambdaInstance.java`, `java8/TestLambdaConstructor.java` | `dex/instructions/invokedynamic/CustomLambdaCall.java`, `codegen/InsnGen.java` | `Object::toString` 断言仍注释为 TODO；构造器引用有精确断言。 |
| DT-28 | 原始类型收窄/扩宽与条件表达式中的强转 | `types/TestPrimitiveConversion.java`, `types/TestLongCast.java` | `dex/visitors/typeinference/TypeInferenceVisitor.java`, `codegen/InsnGen.java` | `TestPrimitiveConversion2.java` 禁用编译，不作强证据。 |
| DT-29 | 引用类型/接口强转 | `types/TestInterfacesCast.java`, `types/TestFieldCast.java` | `dex/visitors/typeinference/TypeInferenceVisitor.java`, `codegen/InsnGen.java` | 要保证访问、重载解析和运行时 cast 语义。 |
| DT-30 | 数组类型与数组字面量的显式构造 | `types/TestArrayTypes.java` | `dex/visitors/typeinference/TypeInferenceVisitor.java`, `codegen/InsnGen.java` | 精确断言 `use(new Object[]{e});`。数组赋值、创建/访问的其他形态归 EM 组去重。 |
| DT-31 | 枚举 `switch` 中源级常量标签和编译器映射数组消除 | `enums/TestSwitchOverEnum.java`, `enums/TestSwitchOverEnum2.java` | `dex/visitors/FixSwitchOverEnum.java`, `codegen/RegionGen.java` | Java 源编译路径与 Java 21 ordinal 直分派 Smali 分别验收；不把两个 lowering 视为同一字节码模式。 |

`names` 22 个文件以及多数 `types/TestTypeResolver*` 是这些语法单元的**支撑机制**：标识符合法化/冲突、包与导入消歧、类型推断、无调试信息的变量恢复。它们不按文件数另算“语法特性”，但每个 DT 单元的可编译性都受其影响。代表入口是 `dex/visitors/rename/RenameVisitor.java`、`dex/visitors/typeinference/TypeInferenceVisitor.java` 和 `codegen/TypeGen.java`。`generics/TestClassSignature.java` 检查损坏签名不栈溢出，也归容错测试，不冒充泛型恢复成功。

额外代码线索：`codegen/InsnGen.java` 存在 `callSuper` 生成路径；本组 `inner` 和 `types` 测试未发现对 `Outer.super` 的直接源码断言。这个点暂不计入 **31 个测试支撑的 DT 单元**，作为测试缺口记在后续差距矩阵；Jarde 已有自己的编译/运行探针。另有 `inner/TestAnonymousClass5.java`、`inner/TestAnonymousClass3a.java` 带 `@NotYetImplemented`，不得当作 JADX 已完成证据。

## 173 文件主归属闭合

以下按每个测试文件的**主要目的**互斥归属；表内单元的代表路径仍可交叉引用。`names` 的 22 个文件包含子目录中的辅助源码。此账本不把类型推断、命名和 Kotlin 输入换算成额外 Java 语法特性。

| 目录 | DT 单元主归属 | CF/EM 交叉归属 | 支撑/容错归属 | 合计 |
| --- | ---: | ---: | ---: | ---: |
| `types` | 15（`TestGenerics*` 9、转换/强转 5、数组 1） | 3（`TestPrimitivesInIf`、`TestFieldAccess`、`TestConstInline`） | 29（`TestTypeResolver*`、`TestTypeInheritance`、`TestConstTypeInference`） | 47 |
| `inner` | 36（匿名 25、具名成员 9、注解常量 2） | 1（`TestOuterConstructorCall`） | 2（合成成员重命名） | 39 |
| `enums` | 25（枚举恢复 23、枚举 switch 2） | 0 | 1（`TestEnumKotlinEntries`） | 26 |
| `names` | 0 | 0 | 22（标识符、包、碰撞与辅助类） | 22 |
| `generics` | 17 | 2（`TestGenerics6`、`TestMissingGenericsTypes2` 的 foreach） | 2（损坏签名容错、泛型导入） | 21 |
| `java8` | 11 | 0 | 0 | 11 |
| `annotations` | 5 | 0 | 2（重命名传播） | 7 |
| **合计** | **109** | **6** | **58** | **173** |

主归属闭合不等于 173 个断言均为成功源码重编；例如 `generics/TestOuterGeneric.java` 的泛型内部构造、`java8/TestLambdaResugar.java` 的 lambda 内联、`enums/TestEnumsWithAssert.java` 的 assert 目标均含未实现测试，必须由各 DT 行的边界解释。`DT-30` 的显式数组初始化与 EM-18 是同一源级单元，汇总时只保留 EM-18。
