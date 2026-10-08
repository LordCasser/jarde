# raw receiver：root 架构预审

2026-10-09，主线基线 `564e22c1340d2d0901117012b986277fb5292313`。Atlas 已打开本项目并作 scoped 查询；结构缓存的字段使用行号偏移，且漏掉实际存在的 write_source，因此仅作 locator，结论来自当前源码直接阅读。预审不是实现验收。

## 现有阶段与事实边界

- `src/facade.rs::field_write_source` 证明唯一 SSA BCI 的 RHS：null、未改写物理参数 entry slot、封闭完成分配。它不证明 receiver 源类型。
- `src/class_source.rs::SameClassFieldUse` 有物理方法、BCI、opcode、CP owner/name/descriptor、read_expressible、write_source、source_complete，无 receiver。
- `same_class_published_method_parameters` 目前仅从 RHS Parameter 收集方法。它在 deferred method Signature settle 后运行，只有 generic_signature_projected 才解析自己的方法 Signature；这条顺序可复用，需补 receiver formal 所需方法（包括 RHS null）。
- `project_field_signature` 先证明 Signature 擦除、后逐写者检查 parsed.ty，最后才原子发布字段。新选择类型不能改字段声明候选类型或绕过字段整体 binding/read guards。
- `report.rs` 的 build::build 生成 Program，随后 emit 成功，再 `field::committed_presentations` 检查实际 Program 中物理 field write/read。可在既有 traversal/同次候选路径输出紧凑 site receiver 事实，不需额外 AST retention 或第二个源解析器。

## 规则出处

[JLS8 §4.8](https://docs.oracle.com/javase/specs/jls/se8/html/jls-4.html#jls-4.8) 区分 raw 类自己声明的非静态字段擦除选择和 static 字段的原类型。[§4.6](https://docs.oracle.com/javase/specs/jls/se8/html/jls-4.html#jls-4.6) 定义数组组件擦除与最左上界。新路径仅处理本类物理字段，不用此规则猜继承泛型映射。

本地 JADX `/Users/lordcasser/workspace/testzone/jadx`：

- `jadx-core/src/main/java/jadx/core/dex/visitors/typeinference/TypeBoundFieldGetAssign.java` 的 getResultType 根据 instanceType 调 replaceClassGenerics；替换失败返回 initType，保留 scope TODO。这是读取端算法线索，不能当写入端已正确实现的证据。
- `jadx-core/src/main/java/jadx/core/dex/nodes/utils/TypeUtils.java` 的 replaceClassGenerics 与 getTypeVariablesMapping；raw instance 空映射，不提供已发布 Java 方法类型证明。
- `jadx-core/src/test/java/jadx/tests/integration/generics/TestGenericFields.java` 明确检查 Summary.price.value 的 Amount 局部恢复；TestTypeResolver26、TestGenerics2 提供 raw cast/receiver 局部类型边界，扩展 fixture 的出处由 evidence agent 保存。

采用访问点 receiver 决定选中成员类型这一原则；不复制 unresolved initType fallback、猜 cast 或后续全局 FixTypesVisitor fixpoint。许可与维护方面，本片只复用 Jarde 已有 parser、erasure、descriptor assignability、AST 和 Budget，无新依赖或复制代码。

## 实现必须闭合的证据

物理 putfield 的 BCI 与 FieldPlan.claim 的 Write/name/owner/descriptor 必须匹配实际 FieldAssign；不能以 primary invoke BCI 或跨方法 derived origin 冒充物理 field site。formal receiver 需 Names.whole/AST名、SSA原始 entry slot 与真实 static flag/parameter_positions 一致，再按最终已发布方法参数分类。raw local 需唯一实际 Declare 的 raw own-class 源拼写、同一实际 receiver Local 和无重绑定/作用域歧义；被折成 this 的别名绝不凭原 source raw 声明放行。source_complete 仅说明完整结构化 body。

新 raw 分支使用已校验字段 descriptor 的擦除目标和原有 raw_parameter_assignable/raw_reference_assignable 封闭边界；this、参数化或未知 receiver 继续原 generic-only proof。不能全局移除 class_type_arguments_present 条件。null 合法，Object→Number 不合法，数组维数/组件与 bound 不丢失。未证明调用、改写值和 phi 不因 raw receiver 获得 RHS 许可。

## 后续发射与 accessor 风险

字段提交之后确有 enum/array/lambda/member-family 重发射。审查得知 lambda inline helper 要唯一 Return(Some(expr))，无法包含物理 FieldAssign statement；caller lambda edit 改 invokedynamic 的 Lambda Expr。array/enum 改 method reference/switch，memberfold 的 synthetic bridge 是读取；这些既有路径不把独立物理字段写 receiver 改成另一源类型。实现仍需以物理 member/site 保持对应并针对新可达替换复核。

真实例外是已发射的 synthetic FieldWrite accessor：callee 的 raw formal 写字段，caller 的 invoke 会在 AST 变成 FieldAssign，receiver 来自 caller，可能为 this。最小片必须拒绝对这种被转写的物理 setter 使用 raw 许可；后续完整支持需 caller-side receiver/RHS 使用事实，独立立项。

root 再读发现 `RecoveryReport.accessors` 仅在 RuleDetails 被请求且 materialization 完成时存在（report.rs 的 RuleDetails 分支），**不得用可选报告详情作为语义 guard**。必须从同次 Program/accessor plan 保留独立最小事实，验证 evidence none/all 不影响恢复。access$ 名称只是拒绝记录提示，不能替代 synthetic/static/body 证明。优先使用现有非序列化 ClassSourceRecovery 候选通道，不将内部 raw 许可混入报告输出平面。

## 验收与独立债务

实现复核补充：raw 许可还要求实际类头已发布且 class scope 非空；普通非泛型类的 `C` 引用不是 raw type，不能因其没有类型实参就擦除 `List<String>` 字段的选择类型。数组擦除赋值单独覆盖引用组件到 Object、维数差和 primitive array，不放宽原 generic-only helper。receiver 提取先排除 primitive 字段，第二遍物理写匹配逐项收费、事实存储逐项收费；facade 方法完整性清单保留原 AnalysisSteps 计费。

重绑定与相同名字的分离局部作用域已有真实 Corretto8 压力 fixture。当前 renderer 把它们发射为同一局部的重复赋值；本片没有唯一声明/无重绑定许可，字段保持擦除，完整类仍可编译。这是尚未恢复的合法 Java 形，不能说成源程序非法；后续若扩展 alias 支持，应独立证明每个实际访问点，不放宽本片拒绝。

冻结四腿需要完整 subject/helper 反编译源、独立空 classpath 重编、JVM -Xverify、写入值身份/数组元素/数值，以及字段和方法分别的 GenericDeclaration 反射。反射 Method 的物理声明用 equals/声明类+name+descriptor 核对，不能用 Method 对象引用的 ==；class binder 可用唯一 Class 身份。

CallHold/ExceptionHold 当前正文已恢复，整类失败源于实际 Object 实参与已发布 T callee；这是调用适配/参数完整使用的独立片。此预审不扩大 constructor/shared candidate、方法 Signature 或字段调用 RHS 证明；root 对该片另行取证，不混入当前实现。
