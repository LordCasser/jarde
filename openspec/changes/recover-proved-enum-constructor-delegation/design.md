## Context

见 [proposal](proposal.md)、[行为规格](specs/java8-recovery/spec.md)和[三方证据](../../evidence/java-syntax-2026-09-25/enum-constructor-delegation/analysis.md)。基础 [recover-proved-enum-constants](../recover-proved-enum-constants/verification-root.md) 已独立验收，只覆盖两个常量直接调用主枚举唯一的 `(String,int,int)` 物理构造器，并在后者完成隐式 `Enum(String,int)` 调用后保存一个源整数参数。新证据的 `ZERO` 调用 `(String,int)`，其 Code 以同一 `this`、原 name/ordinal 和常量 `0` 委托到 `(String,int,int)`；`ONE` 直接调用后者。终端构造器还先调用用户 helper，再保存字段。两条物理构造器的 `Signature` 分别是源码 `()V` 与 `(I)V`，当前直接与物理描述符比擦除会拒绝。JADX 1.5.6 的 `EnumVisitor.processConstructorInsn` 能从常量构造点定位所调重载，`markArgsForSkip` 随后跳过前两个注入参数；这个定位顺序值得复用，但无条件跳参并不能证明委托构造器是否原样转发 name/ordinal，Jarde 要沿调用边和参数来源逐项确认。[九组 verifier-valid 拒绝控制](../../evidence/java-syntax-2026-09-25/enum-constructor-delegation/negative-controls/analysis.md)进一步证实：把无参构造器的 name 转发改成 `aconst_null` 后，原 class 的 `ZERO.name()` 是 null，而 JADX 重编为 `ZERO`；ordinal 转发改为常量 1 后，原 `ZERO.ordinal()` 为 1，而 JADX 重编为 0；交换 `$values()` 的读取次序还使 JADX 改变构造效果顺序。JADX 对 `$VALUES` 与辅助方法的启发式消隐不能作为 Jarde 的通过判据。

当前 `ClassSourceRecovery` 只把 `<clinit>` 的顶层 AST 步骤作为同轮 sidecar 交给类装配；两个 `<init>` 的 `ClassSourceMethod` 虽保留原报告和发射文本，却没有可直接取用的用户调用/字段赋值 AST。2.1 的构造调用边仅用已收集的 Code 候选即可证明，不需要新 IR；2.2 若要原样保留终端构造器的 helper 与字段写入，必须在现有单次恢复中增加最小的构造器 AST 交接，按原 BCI/Code 与消费核对。不能从 `RecoveryReport.text` 截取语句，亦无须新建通用反编译 pass。

## Goals / Non-Goals

**Goals:** 在基础两常量能力已独立验收的前提下，复用其同次候选并对一个零源参数构造器单次委托到唯一 `int` 构造器、另一个常量直接传 `int` 的 Java 8 子集建立新的整组证明，完整保留源码重载、用户效果、来源和反射构造器形状；把缺证与预算停止按原契约拒绝。

**Non-Goals:** 任意构造器委托图、多个整数/引用参数、变量或效果性委托实参、常量专属子类体、泛型构造器、通过 JADX 文本或目标代码执行推断构造关系。

## Decisions

1. **复用同一次基础枚举候选，不另造识别器。** 基础类级证明已核对字段顺序、name/ordinal、隐式数组、`values`/`valueOf` 与 `<clinit>`；此变更只扩构造器选择与委托边。基础计划当前会拒绝第二个构造器，因此实施时从它同次收集、尚未投影的结构化候选继续证明，再与基础的其余门原子提交。单靠 `ACC_ENUM`、常量名或 JADX 的 values 数组排序既无法区分哪个常量选了哪个重载，也无法保证原反射构造器集合。
2. **精确证明两条物理构造器和调用图。** 只接受本类唯一 `(Ljava/lang/String;I)V` 与唯一 `(Ljava/lang/String;II)V` 两条目标，常量构造点分别与这两条 Methodref 精确绑定。前者必须以入口 `this` 为 receiver，将入口 name/ordinal 不变地传给后者，并额外传入真实整数常量 `0`；其 Code 不得有其它可观察动作、异常表、分支或第二条调用。后者必须先以相同入口 name/ordinal 调用 `java/lang/Enum.<init>(String,int)`，随后仅保留已完整恢复的用户调用与字段保存，按原 BCI 顺序发射。首片的用户效果限于一个外部类静态 `(I)V` helper 和一个本类 `private final int` 实例字段；其它合法正文暂时拒绝，不能据此删除效果。比较物理 Code/SSA 身份及所有消费，而非只比较描述符或 `Signature` 文本；`Signature` 的 `()V`/`(I)V` 与**剥除已证隐式前缀后的**源码参数逐位置比对。任何重复/歧义成员、未读 Code 或不一致参数使整组拒绝。

   首片终端用户效果进一步限定为：一条对枚举类以外、可写成合法 Java 类型路径的 owner 发出的静态调用，目标方法名是合法 Java 标识符，descriptor 精确为 `(I)V`；该调用的唯一实参、实例字段写入 RHS 都由物理 Code 的 slot 3 证明来自终端构造器的同一个源整数参数。实例写入只接受一个精确 `private final int` 字段。实例内 helper、实例/接口调用和其它字段形状留待独立证据，不由此首片覆盖。
3. **投影两个构造器，但保留物理报告。** 在现有 class-source 装配的短生命周期 enum 计划中暂存两个构造器的物理身份、前缀来源，以及从同一次 `recover_for_class_source` 取得的最小构造器 AST 候选；剥除已证的注入参数与 `Enum` 调用后写 `DelegatingEnum() { this(0); }`，终端构造器写一个 `int` 参数并按原顺序保留 helper 调用和字段写入。字段、方法的 JSON 仍按原表顺序保留 item、outcome、真实 source map；类源码是否隐藏隐式成员由基础整组证明控制。不能从 `RecoveryReport.text` 查找 `this(` 或删字符串，也不能另做一遍不计费方法分析。
4. **完整验收包括反射和反例。** 同一冻结 class 的原/JADX/Jarde 全类在 `-g`/`-g:none` 下分别 Java 8 重编、`-Xverify:all` 执行，比较 values、name/ordinal、字段、helper 次数/顺序及 `getDeclaredConstructors()` 的参数数目集合 `{2,3}`。把委托目标、常量 `0`、name/ordinal 身份、用户效果、`$VALUES`/辅助方法和预算/取消分别设为拒绝控制；若补丁不能通过 verifier，必须明确它只测试证明函数，不能冒充可运行对照。all/essential 类正文一致。当前不用新库：reader/SSA/预算和类级装配已提供所需事实；JADX 的 Apache-2.0 许可容许算法对照，但引入其 Java/Dex visitor 不会形成受选定环境约束的 Rust 证明。

## Risks / Trade-offs

- **只保留输出值却丢构造器重载** → 正例同时验收 `this(0)` 文本与反射 `{2,3}`，拒绝把 `ZERO` 统一改成 `ZERO(0)`。
- **投影遗漏委托/终端调用的用户效果** → 仅剥除逐 BCI/SSA 已证的隐式前缀，余下结构化语句按原顺序保留；未证明任一使用或效果则整组拒绝。
- **现有类级证明只认单构造器** → 从已验收的同轮 Code/成员候选扩展调用边和整组投影；双构造器路径在全部证据闭合前不得复用单构造器的 `Proved` 结论。
- **首片过窄** → 委托实参只接无效果常量 `0`，两条构造器均必须唯一；复杂合法变体留待独立证据，不为追求覆盖而弱化证明。
