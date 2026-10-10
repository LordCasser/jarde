## Context

两份fresh完整类基线各31命令/8腿：原双JDK2/2、JADX default/none4/4、当前Jarde2/2均重编执行并匹配原raw。literal的static数组已提升，ordered因非final `trace=0`被静态组拒绝；实例b均留构造器。见../../evidence/java-syntax-2026-10-10/array-field-initializers-literal与array-field-initializers。问题是呈现准入过宽拒绝，不是数组提取或执行语义错误。

## Goals / Non-Goals

**Goals:** 修正常量变量阶段判定，同时保持现有静态整组效果、物理身份、原子呈现与预算契约。

**Non-Goals:** 复用proposal所列边界；尤其不实现普通实例字段共同初始化、构造器链或部分静态组。

## Decisions

1. **只改已证字段flags下的阶段条件。** `prove_static_initializer_group`已核对source_fields/field_headers身份、flags、类型和完整表。对已知field_index直接复用原flags：只有final且RHS仍可能是常量表达式时，保留当前阶段拒绝。非final的常量RHS继续其他证明，不能新增is_final实体、通用常量分析或第二投影pass。

2. **以语言不变量区分阶段。** [JLS4.12.4](https://docs.oracle.com/javase/specs/jls/se8/html/jls-4.html#jls-4.12.4)将常量变量限定为final primitive/String加常量表达式；[JLS12.4.2](https://docs.oracle.com/javase/specs/jls/se8/html/jls-12.html#jls-12.4.2)区分常量变量与随后文本顺序初始化。非final声明`trace=0`仍是运行时赋值，不能沿用final阶段变化理由。已有RHS表达式遍历与效果BCI证明继续完整执行。

3. **复用原子投影。** 当前`project_static_initializer_group`已先发射全部fragment、计费、再一次附到字段声明；既有字段顺序通道省略根源码中的static块，同时保留原物理方法报告。此片不改writer/AST/报告平面。

4. **参考JADX但保持本架构证明。** 本地ExtractFieldInit的processStaticFields/collectFieldsInit/filterFieldsInit/addFieldInitAttr提供字段写与顺序参考；不引入其指令删改/shrink重试或表面重排。实例moveCommonFieldsInit需多构造器、this/super链和真实field身份的独立证明，单独记录。现有Rust机制无需外部库或新增依赖，避免无关许可与维护负担。

## Risks / Trade-offs

- final runtime常量被提前 → 用无ConstantValue的blank-final静态常量负例，继续整组拒绝；接口隐式final亦不放宽。
- 放宽阶段同时绕开效果/读写证明 → 只修改一处final条件，继承漏写/重复/额外效果/前向读/异常/ConstantValue与预算取消回归。
- 编译成功掩盖副作用变化 → 原/JADX/candidate完整source不修文本、空CP/SP、新classes -Xverify，逐raw三流核对；保留标量/数组求值次序、两次构造和整数溢出结果。

此slice不代表EM18整单元完成。CF16小栈溢出与实例字段提升另记，不混入此条件修正。
