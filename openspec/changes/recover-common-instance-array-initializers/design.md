## Context

动机见proposal。instance-field-init-next的完整三类+Runner基线为31命令/8腿，root verifier v7已接受：原/Jarde各2/2 raw相同；JADX四腿编译成功却在DifferentRHS运行失败。不是提取数组本身缺失，而是共同实例初值的呈现差距。JADX TestArrayInitField是原锚点，多ctor/this链/不同RHS组合明确属于扩展控制。

当前普通静态组已用ClassInitializerCandidates处理；普通实例没有共同前缀证明。ClassSourceMethodAst的完整Program在jarde-java内opaque，body consumer列表过滤了顶层statement，不能靠它或BCI顺序猜连续前缀。该sidecar已有完整Code/exception、instruction BCIs、call_targets和init prologue，field@1的已证Evidence在同轮构建时可取得。class_source::projected_body_method_text已有通用方法装配入口，facade的member_texts/staged_member_emissions已有原子派生呈现，不需要伪造NestedAnonymousExpression。

## Goals / Non-Goals

**Goals:** 全部已闭合direct-super ctor共同数组前缀一次声明提升，保持每实例新数组、super/元素/ctor主体次序与异常语义、物理身份/BCI和预算停止。

**Non-Goals:** 复用proposal边界。第一MVP只准一维primitive NewArray的已恢复initializers，元素为已知primitive字面值/必要primitive cast，或参数也是这些常量的same-class static primitive-return调用。不准嵌套数组、reference协变、field/this/parameter/local读、新对象、任意算术或动态长度；super(args)的原有参数求值保留，不要求Object()V特例。任何this ctor整组退出，与基线双方当前正确行为一致。

## Decisions

1. **从JLS执行位置证明，而非移行文本。** [JLS12.5](https://docs.oracle.com/javase/specs/jls/se8/html/jls-12.html#jls-12.5)步骤3/4/5依次为super、实例初值、ctor剩余主体。只取每个完整ctor开头已证super调用后紧邻的连续this.field数组写序列；无intervening statement，序列数目/字段序列/RHS必须全同。随后各ctor主体可不同，原样保留。不从有条件/try内部取写、不滑过其他effect，不能只处理某个ctor。

2. **保留必要field身份，不再建第二候选框架。** opaque AST内部若尚无同轮field@1写身份，可在仅constructor sidecar保留所需已claim的crate-private field::Evidence，复用其owner/name/descriptor/is_static/bci；按node/记录字节计费并poll。adapter在jarde-java内验证顶层节点和claims，跨crate仅返回最小候选/已派生body，AST internals不开放。不得扩义ClassInitializerCandidates或enum候选，不新增pass/global framework/公开report schema。现有AST与formatter已满足能力，无库缺口、依赖或代码复制；参考JADX算法，不引入其GPL代码/删指令后shrink重试。

3. **强RHS相等只忽略物理origin。** 同类型/维度/长度/元素数目与顺序；primitive字面值含完整long/raw float/double bits和presented type、cast类型和值逐项相等。每个call须与自身physical call target唯一join，invokestatic opcode、owner/name/descriptor完整相同，所有实际参数递归比值，不以文本/名字/形状相等取代证明。只准当前类已唯一确认static定义、primitive return及无Exceptions attribute的目标，避免改变重编解析或checked exception上下文。DifferentRHS mark31/32必须失败。本地dev JADX isSame软比较为候选漏洞，未验证1.5.6内部图，不照搬它或声称确定二进制根因。

4. **完整class census和字段顺序。** facade核method/field表完整、每个physical ctor恰一个同轮complete/Structured/无fallback/无handlers AST，无其他投影先占ctor；array字段必须同class唯一匹配完整owner/name/descriptor、nonstatic、合法已有declaration，数组类型与descriptor一致，单个prefix唯一写且该field在ctor剩余body无重复写。任意ctor漏写/不同序列/不同RHS整组不投影。final数组不成为常量变量，可沿相同证明；scalar不在范围。初值字段必须与当前实际root field order一致（包括已有static projection之后的顺序），无需新field重排。不扰static声明或clinit逻辑。任意nonstatic字段已声明ConstantValue时整组退出（复用class_source::declares_constant_value），任何候选字段已有初值亦不覆盖：当前writer会把该属性写成声明初值，不能在未证明其他实例初值次序时搬移。此guard不修该独立旧路径。

5. **同轮AST派生、一次提交。** emit原完整ctor AST并与它当前physical artifact/placed text核对，避免覆写已由其他projection修改的body。先发射所有field RHS与仅去prefix写的ctor bodies，用既有通用projected_body_method_text装配、计费全部输出；所有成员/field有效并未Stop后才同时发布字段初值与root member_texts。physical methods的RecoveryReport/text/source_map和原BCI保留，不将精简constructor body当物理原报告，不添加误导匿名kind。保留prologue及全部suffix statement，包括最终return。任何Refused/Stop不发布部分field/ctor edits，继承现有执行/诊断停止通道，取消优先。

6. **验收直接复用完整基线，不重造oracle。** 新CLI冻结后对现有literal/ordered与三类实例基线双JDK原样完整source重编/new classes -Xverify，比较原始exit/stdout/stderr。正例升字段且两ctor不重复调用；this与DifferentRHS保持原写，各实例数组身份与trace相同。另以少量完整控制覆盖漏写、重复、intervening effect、顺序冲突、参数依赖、异常scope、final数组和两字段prefix。公共Stop测试证明无半发布，不以总预算推断构造中途私有rollback。

## Risks / Trade-offs

- 同形不同参数误提升 → strong值/完整call target比较及已捕获31/32反例。
- super前后或两元素effect重排 → strict AST顶层前缀/实际字段顺序和完整runner raw比较；无new reordering。
- 选读证据改变产品准入 → 同轮sidecar independent of RuleDetails，default/all生成正文相同。
- 部分ctor无法恢复却提升字段 → 完整method census失败整组保留。
- 与其它projection冲突 → 核现有placed text，无合法统一当前body则不改；无重新获取旧AST覆写。
- 新事实成本/Stop中途半提交 → 有界遍历/逐项计费/poll和全部输出先stage，final commit一次。

## Migration Plan

先独立确认static片A1指纹登记修复提交的确切CI，同时Luna准备本片patch但不应用。root审查、唯一工具链执行、冻结新CLI并完成双JDK/旧回归/准确新CI；任务按实际结果勾选。仅清本仓target，20GiB/1GiB守卫不降；historical raw和旧CLI全保留。
