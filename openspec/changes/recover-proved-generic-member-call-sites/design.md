## Context

见 [proposal](proposal.md)。[非泛型成员构造](../recover-proved-member-inner-construction/design.md)已证明选定定义中的双向 `InnerClasses` 关系、合成外层字段与构造器 prologue，并在调用点证明 SSA 身份、`requireNonNull` 和效果次序。它目前刻意拒绝目标类与构造器上的 `Signature`。现有 reader 可带预算解析类/方法签名并证明擦除；`New` AST 已区分完整语义类型与限定接收者/成员简名，但尚无泛型创建语法标记。JADX 的 `SKIP_FIRST_ARG` 与 `InsnGen.addOuterClassInstance` 可参考源级参数和限定接收者的发射位置，不能替代上述物理及 SSA 证明：[两层泛型反例](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/analysis.md)中它丢失外层实例；[非泛型外层对照](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/non-generic-outer-generic-member/analysis.md)中仅返回 `Object` 的调用也被错误写为不能重编的 `new Outer.Inner(outer, ...)`，而泛型返回声明的对照正确。

## Goals / Non-Goals

**Goals:** 在原始依赖 class 已固定的前提下，只为调用方恢复一个可重编、可执行的 Java 8 泛型成员构造子集；保持 `new@1` 的物理目标与实参来源、预算及停止合同。首个目标形状为非泛型公开外层类、一个公开非静态成员类 `Inner<V>`、一个无重载的公开构造器，其 `V` 无复杂界且构造器签名参数与物理 descriptor 去掉外层首参后的尾部逐项擦除相同。

**Non-Goals:** 不投影 `Outer<T>.Inner<V>` 的完整类族声明，不恢复成员字段/方法的 `V` 类型、局部泛型变量、跨类声明装配或源中显式 `<T>` 与 diamond 的原始选择；不放宽匿名/局部类、复杂泛型界、构造重载、未证可访问性与缺失依赖。后续嵌套声明与泛型作用域另立 change，不能由调用点的成功报告推导整 jar 可重编。

## Decisions

1. **泛型仅扩展既有成员目标证明。** 在现有 target/outer 定义选取与双向成员关系成功后，读取目标类唯一 `Signature`，要求单个自有 `V`、普通 `Object` 界、物理父类及接口擦除一致、无 type-use 注解或外层类泛型作用域；再读取准确 `<init>` 的唯一 `Signature`，要求无方法类型参数、一个源级 `V` 实参与 `void` 结果。构造器集合限唯一可用目标，免得源码 diamond 在重编时绑定另一重载。原有非泛型分支不改变。替代方案是全局开放带签名成员类，既不能证明隐式首参与签名位置，也会引入声明投影，故不采用。
2. **只在已证明的前缀边界对齐签名。** 当前方法签名擦除证明仍针对完整物理 descriptor；泛型成员构造的特殊之处是 JVM descriptor 的首参为已证明的外层捕获，而 `Signature` 只列源级参数。在此边界内以 descriptor 的尾部构造对照，复用 reader 的解析和擦除证明，绝不把“跳过构造器首参”做成通用启发式。签名、目标或读取发生预算/取消时沿现有停止路径；其它缺证据只拒绝该调用站点。
3. **发射使用一个明确的泛型创建选择。** 扩展现有 `New` 构造表达式而非新增表达式族，保留完整二进制 `ty`、物理 `NewRecord.arguments` 和限定值的 BCI；仅当泛型目标证明存在时拼写 `.new Inner<>(...)`，否则沿已验收的 `.new Inner(...)`。diamond 是对此受限调用方的可编译等价表示，不声称原源码也写 diamond。不能用类名中的 `$` 或单纯给已发出的字符串补 `<>`。
4. **验收层次严格分开。** 对冻结 class 做原/JADX/Jarde 三方观察；调用方 Java 8 重编时 classpath 指向原始 `Outer` 与 `Outer$Inner`，并对正常/null/效果轨迹执行 `-Xverify:all`。另外记录完整类族重编是否成功，但不把它作为调用点子片的成功条件。essential/all/range、报告的物理 BCI、目标缺失/错签名、预算及取消仍按现有门复核。无需外部库；JADX 仅测试对照，不进入生产依赖。

## Risks / Trade-offs

- **泛型签名可被篡改为可解析却不匹配物理调用** → 逐位置擦除对齐源级尾部，并用错签名 class 控制拒绝；不能从 `V` 的拼写直接信任类型。
- **diamond 受赋值上下文或重载影响** → 首片限制唯一构造器与简单 `V` 界，调用方独立 Java 8 重编及执行验收；超界保持拒绝。
- **调用方可编而整套源码仍失败** → 报告和文档只承诺调用点，嵌套声明、两层泛型作用域及类级签名另行规划。
- **新 AST 标记影响普通 `new` 遍历** → 对现有 `New` 的所有消费者做穷尽编译检查和定向普通/非泛型成员回归；不修改 `NewRecord` 的物理参数语义。
