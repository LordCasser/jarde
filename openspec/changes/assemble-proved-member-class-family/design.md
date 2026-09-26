## Context

见 [proposal.md](proposal.md)、[行为合同](specs/java8-recovery/spec.md)、[来源合同](specs/source-maps/spec.md)与[家族装配审计](../../evidence/java-syntax-2026-09-26/named-member-outer-receiver/family-assembly-analysis.md)。当前 `Engine::class_source_with_evidence` 只绑定并恢复一个物理定义；`ClassSourceAssemblyContext` 已有 typed 嵌套属性，但 `ClassSourceReport` 和 `class_source::source_text` 仍是一类一文本。已实现的 `member_inner::prove_target` 面向公开构造调用，其 raw `ACC_PUBLIC` 门槛会拒绝 javac 的 package-private 成员及构造器，不能充当源码声明证明。

JADX 的 `RootNode.initInnerClasses` 建树、`ClassGen.addClassBody` 递归呈现的顺序可借鉴；`ClassModifier.removeSyntheticFields` 只以 parent 类型、构造器首参及入口 `IPUT` 推断捕获，缺唯一性和完整值流核对。Jarde 继续使用 reader 的 typed 属性、精确物理身份、SSA 和同一预算，不引入 JADX 或新的通用反编译 pass。

## Goals / Non-Goals

**Goals:** 一个根与一个直接命名非静态成员的闭合子集；在单一请求中保留两份物理报告，输出一个嵌套 Java source unit；证明捕获参数、字段、读取及本家族内构造调用；可观察顺序、异常处理、来源、预算和停止均闭合。

**Non-Goals:** `Outer.super` 的 synthetic 方法桥、local/anonymous、静态嵌套、泛型外层作用域、`this()` 构造委托、多构造器、跨多个 Java source unit 的工程装配或无证据地隐藏任意 synthetic 成员。合法但不满足此子集的输入保留物理结果和拒绝，不升级为可编译家族。

## Decisions

1. **从已选根定义发现成员，不按名称搜索。** 根绑定后读取双方的 `InnerClasses` 与 `EnclosingMethod`，用同一个请求环境精确解析 child `PhysicalDefinitionId`；双方唯一 self row 的 child/outer/name/flags 必须一致，并确认 child 无 local/anonymous 身份。源码修饰符取已证的成员关系属性，不取 child 原始 class flags。解析缺失、多定义、属性冲突或物理视图不完整时拒绝这次家族投影。替代方案是按 `$` 拆分路径或在所有 class 中寻找同名候选；它会把二进制名约定误当语言身份，也可能选错 loader 或 MR 变体。

2. **复用单类准备与恢复，再做私有家族投影。** 将现有根类读入、声明/字段/方法恢复的工作提取为可对一个已绑定定义顺序执行的内部过程；root 与 child 消费同一 `Budget`，保留每个物理 `ClassSourceReport` 的字段、方法、coverage、execution 和 source map。根报告可直接包含 child 的物理报告，不另建公共 Session 或全程序 class graph。类家族证明只在两份报告均可检查后进行，原报告不因源码隐藏而删掉物理成员。替代方案是递归调用公开 class-source 再拼字符串；公开调用各自封口执行状态，不能原子合并来源/停止，封口文本也没有内层 Java 作用域。

3. **以 SSA 闭包证明捕获，再交给已有构造与值呈现。** child 必须有唯一可解释的 `<init>(Outer, …)`、唯一目标 Outer 类型的 synthetic/final 字段，并证明首个物理参数作为同一 SSA 值在构造 prologue 写入该字段，接收者是构造中的 child `this`。对本家族中将被改写的每个字段读、构造调用逐点核对 field/ctor 物理身份、receiver 值流、空值检查、参数顺序、异常覆盖和额外使用；同类型 `other` 不命中捕获证书。`member_inner` 原来的公开跨类调用证明仍可单独成立；同家族成员可访问性应由已证嵌套关系与 Java 词法位置决定，不能仅删除 `ACC_PUBLIC` 检查。替代方案是名字为 `this$0` 就删字段，或将所有 Outer 类型值写为 `Outer.this`；同类型参数对照已否定这两种捷径。

4. **从结构写一次文本，保留投影与物理两个平面。** 将 class-source writer 抽出“包/顶层头”与“类声明/成员体”两层，根只写一次 package，成员在根体内按关系拼简单名和修饰符；字段/构造器隐藏以通过证明的物理 identity 集合驱动，不编辑已生成字符串。根报告的 `text` 是家族文本，child 报告仍保存其原始物理身份和独立恢复结果；派生投影记录绑定源码位置与原字段、物理 ctor 参数/写入和 child 方法 BCI，两个 owner 的相同索引不能碰撞。若投影修改某个 method body，保留原 `RecoveryReport` 并单列其投影关系，不把未修改的 source map 冒充新文本的坐标。替代方案是把 child 完整 `source_text` 剪掉头尾后插入 root，无法可靠处理 package、注解、构造名、缩进、预算和来源。

5. **证明完成后才发布完整家族；停止保持前缀。** 在发现、解析、扫描、恢复与输出的每个可增长阶段 poll/charge。家族证书不闭合时不隐藏任何物理构件，也不把部分嵌套文本标为完整；保留 root/child 已取得的记录、拒绝和各自覆盖。独立 `recover_method` 不触发家族发现。冻结正例须在有/无调试信息下，以 Java 8 重编、`-Xverify:all` 的值、效果和异常对照原/JADX；负例用 verifier-valid 变体或直接 proof-unit 明确证据等级。JADX 结果用于检查表达目标，不代替 classfile 与 JVM 判据。

## Risks / Trade-offs

- [隐藏字段后有家族外物理消费者] → 在选定输入可见范围内检查结构引用；存在已知外部消费者时拒绝可编译家族承诺。无法证明开放世界不存在消费者时只声明本 source unit 与冻结执行范围，不声称整 jar 或任意外部链接兼容。
- [同名 class 来自不同 loader、archive 或 MR 版本] → 跟随请求环境的精确物理定义解析；不得合并两份同名定义的成员索引/BCI。
- [构造捕获写入、super 调用或异常区位置不等价] → 先做全段顺序/处理器证明；不成立时引用原方法，不能只删除 `this$0` 一行。
- [child 某成员停止导致看似完整的根文本] → 将 child execution/coverage 与家族投影状态分别呈现；不可用根原有 complete 标志代表 child 已完成。
- [`access$` 方法桥不能写为 Java] → 在本变更明示拒绝完整家族；后续独立证明桥 body、捕获 receiver 和全部使用闭包后再投影 `Outer.super`。

## Migration Plan

公开 `class-source` 的已证明家族文本/JSON 为有意的 breaking 结果形态。先以冻结正例和拒绝变体验证物理报告不丢失，再接入默认/完整证据模式；无家族证书的类沿用单类文本。若后续发现投影不健全，关闭该窄证书的准入即可回到保守物理呈现，原 class/成员报告不需要迁移。
