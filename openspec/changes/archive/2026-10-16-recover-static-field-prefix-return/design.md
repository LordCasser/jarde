# Design

## Context

动机与用户行为见proposal及delta。当前实际产品ba3fa76af的冻结CLI c44bde2c…fa396来自4907项源码输入；这是本片冻结实施前的历史基线。两原FO的10B Code分别为 `b20007046059b30007ac` 与 `b20002046059b30002ac`，CP编号不同而目标同为FO.i:I；实际BCI为getstatic0、常量3、iadd4、dup5、putstatic6、ireturn9，无handler。完整FO输出在两腿均因bump说明体缺return而javac退出1；evalThenFin的临时值与finally已恢复，整个类尚未重建运行。

现有FieldIncrements只从DUP_X1证明实例八指令receiver形；void-only单位字段更新不承载新值返回，postfix snapshot承载旧值。新增独立driver原类控制共6命令、四次双JVM运行匹配60B六行oracle，不能把原类控制当修后验收。可移交证据见results/repro-v1。

## Goals / Non-Goals

**Goals:** 在现有字段/SSA/预算/Return接缝证明并呈现静态int新值返回，整链原子认领，复用物理来源与公共报告；完整FO两腿实际重编运行。

**Non-Goals:** 不把本例推广为任意dup消费者，不修改finally/局部作用域，不重写旧实例更新规则；详细能力边界见proposal，不从本片断言任意JAR/WAR等价。

## Decisions

1. **复用本项目语义证据。** 使用已有field@1读写声明、当前类字段可更新int核验、SSA/effects、预算meter和Return发射；外部库没有这些私有物理身份/SSA/来源，不新增依赖或通用更新框架，无许可维护面变化。固定窗口检查需预算与取消，不能把新增扫描挂进旧无Result的OnceCell计算。
2. **整链证书先于认领。** 候选完整同块、物理无clone/handler，getstatic/putstatic同owner/name/I/staticness并绑定本类nonfinal/nonvolatile声明。证明旧值与literal1是iadd输入，新sum是dup输入，dup的两份实际category1值分别且仅由写回与return消费；没有额外消费者或夹入效果。BCI列表只作fixture来源，不作通用准入常量。失败不改变原ownership/visited；停止传播原原因。
3. **新值不是再读取。** 在Return唯一发射一次前置更新，复用adapt_return。目标按现有静态Field节点与合法owner类型拼写，read保留字段子来源，write为更新主锚，常量/add/dup为派生来源，return保留自身主锚。优先采用最小真实前置更新AST节点及既有分组/遍历/预算发射；不仿照legacy实例把整句新文本塞进Local，不为迁移旧表示扩围。赋值后再getstatic的替代方案会多读字段，不能满足原消费者身份。
4. **兼容控制与拒绝分开。** 不放宽现有instance receiver、void单位更新或postfix旧值判据。构造异字段、错误新旧值、非一/非add、额外alias、效果/handler/跨块及缺失声明等边界；若其它既有规则能正确恢复，不把必须拒绝写成所有规则的全局限制。解析/方言/运行选择/verifier保持原管线，源码恢复成功必须另由完整编译执行证明。
5. **双编译腿完整重放。** 原FO.java与driver冻结，不改原类/Runner/oracle，不从产物删bump以单测finally。最终CLI每腿完整report.text原样编译，rebuilt-only执行固定main与独立driver，核0/0、8/8、MIN/MIN、8/100、MIN/100、1/100；保留所有失败及未执行步骤。可按需补当前JADX完整对照，JVM原类行为与证书才是判据。

### 实施期补充：owner的合法解析

两条独立手写javac控制已确认：同名int字段会让 `++PrefixOwnerShadow.i` 中的首标识符解析成字段，产生 `int cannot be dereferenced`。该控制不涉及FO或Jarde生成文本。`spell_reference` 可拼写并不等于表达式中的owner已正确绑定；现有命名阶段已保留静态owner首标识符以避免生成局部/参数遮蔽，但不能改真实字段。

本片继续使用有真实字段子节点的限定owner表达式。在认领前预算核对实际class_fields中的首标识符冲突；现有输入只能在已知Object直接父、无接口时排除未读继承字段的遮蔽。未知继承关系或冲突属于本新投影缺少合法owner证明，保留原拒绝；不改field@1或其它更新规则，不新建依赖展开/名称解析框架，也不偷偷以裸字段名或Local文本替代Field。FO满足这项证明前提。更一般的owner绑定须补齐证据后另扩准入，不能凭当前字段表没有冲突猜测继承表也没有冲突。

实施代码还借用同次读取的class attribute shells和当前方法唯一header，预算核实absence。当前投影没有泛型binder或源码成员类型名表，class的Signature/InnerClasses/EnclosingMethod、当前method的Signature，或未知/歧义header均按缺证保留拒绝；不把二进制`$`尾段当完整source成员名证明。其它方法的Signature不影响当前方法。该保守边界不声称已支持无冲突的泛型/成员类型，日后须以现有typed属性证据补齐绑定后扩展，不为本片新建通用resolver。

### 停止阶段边界

本片的“无部分更新”遵循既有提交边界：候选证明或正文发射停止，经Result传播而不提交半个更新正文及来源；不退款已准许费用。完整正文成功提交之后，可选evidence/source-map重放的较晚停止仍按现有契约保留已完整正文与明确的证据进度，不能宣称整个operation事务回滚。固定c075的静态组合核验见results/fixed-recovery-v1/stop-composition-static-review.json；新增动态实验是proof预算/取消，deadline和输出停止复用共同poll/checked writer机制，不冒称本片新增专例已经执行。

## Risks / Trade-offs

- [把copy的producer BCI相同误当两份值相同] → 核实际SSA输入/输出及唯一consumer，而非只凭opcode或comes_from(copy)放行。
- [字段类初始化、可见性或写回语义变化] → 首片限当前已证明本类普通int声明，读写各一次，非法/缺失/volatile/final保持现有拒绝或其它正确规则。
- [新AST漏掉来源或递归/发射路径] → 核所有相关遍历/优先级/借用与预算发射匹配点，加入最窄分组和source-map控制，不建立泛化框架。
- [正常首停被半证书覆盖] → 证明完成后才认领，真实证明期收费/取消和输出停止各自验证，不退款或重试。
- [FO辅助方法与finally错误混为一谈] → 保持完整类编译门和逐方法原因，FY/FX的独立finally缺口仍留原计划，不以此次修复冒称关闭。

## Migration Plan

先等reference片自身CI验收/归档/主线源码绑定，再应用本片；实施开始重取实际基线，冻结永久红例及控制，局部实现后运行定向兼容门、完整双腿源码对照和自身产品CI。只有真实验收后才勾tasks、同步delta并归档；失败保留，不把本规划或旧CI当实现成功。

## 固定宽度候选分流（实施修正）

c075自身CI的P5 bulk暴露了非候选六指令int方法也进入逐项计费扫描的问题。后继a89在任何新SSA证明收费前先核完整六opcode `getstatic/iconst_1/iadd/dup/putstatic/ireturn`，与既有单位更新候选分流的固定宽度做法一致。仅核首opcode不足以排除NestedEval.tick；不会把非候选的正常工作记到账上。真正候选仍逐项poll/charge，并保留原值身份、字段声明、来源及最后poll后才交证书的全部证明；没有改旧计费快照或退款。失败原始CI和新head完整重放分别见results/own-ci-failed-c075-v1与dispatch-local-v1。
