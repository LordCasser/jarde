## Context

见 `proposal.md` 与 [DT-04 冻结证据](../../evidence/java-syntax-2026-09-27/anonymous-inner-this/report.md)。当前 `ExprKind::QualifiedThis` 和 emitter 已能表达 `Inner.this`；`build.rs` 只在收到 `ProvedCapturedOuterRead` 时把同类 synthetic field read 转换为该节点。Facade 现有成员家族捕获证明面向具名非静态直接子类，匿名 child 的 `EnclosingMethod` 被明确拒绝。未投影的匿名 child 会保留 `this$0` 写入，而 Java 8 源语法不允许把该赋值写在 `super()` 之前。

## Goals / Non-Goals

**Goals:** 复用同次方法 AST、现有捕获字段语义和 `QualifiedThis` 输出；只对单个、封闭且完整的匿名外围实例捕获家族发布匿名源码；让全部依赖类源码可按 Java 8 重编。

**Non-Goals:** 恢复匿名父类的源级构造实参（DT-06a）、匿名类中的任意局部变量捕获、多个/跨类分配点、嵌套匿名类，以及一般化的构造器语句重排。此切片不把 `this$0` 变成新的通用表达式或 classpath 猜测入口。

## Decisions

1. **将匿名类作为已有类族投影的一种身份关系，而不是按 `$1` 名字识别。** 用所选物理定义上的 `EnclosingMethod`、匿名 `InnerClasses` self row、owner 方法身份及完整的 owner XRef 绑定外围类、匿名类与调用点；匿名编号本身不构成证明。相比把匿名类误标为具名成员，显式保留 `EnclosingMethod` 的物理事实可拒绝方法不匹配和多个分配点。
2. **精确证明一条外围捕获链，再复用现有 `QualifiedThis` handoff。** 验证唯一的 synthetic-final `this$0` 实例字段、匿名构造器的首个外围类型参数、`UninitializedThis` 上精确的 pre-super `putfield`，以及匿名方法内全部 receiver/read BCI。只有这些物理身份闭合后，才向恢复层提供相同形式的 captured-outer read fact；不把普通 `field@1` 的 `this.this$0` 自动推断成词法接收者。
3. **在分配点的同次 AST 中内联完整匿名体。** 用原 caller AST 中唯一的 `new` 表达式承载匿名 class body，并把已证明的 captured reads 写为 `Inner.this`。成功投影隐藏字段、合成构造器和前置字段写，因为匿名 Java 源码由语言替代这些 classfile 构件；不改变真实 classfile 的执行顺序。相比修复一般构造器 emitter，此决策把 Java 8 编译修复限制在已证明的匿名投影内。
4. **整个根文本仍原子发布。** 任何方法正文 fallback、匿名方法不完整、额外捕获、owner 引用未闭合、预算停止或取消都拒绝整次匿名投影并保留物理类输出；物理定义身份与独立查询不被匿名视图取代。可选 source-map 记录只解释已提交投影，不能作为准入条件。
5. **用三方 Java 8 编译/运行而不是文本相似度验收。** 重放固定原始 class SHA 和关键 BCI；将完整 JADX 源码与 Jarde 根源码在正确依赖下重编，启用 `-Xverify:all` 对照观察值。源级失败时不声称 Jarde 已验证运行等价。

## Risks / Trade-offs

- `[把局部捕获参数误认成 this$0]` → 用字段 descriptor、constructor slot、写入 BCI 和所有读取点证明同一捕获身份；存在别的 synthetic/用户字段时拒绝。
- `[存在第二个匿名分配却只投影一个]` → 对完整所选 owner 范围做闭合 XRef 和 BCI 扫描；多分配或范围不完整不内联。
- `[匿名源码无法解析所需类型或方法]` → 限制到已有源名与调用者 AST 可表达的类型/方法，完整方法集合编译前一次性发布；访问/名称不明时保留物理形式。
- `[误将 super 前的物理构造器写入当作普通 Java 语句]` → 只在完整匿名体投影内隐藏经证写入，不移动它；未证明投影继续暴露降级原因，不承诺可编译。
- `[与 DT-06a 共享匿名类外形而扩大父类构造参数范围]` → 本切片只证明外围捕获字段，不选择父类 overload、不移动调用者构造实参；相关检查与 fixture 仍由 DT-06a 变更验收。
