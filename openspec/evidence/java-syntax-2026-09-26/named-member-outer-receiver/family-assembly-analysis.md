# 命名成员类的源码家族边界（2026-09-26）

本笔记依据同目录 `variants/OuterReceiverCases.java` 的 Java 8 class、`javap-Outer.txt`、`javap-Member.txt` 与三方重放，以及本地 JADX 源码审计。该样本把 `other.state`、词法 `OuterReceiverCases.this.state`、`OuterReceiverCases.super.value()`、`Member.this.value()` 分别做成可辨结果；原版和 JADX 完整类重编运行都输出 `20:10:1:3`。单靠接收者类型都是 `OuterReceiverCases` 无法区分 `other` 与捕获的外层 `this`。

## 当前装配边界

`Engine::class_source_with_evidence` 在 `src/facade.rs:1308` 精确绑定一个物理定义，`src/facade.rs:3174` 最后只调用一次 `class_source::source_text`；`ClassSourceReport` 和 `src/class_source.rs:5893` 的文本生成均以单物理类为单位。已有 `ClassSourceAssemblyContext`（`src/class_source.rs:122`）是所选类的 typed `InnerClasses` / `EnclosingMethod` 事实，并不是类家族树。现有 `member_inner::prove_target`（`src/member_inner.rs:49–108`）证明的是一类构造调用目标，而且要求 target 原始 class flags 和构造器 flags 都有 `ACC_PUBLIC`；冻结 `Member` 的 class flags 只有 `ACC_SUPER`、构造器 flags 为零，成员可见性在双向 `InnerClasses` row。因此不能拿这份调用点证书直接当声明装配证书。

最小可重编单元是一个顶层 `OuterReceiverCases` 声明，其中嵌入已证明的 `Member` 声明。`ReceiverBase` 与 `ReceiverMemberBase` 仍是独立顶层依赖。应在根类绑定、读取 typed nesting facts 后，由同一 `Budget` 和 `Execution` 顺序读取被精确确定的 child `PhysicalDefinitionId`，复用单类的 prepare/recover 工作形成家族树，再一次呈现源码。递归调用公开 `class_source_with_evidence` 会重置预算、停止状态和报告，拼接多个已经封口的 class text 也不能表达 Java 嵌套作用域。

家族证明至少核对 parent/child 两侧唯一一致的 `InnerClasses` self row、outer row、`inner_name`、child `this_class` 与同一物理解析环境；拒绝重复/冲突 row、`EnclosingMethod` 的 local/anonymous、含糊或缺失的定义、两个物理定义争同一 Java 全名。字段、方法、`RecoveryReport` 与 source map 始终保留各自的物理 owner；`PhysicalMethodId` 已含 owner，不按表索引或名字合并。若新增家族文本投影映射，需同时锚定 caller 的物理 owner/method/BCI 与被投影 helper 的物理方法/BCI，不能把 child 来源伪装为根类来源。每次发现、读取、扫描、输出均须沿用现有 poll/charge；取消、预算不足、解析歧义与部分读取均 fail closed。

## 隐藏合成构件的证明顺序

`Member.this$0` 是 synthetic/final，descriptor 精确为 Outer；物理 `<init>(Outer)` 的首参也精确为 Outer，构造器 BCI 2 把该实参写进该字段。但名称、类型、flags 或 `MethodParameters` 各自都不是隐藏字段的许可。必须用 SSA 证明唯一捕获字段和唯一构造器参数来自同一个值，字段写入接收者是正在构造的 child `this`，写入/读取/构造调用的完整闭包及其执行顺序与投影等价；然后才能在源码中省略 `this$0`、首个物理参数和捕获写入。其他 child 方法对捕获字段的读取须沿 def-use 到达该值；同类型 `other` 不能替换。第一片先拒绝 `this()` 构造链、多构造器、多捕获候选、未对齐的参数注解与泛型外层作用域。

`access$101(Outer x)` 是更严格的边界：其字节码在 Outer 内以 `invokespecial ReceiverBase.value` 调用 Outer 的直接父类。Java 源无法把该静态 helper 原样写成对任意 `x` 的 `super` 调用。只有证实 helper 的唯一 synthetic/static 定义及完整方法体，并遍历全部物理调用/引用（含可能的 method handle/bootstrap），逐点证明实参就是被捕获的 Outer、顺序和类型不变，才能将调用点投影成成员体内的 `OuterReceiverCases.super.value()`，同时仅在源码文本中省略 helper。闭包不完整时仍保留物理证据并拒绝“可编译家族”承诺。`access$000` 字段 getter 可先作为合法 helper 保留；字段内联另立证明。现有 `accessor@1` 只处理同类字段桥，`special_receiver` 只处理当前方法 entry-this 与当前直接父类，均不能替代上述跨类证书。

## 与 JADX 的关系和实施顺序

本地 JADX 的 `RootNode.initInnerClasses` 先建父子树，`ClassGen.addClassBody` 递归写入 child，`ClassModifier.removeSyntheticFields` 先处理 child 再清理合成字段；这个发现、恢复、呈现的顺序可参考。其字段消除主要依据 synthetic 字段类型等于 parent、构造器首参类型相同且入口首条 `IPUT`，没有证明字段唯一性、全部使用和完整构造控制流，不应直接移植。`InsnGen.callSuper` 沿类链选择可赋值 owner 输出 `Class.super`，能解释样本输出，但 Jarde 仍应证明被捕获接收者与 Outer **直接**父类的精确 `invokespecial` 目标。

分两项 OpenSpec 实施：先做 named-member 家族身份、构造捕获、`Outer.this`、嵌套声明，并在不含 `access$101` 的对照类上可编译重放；此阶段允许保留普通字段 getter helper。再做 `access$` 方法桥的完整用法闭包与 `Outer.super` 投影。第二项是完整样本 `20:10:1:3` 可编译的必要条件，不能提前宣称第一项覆盖了该样本。local/anonymous、静态嵌套捕获、泛型外层作用域、构造委托、多命名空间 source bundle 分开追踪，不混入首项。
