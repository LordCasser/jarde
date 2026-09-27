## MODIFIED Requirements

### Requirement: Complete class source preserves proved method-local generic signatures

对于完整恢复且有方法 `Signature` 的 Java 8 方法，系统 SHALL 在 reader 证明签名作用域与物理 descriptor 擦除相符、同轮方法候选证明正文值来源、声明及调用绑定能够保留时，写出精确参数化返回类型。顶级普通类的公有无参实例 `List<String>` 方法若正文准确、无效果地直接返回 `null`，SHALL 消费既有同轮 null 候选并在完整类源码中保留该返回 `Signature`。证据不足或停止时 MUST 保留物理声明和可查报告，不得只凭最终文本补写泛型头。

#### Scenario: Parameterized List method returns direct null
- **WHEN** 顶级普通 `Object` 子类的公有无参实例方法具有唯一 `Signature` `()Ljava/util/List<Ljava/lang/String;>;`、物理 descriptor `()Ljava/util/List;`，完整 Code/AST/SSA 候选证明准确无效果 `aconst_null; areturn`，且同名调用绑定无冲突
- **THEN** 完整类源码 SHALL 写 `java.util.List<java.lang.String> empty()` 和 `return null;`；原、JADX、Jarde 完整类型与同一消费者 SHALL 通过 Java 8 重编及 `-Xverify:all`，返回泛型反射结果 SHALL 一致

#### Scenario: Raw control and existing parameter return
- **WHEN** 同类还含无 `Signature` 的 raw `List` 字段/方法及已证的 `List<String> id(List<String>)` 参数直接返回
- **THEN** raw 类型 SHALL 保持 raw，既有泛型字段与 `id` SHALL 保持原有投影；null 返回的准入不得改变邻近成员

#### Scenario: Return signature or body proof is incomplete
- **WHEN** `Signature` 擦除不符、不属于该固定参数化返回形态、正文存在额外效果或 handler、同名本类调用绑定未证，或候选/输出遇到预算耗尽、取消
- **THEN** 系统 MUST 拒绝泛型声明投影或传播停止，不得发布半个方法头；物理 descriptor 声明、拒绝来源和独立方法报告仍 SHALL 可查
