## Context

见 [proposal.md](proposal.md) 与[简单成员类证据](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/analysis.md)。`UseInner.make` 的字节码是 `new; dup; aload outer; dup; Objects.requireNonNull; pop; mark; invokespecial (Outer;I)V`，当前 `init::verify` 在第二个 `dup` 拒绝 `jre_new_interleaved_effect`，`New { ty, args }` 也无法表达限定接收者。已有[效果顺序对照](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/negative-controls/analysis.md)证明：空接收者须阻止普通实参效果，表达式前的 `P` 效果须保留，嵌套 `B` 再 `A` 的效果不能交换。JADX 1.5.6 在这些非泛型样本可重编，但其泛型外层调用另有不可编译输出；本变更不把 JADX 文本当证明。

`jarde-reader` 已有 typed `InnerClasses`/`EnclosingMethod`，class-source 装配已有私有类级 nesting handoff；`jarde-java` 只持有本方法的 SSA/操作与同类成员事实。`class_source` 的 facade 则持有请求环境和物理绑定，可按预算读取被调用的目标类。职责应留在这两层之间，不能让方法 IR 或 CLI 自行查找 `$` 名称。

## Goals / Non-Goals

**Goals:**

- 让公开可访问、非泛型的成员类目标从选中环境得到唯一物理定义；调用点只消费已确认的私有关系事实。
- 在 `new@1` 的既有候选中证明实例、隐式外层首参、空值检查和普通实参的顺序，并将其写成一个带限定接收者的 `New` 表达式。
- 对任何缺证步骤保持原子拒绝、来源及完整预算/取消语义；普通 `new Type(args)` 走原证明路径。

**Non-Goals:**

- 嵌套类声明与整个 jar 的单份可编译源码装配，外层/内层泛型作用域、匿名或局部类、非公开访问关系。
- 通过 `$` 名称、首参类型或 `Objects.requireNonNull` 的出现单独推断成员关系；不复制 JADX 的代码或引入新的通用 class graph/IR/pass。

## Decisions

1. **目标事实由环境层按需确认，再通过窄的内部 handoff 交给恢复层。** 仅 class-source 成员恢复路径沿现有 `resolve_class_source_dependency` 所用的 `resolve_symbol`/`read_definition` 选择链，在同一已选物理环境中按构造调用的准确目标名读取唯一 class；预算计入该读取，缺失/多定义/停止分别保留为不能证明的结果。`new` 与 `<init>` 的准确 owner 匹配是候选，二进制名含 `$` 只限制按需读取，不证明成员关系。目标自身的 `InnerClasses` 条目须指向准确外层、具有效 Java 简名且非 `static`，`EnclosingMethod` 不得表明局部/匿名身份；声明、构造器和访问位限制在本首片可独立重编的公开非泛型子集。用目标的物理构造器 descriptor、唯一的合成外层捕获字段和 prologue 中首参对该字段的写入互证隐式外层参数。还须在选定外层 class 的 `InnerClasses` 表内找到唯一、同名同外层同访问位的目标条目；只改变外层条目而保留目标条目及调用方字节时，JVM 仍可执行，`javac --release 8` 却无法编译 `outer.new Inner(...)`（[字节负控](../../evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/invalid-controls/byte-variants/analysis.md)）。调用方同名 `InnerClasses` 条目若存在须交叉核验；JVM 校验可接受调用方缺失该属性的 class，故不能把该条目作为目标读取前提。`RecoveryRequest` 只接收这一站点所需的已绑定事实；普通 method-only 请求若无此事实继续拒绝。备选的按 `$` 拆名字推断成员身份或把目标类装进通用方法 IR 会混淆符号名、物理解析和方法分析，故不采用。

2. **在既有 `new@1` 内增加受限成员形状，而不放宽普通形状。** `new/dup`、准确 owner 的 `<init>`、被初始化实例的唯一消费沿用现有证明。成员分支要求物理首参的 SSA 身份与限定接收者同源，且仅接受可在当前位置发射一次的局部/`this` 值；其后的 `dup; invokestatic java/util/Objects.requireNonNull(Object)Object; pop` 必须是无额外消费者的准确栈形、位于普通参数效果之前，并与构造表达式处于同一可呈现异常区域。普通实参逐一按原栈顺序绑定，区间内不得有未归属的效果、重复读取、额外转移或重排。特别是现有 `produces_a_read_value` 对无栈结果调用返回 true，成员分支不能据此把独立的 `void` 调用当作普通参数值；这类调用须有直接消费证明，否则拒绝。成功时由限定 `new` 自身的 Java 8 空值检查承担已证明的检查，原检查 BCI 归入该站点来源；方法中更早的 `P` 留在原位置。这里可借鉴 JADX 将物理首参与源级实参分开的思路，但其 `SKIP_FIRST_ARG`/类型相等门没有证明 SSA 身份及时间关系，不能作为 Jarde 准入。

   本地 JADX 1.5.6 的 `ConstructorVisitor::processInvoke` 先把 `<init>` 调用换成 `ConstructorInsn`，并移除 `NEW_INSTANCE`；`InsnGen::addOuterClassInstance` 在方法标了 `SKIP_FIRST_ARG`、类是 inner 且首参静态类型等于外层类时输出 `outer.new`，后续 `generateMethodArguments` 跳过物理首参。这给出可复用的“构造调用先统一、输出参数按隐式前缀切片”顺序，但没有校验该首参就是被空值检查的 SSA 副本，也没有校验检查与普通参数效果的先后。独立的[普通 `new` 副作用反例](../../evidence/java-syntax-2026-09-25/ordinary-new-void-effect/analysis.md)还证明：提前折叠构造时，JADX 可将原 class 的 `CST` 改写成 `SCT`。因此本变更保留物理参数和 BCI 证据，先在 Jarde 现有 `new@1` 证明整段，再让 AST 仅隐藏已证明的隐式前缀。

3. **只扩展现有 `New` 的表达式形状。** 为 `ExprKind::New` 加一个可选限定表达式及经过 `InnerClasses` 证明的简单成员名，发射 `qualifier.new Inner(args)`；普通分支仍发射 `new Type(args)`。`new_expr` 将物理首参渲染为限定值、从源级参数列表剔除，同时按构造器 descriptor 的剩余位置校验普通实参。所有涉及表达式遍历、类型、来源和预算的既有消费者须覆盖新字段；`NewRecord.arguments` 仍列物理实参 BCI，不让报告悄悄改成源级位置。备选的发射后文本重写不能保证绑定、来源和停止边界，故不采用。

4. **先建立错形反例，再开放准入。** 现有 javac 正例不可能生成“限定值与物理首参不同身份”或伪造的 `InnerClasses`；任务先用受控 class 字节变体构造 verifier-valid 的身份/关系负例，并验证运行差异或拒绝。无法构成 verifier-valid 变体时，仍以直接 proof-unit 测试穷尽相应门，明确记录其证据级别；不会把推测当已证反例。测试以原 class、JADX、Jarde 的 Java 8 重编及 `-Xverify:all` 值/trace 对照验收，源码只对调用方单类重编并引用冻结的原成员类。生产实现复用现有 reader/SSA/预算和类装配，暂不引入第三方库；对 JADX 仅比较算法与结果，不复制实现或引入许可维护负担。

## Risks / Trade-offs

- [目标类字节来自错误物理变体] → 只用请求的环境选择、真实定义身份及准确构造器 descriptor，不在全快照按名字捞任意 class；缺证即拒绝。
- [显式 null-check 折叠改变效果/异常] → 限制准确 SSA 栈形、同一异常区域和普通参数前的检查，并用 null、嵌套效果、前置效果三组运行对照验证；扩展到有副作用限定表达式留待另案。
- [局部 AST 形状扩展漏掉遍历器] → 搜索 `ExprKind::New` 的所有消费者，编译穷尽匹配，并定向检查来源、预算和取消。
- [只有调用点恢复却把整 jar 称为可编译] → 明确使用原目标类作为编译依赖；泛型和成员声明投影保留单独路线图行。
