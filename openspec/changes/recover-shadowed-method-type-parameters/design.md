## Context

见 [proposal](proposal.md) 与[三方冻结证据](../../evidence/java-syntax-2026-09-25/method-type-variable-shadowing/analysis.md)。reader 已把类和方法 `Signature` 解析为带变量声明、界、参数与返回的结构树，并核对物理 descriptor/`Exceptions`。`class_source` 只把已成功发布的类头作用域交给方法；静态直接参数返回的 AST/SSA 候选与泛型方法拼写已存在。当前 `prove_method_signature_erasure_with_class_scope` 把类/方法变量放在同一禁止重名空间，`erase_type` 还先查类变量；两处都与 Java 方法变量遮蔽规则相反。

## Goals / Non-Goals

**Goals:** 在 reader 的同一证明入口修正类/方法两层查找顺序，使静态直接返回方法的既有投影能够保留 `<T>` 或 `<T extends CharSequence>`；让冻结的两份完整类、原强类型调用方在 Java 8 下重编与执行一致，并保留类/方法各自的反射签名。

**Non-Goals:** 不对方法正文做新的泛型值流推断，不开放非静态方法或覆写绑定；不处理成员内类继承外层变量、带参数化界、未知类型层级或调用点泛型实例化。此变更不修复 JADX 的实现，也不把其输出当验证依据。

## Decisions

1. **作用域由内到外查找，重复只在同层拒绝。** 方法 `type_parameters` 内继续拒绝重复名并保持界循环检测；类 `type_parameters` 已由类证明独立拒绝重复。方法变量与类变量同名合法，`erase_type` 先在方法局部表中查找，再在已证类表中查找。`validate_type_variables` 的“任一作用域存在”验证不决定擦除，仍须由上述顺序完成绑定。与新增带 owner 的全局类型变量实体相比，这里只有两层固定词法关系，现有方法/类表已足够；嵌套类跨多层 owner 的身份问题另案处理。
2. **所有物理位置沿现有擦除门逐项检查。** 方法变量第一界为 `CharSequence`、类变量第一界为 `Number` 时，`(TT;)TT;` 应对物理 `(Ljava/lang/CharSequence;)Ljava/lang/CharSequence;`，而不是对 `Number`。参数、返回、throws 以及变量界里的递归引用均调用同一优先级；若签名经变造与 descriptor 或 `Exceptions` 不符，沿现有 `jvm_signature_erasure_mismatch` 或作用域错误拒绝，不为重编而修补签名。JADX `SignatureProcessor` 在本样本拒绝异界方法签名，说明其检查顺序不能照搬。
3. **source 层不新增候选。** 冻结方法是静态、方法自有变量、同轮 AST/SSA 证明直接返回参数、无其它复杂调用；`generic_method_declaration` 已接受这一形状并按方法签名拼写变量界、参数和返回。reader 证明通过后复用此门原子发布声明；类头仍在它自己的证明成功后才提供作用域。若实践发现 source 层另有实际阻断，只允许修复此现有拼写/发布接缝，不扩到其它正文形状。
4. **受控重编与负例并行。** `replay.py` 固定有无 debug 表的原始 class、JADX/Jarde 文本和强类型调用方诊断；实现验收重新生成 Jarde 源码并重编两个完整类与未改的 StrongCaller，执行并核查反射泛型字符串。reader 单元控制方法同层重名、未绑定、循环与错擦除；原类 T 与方法 U 不同名的既有测试必须保持通过。预算、取消沿原 parse/proof 入口，不新读 class 或依赖。

## Risks / Trade-offs

- **只删同名拒绝而未改查找顺序会把方法 T 擦成类 T** → 用异界正例和错擦除负例同时测 `CharSequence` 与 `Number`，并检查方法参数/返回的物理 descriptor。
- **类变量原有引用被方法变量无意覆盖** → 查找只在该方法确实声明同名变量时优先局部；不同名类变量成员沿现有回归证明。
- **方法泛型会影响调用点重载选择** → 只使用现有静态直接参数返回及本类调用绑定门，独立强类型调用方对生成类重编；未证复杂正文继续拒绝。
