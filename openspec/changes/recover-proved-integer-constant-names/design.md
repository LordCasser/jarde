## Context

见 [proposal.md](proposal.md) 和 [CF-12 审计](../../evidence/java-syntax-2026-09-27/cf12-integer-switch/report.md)。`src/facade.rs` 已按字段表顺序读取每个字段自己的 `ConstantValue`，并让 `src/class_source.rs` 写出 `static final int LOW/HIGH`；方法恢复保有同次 AST 侧车。当前整数 `SwitchArm.keys` 是物理 key，`emit.rs::switch_key` 对 `int` 直接写十进制；`return` 中的 `ExprKind::Integer` 同样只写数值。固定 JADX 在 `ModVisitor.replaceConstKeys` 使用常量表替换 key，再由 `RegionGen.addCaseKey` 写字段 alias；其 `ConstStorage` 会处理重复值，但也可搜索全局字段。后者不能证明原 class 的别名关系，本任务只借用“先建立常量候选，再呈现标签”的顺序。

## Goals / Non-Goals

**Goals:** 在完整单类、完整字段表和已恢复方法 AST 中，以同次 `ConstantValue` 和源级名字闭合证明，把唯一字段名用于整数 case 与该 case 的直接整数返回；保持 case/return 的原始 BCI、key 与行为。未获证时仅保持现有数字文本。

**Non-Goals:** 一般常量传播、整个方法里所有数字的替换、跨类或继承字段、资源 ID、char/string/enum 标签、`<clinit>` 计算得出的值、改写方法级恢复、推断原始源码真的写了字段名。

## Decisions

1. **从已读字段事实建立有界候选，不解析生成文本。** 在类级字段读取时只选本物理定义、准确 `I` descriptor、`ACC_STATIC|ACC_FINAL`、可写 Java 字段名和有效 `ConstantValue` 的字段；要求字段表完整且候选值唯一，重复值从候选中剔除。声明已经成功写出才可引用。复用 reader 的 typed attribute 结果与现有预算，不重新读 class 或搜索外部 class。与 JADX 的类/全局常量表相比，这个范围更窄，但排除了同值全局误配。
2. **在同次 AST 侧车上作源码投影。** 方法级 `RecoveryReport` 保持不变。类装配阶段只查看完全恢复的整数 switch，其 case key 与直接 `return` 的整数叶各按数值独立配唯一候选；核对局部/参数名不会遮蔽所选字段，且该字段在当前根类位置可无歧义书写。以已有 `ClassSourceMethodAst` 的克隆、预算遍历、重新发射和 `source_text_with_method_projections` 模式实现，不用源码字符串替换。若既有 AST 形状不能表达无前缀的静态常量名，可增加一个专门的表达式叶和一个整数标签呈现分支；不要把字段伪装成 `Local`、把 key 改成字段值，或建立新公共 pass。
3. **候选失败仅退回数值，证明中止原子处理。** 同值字段、错 flags/属性、名称冲突、复杂 return、未产生完整 AST 都维持原文本。已证候选的表达式 origin 继续包含原 literal/return BCI，case 继续锚到原 switch BCI；字段身份由类字段记录和私有候选保留，不伪造方法内的字段读取指令。预算/取消先于最终源码发布完成或停止，不能留下半份投影。
4. **不引入依赖。** 本项目的 reader、`MemberDefault`、同次 AST、source-map 和预算接口已足够；外部反编译/CFG 库既不能提供本次类字段物理身份，也不能替代源级名字与原子输出证明。

## Risks / Trade-offs

- [同值数值不等于原始源码字段使用] → 把字段名明确视为源码质量投影，原物理 key/value 和 BCI 不变；同类唯一性是可复现的规范选择，不声称反推出原 token。
- [字段名被参数或局部遮蔽，或同值字段歧义] → 逐方法核对词法名字与字段表；不能安全写 `LOW` 时维持 `2748`，不依靠编译失败再回退。
- [case 标签类型与返回位置不相容] → 仅 int selector 和直接 int 返回；char、强转、嵌套算式留在既有规则。
- [AST 投影影响其它类级投影或来源] → 在现有类级 staging 顺序中只对普通根类投影；字段、方法物理报告保持原样，定向检查来源与预算/取消。
