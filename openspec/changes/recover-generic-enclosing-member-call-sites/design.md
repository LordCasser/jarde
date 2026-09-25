## Context

见 [proposal](proposal.md) 与 [矩阵证据](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/shape-matrix/analysis.md)。现有 `member_inner::prove_target` 已核验目标自身 `InnerClasses` 行、合成捕获字段、构造器前缀和 prologue；`outer_relation_agrees` 又核验外层的反向行，但以“外层含 `Signature`”为硬拒绝。`facade::read_class_source_member_inner_targets` 在这些事实通过后才给 jarde-java 现有 `New`/SSA 站点证明。`class_source` 的方法头来自 descriptor；普通泛型投影已使用 reader 的签名解析与擦除，却只拼写单段 class 类型且只接纳参数直接返回等少数 body 候选。`class_source` 目前一次仅装配一个 class，不具备嵌套声明树。

JADX 1.5.6 的 `ClassModifier`/`InsnGen` 提供把外层物理首参与普通源级实参分开的发射位置；其 `SignatureParser` 保留嵌套段。矩阵里四个 JADX 调用方都省略 `outer.new`，独立重编均失败，因此此处借用位置和数据形态，不沿用其仅凭类型/标记放行的判定。现有 reader 已提供 `ClassSignature`、`MethodSignature`、每段 `ClassType` 与擦除证明，无需新解析库；JADX 不成为运行依赖。

## Goals / Non-Goals

**Goals:** 在已选类定义、已证成员关系和已证 `new@1` 身份/早空值检查之上，支持 `Outer.A<T>` 作为限定创建的外层，向调用方输出可编译的源级嵌套类型名。覆盖原始 jar 依赖下的矩阵四个独立调用方，含原始外层类型、参数化外层类型、泛型成员和参数化成员返回。保留预算、取消、BCI 和来源记录。

**Non-Goals:** 不装配 `Outer/A/Generic` 完整源码，不为成员类发布跨声明 `T`/`V` 词法作用域，不猜原源码是显式 `<V>` 还是 diamond，不靠 LVT 推断类型。不放宽匿名/局部类、复杂界、构造重载、非公开成员、未知继承关系或任意 `$` 二进制名。

## Decisions

1. **外层泛型是一项额外证明，不是全局禁令。** 删除 `outer_relation_agrees` 对任意 `Signature` 的硬拒绝，仅在其双向 `InnerClasses`、准确类头和选定定义已成立后解析外层的单一类签名并验证擦除。首片只接受公开静态成员 `Outer.A<T>`：从 `A` 自身行及 `Outer` 的反向行确认 `A`，证明 `A<T>` 有一个普通 `Object` 界的变量、无额外父类/接口/type-use 注解。非泛型外层沿已验收分支。与“看到 `$A` 就拆名”相比，这能拒绝同名顶层类及冲突 InnerClasses；与扩展完整词法作用域相比，调用方只需要类身份和声明实参数量，不需要投影成员方法里的 `T`。
2. **一个选定的源类型路径供调用点和方法头共享。** 对调用方 descriptor/Signature 涉及的 `Outer$A` 与 `Outer$A$Generic`，逐段核对已选定义、双向成员行、静态性、简单名、泛型变量数量与 class 签名擦除，形成仅当前请求有效的二进制名到已证源级段的映射。reader 的 `SignatureType::Class` 保留每段参数；用此映射拼写 `matrix.Outer.A<String>` 与 `matrix.Outer.A<String>.Generic<Integer>`，并确认最后一段的二进制名等于物理 descriptor 擦除。原始 `A` 参数以及同一 `A` 的 `mark(...)` 静态调用限定符也使用映射拼成 `matrix.Outer.A`：实现前的独立 `CallMark` 探针证明现有通用发射会输出 javac 找不到的 `matrix.Outer$A.mark(...)`。这个映射来自环境唯一解析，不能作为全局缓存或由调用方 `InnerClasses` 单边记录推出；`Outer.A` 的静态成员边和 `A.Generic` 的非静态成员边分别核验。签名形态不匹配则局部拒绝，不能退到会误导编译器的 `$` 拼写并宣称成功。
3. **方法头投影只在同一已恢复构造 body 上放行。** `UsePlain` 与 `UseGenericObject` 的参数化 `A<String>` 方法签名虽经擦除证明，但现有普通泛型投影因返回值不是参数而拒绝。增加受限的同一运行候选：body 为已证 `new@1` 的单一返回，参数仅作为封闭实例并通过已证 SSA 绑定，其余实参的副作用位置不变；返回类型为 `Object` 或该成员参数化类型，后者与已选目标、方法 Signature 最后一段一致。用这个候选更新方法头的源级类型，而不开放所有已恢复 body 的泛型签名。无 Signature 的 `UsePlainRaw` 仍需经已证类型路径更新 descriptor 拼写。候选不足时保留现有拒绝。
4. **创建语法与物理实参保持分层。** 泛型成员的类/构造器 Signature 沿用上一 change 的尾部擦除证明和 `generic_diamond`；非泛型成员不输出 diamond。`NewRecord` 仍保留物理外层首参、目标和每个 BCI，发射只消费已经交接的 `outer`、成员简单名与普通实参。不得在字符串后处理阶段删除第一个参数，也不得通过 JADX 的 `SKIP_FIRST_ARG` 代替外层 SSA 同一性和空值检查顺序。
5. **验收以调用方重编、运行和反例为准。** 原始 jar 的类族不替换；单独提取 Jarde 的四个调用方源码，Java 8 编译后替换这些 caller class 执行矩阵 Runner，在 `-g`、`-g:none` 下比较完整 stdout。控制包括缺/重定义、错误双向成员行、错误泛型参数数量或擦除、错误构造器签名、SSA 外层错身份与迟空值检查；缺证据只拒绝相关站点，预算/取消上升为请求停止。源码展示标头本身的保守说明不代替编译执行证据。

## Risks / Trade-offs

- **参数化类型改写可能改变 Java 绑定** → 只对同一运行已证构造返回/外层实例用法启用，并由 caller-only Java 8 编译与执行核验；其它 body 仍拒绝。
- **`$` 既可能是成员分隔符，也可能属于合法标识符** → 只按已选定义及双向 `InnerClasses` 建源类型路径，遇到冲突或缺失不猜。
- **构造器签名省略物理外层首参** → 延用已证捕获字段、prologue 和 descriptor 尾部对齐，不能把差一个参数作为普适规律。
- **完整类族仍不可重编** → 报告只承诺本 change 的独立调用方；完整嵌套声明、外层 `T` 的词法作用域和成员声明投影另立 OpenSpec。
