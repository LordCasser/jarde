## Context

见 [proposal](proposal.md)、[行为规格](specs/java8-recovery/spec.md)和 [recover-proved-enum-constants](../recover-proved-enum-constants/design.md)。后者的 Stage 单构造器/两常量投影已在任务 2.2 验收，带用户静态后缀的 Measure 也已在 2.3 验收；首片只准入“一个源级 `int` 参数、直接构造主 enum”；本变更的 `Op` 两常量没有源参数，`new` 的 owner 是各自匿名子类，**不能直接消费该首片的成功计划**。本变更须复用前置实现的同次常量候选、隐式数组和辅助方法核验，并将证明扩至零源参数/匿名 owner；主类与子类关系合证后才产出成功计划。当前 `Engine::class_source` 为主类每个成员保留物理记录并恢复正文，`ClassSourceField`/`ClassSourceMethod` 已有源级文本与原始报告，但没有“一个 enum 常量引用的匿名子类及其方法体”的类级关系。现有 member-inner 候选侧车只为具名成员内类的调用与类型拼写服务，不能拿来替代这一关系。

本地 JADX 1.5.6 的 `EnumVisitor.convertToEnum` 从 `<clinit>` 的线性前缀及 `$VALUES` 构造提取常量；`processConstructorInsn` 找到不同于主枚举的构造 owner 后把 ClassNode 交给 `processEnumCls`，`ClassGen.addEnumFields` 将其方法体发在常量后。`ProcessAnonymous.canBeAnonymous` 允许仅在 enum `<clinit>` 使用的类。可借鉴的是构造点到子类再到类体的方向，以及将类体附在常量语法位置；不能复制按 synthetic/名称猜测、优先 `$VALUES` 字段和隐去子类构造器的宽松决策。JADX 本地许可为 Apache-2.0，许可本身不妨碍参考，但其 Java/Dex visitor 与本项目 Rust reader、选定环境和预算模型不相接；直接引入依赖不会减少核心证明工作。

实施接缝须先区分**采集**与**成功证明**：当前 `may_capture_group_code` 会在 `ACC_ABSTRACT` 处排除 `Op`，而 `prove_group` 还把每个主类方法有 Code、构造 owner 为主枚举及一/两条已知构造器作为成功门。首步仅放宽 Java 8 两常量候选的便宜采集门，沿既有 prepared member 单次运行保留 `<clinit>` 的精确 `new`/`invokespecial` owner、BCI 和全方法 member-use；无 Code 的抽象声明保留物理头事实，不伪造空 Code。之后在同一选定环境和 Budget 下按构造点读取唯一子类，核对 typed `InnerClasses`/`EnclosingMethod` 与主类声明；再把主类抽象声明和 synthetic 访问构造器作为待证的具体形状。以上任何中间状态都不能产生 `Proved` 或触发类源码投影；2.2/2.3 的独占使用、委托语义和子类正文闭合后才原子发布。这样复用现有候选/reader/类装配接缝，无须新增通用 visitor 或按 `$1` 名称枚举子类。

2.1b 的子类关联保存在 class-source 报告的私有 `serde(skip)` 侧车中，仅供同次后续证明消费；只有当前两常量组为 `Refused` 且 `<clinit>` 同次扫描含已验证的非主类构造 owner 时才读取目标类。关联记录保留常量字段索引、分配与构造 BCI、构造描述符和选定物理定义，本身不改变 `Refused`，没有类源码投影权；普通已证明枚举不会因此多读依赖。

冻结的 `javap` 还揭示两个不能略过的主类差异：`Op` 带 `ACC_ABSTRACT`，其 `apply(II)I` 无 Code；`Op`/`Mixed` 的主类除了私有 `(String,int)` 构造器，还有 javac 生成的 `(String,int,Op$1)` / `(String,int,Mixed$1)` synthetic 访问桥。匿名子类构造器传 `null` 给这条桥，桥再原样转发 name/ordinal 至主类私有构造器。现有基础证明既拒绝抽象 enum，也要求每个方法都有 Code、恰好一条构造器；因此正例必须证明完整构造链、抽象声明和每个常量体的实现，不能仅让常量调用点换 owner 后绕过这些门。[等宽 ordinal 转发反例](../../evidence/java-syntax-2026-09-25/enum-constant-body-bridge-controls/analysis.md)已证明该访问桥一处变化即可令 JADX 重编结果从原 class 的 `ADD:1` 变为 `ADD:0`，虽两者均通过 verifier。

## Goals / Non-Goals

**Goals:** 在两个已证明常量的首片中支持各自独占匿名子类覆写简单方法，及一个普通常量与一个带体常量的混合形状；原 class、JADX 和 Jarde 的 Java 8 重编/运行结果对照，证明边界失败时不删物理事实。

**Non-Goals:** 修改普通枚举基础 change 的验收范围、泛化任意常量数量/源参数形状、任意匿名类内联、带捕获或额外字段/初始化的子类、开放世界中所有外部使用点证明、JADX/Kotlin 枚举方言、修改单方法恢复语义。

## Decisions

1. **复用基础候选与证明器，不另造枚举识别器。** 基础 change 必须先交付直接构造主类、带一个源级整数参数的完整计划；但该计划对 `Op` 的零源参数和匿名 owner 会拒绝，不能把拒绝当成功前提。本变更在同一次类装配中复用原常量字段、构造 BCI/owner、name/ordinal、隐式数组和辅助方法候选，窄扩零源参数与构造 owner 不同于主枚举的证明分支，随后把子类关系/正文、主类 synthetic 访问桥、抽象声明和主类初始化**原子合证**；没有完整合证就没有可投影常量列表。owner 等于主枚举时保持普通常量，owner 不同时才尝试子类。不得从 `Op$1`、`ACC_ENUM` 或子类表顺序猜测常量归属，不重扫 `$VALUES` 得出第二套结论。
2. **选定物理子类和独占使用先于方法恢复。** 借现有解析环境、`ConfirmedRead` 与 reader typed `InnerClasses`/`EnclosingMethod` 读取子类，核对自身 `this_class`、直接父类为主枚举、匿名内类关系、唯一构造器及精确构造描述符。现有 `resolve_class_source_dependency_read_raw` 已提供选定定义与同次物理字节，`read_class_source_assembly_context` 已提供 typed 嵌套属性；不另建按名字搜索的子类索引。冻结 javac 子类的匿名 `InnerClasses` 行是 `inner_name=None`、`outer_class_index=0`，`EnclosingMethod` 指向主枚举类且 `method_index=0`；这些零值是合法的类级匿名关系，不能当作缺失，也不能凭它们单独认定常量归属。子类 classfile 可以带 `ACC_ENUM`，不能把它当作独立源码 enum。复用当前已选物理输入范围的 XRef 和主类所有 Code 扫描，确认这份子类只在对应常量构造点被分配，没有第二个常量、普通方法、字段或构造器句柄的身份使用；扫描/解析不完整就是未知。这里证明的是所选输入范围，不假装排除范围外动态加载。若现有 XRef 尚不能对某种引用作完整否定，首片保守拒绝，不新建全局索引。
3. **子类构造和正文必须全量可呈现。** 首片只接无实例字段、无类初始化、无捕获、单一编译器隐式构造器及一个或若干可拼写、完整恢复的实例覆写方法。`Op` 子类构造器只能把隐式 name/ordinal 经唯一已证的 synthetic 访问桥转交主枚举私有构造器；传给桥的 `null` 哨兵不得被桥读取或产生效果，桥必须原样转发 name/ordinal，不能暗带源参数或额外效果。`Mixed` 同样处理这条桥，同时证明普通常量直达主构造器；`Plain` 不要求不存在的桥。主类若为抽象 enum，只准接受无 Code 的源级抽象声明，且每个常量都必须选定完整实现该方法的专属子类；不能把无 Code 当作扫描缺失。方法声明的目标在主枚举可绑定，不接受不能解释的 synthetic/bridge/辅助成员。每个有 Code 的目标方法通过既有恢复入口按选定身份运行一次，要求 `structured`、无 fallback/来源缺口，且源码声明/参数绑定可重编。以调用图、方法名或 JADX 的可见性标志代替正文证明均不允许。依赖读取、方法恢复和发射沿用同一 `Budget`/取消；停止原样传播。
4. **类装配做原子语法投影，不回写物理方法。** 为已证常量携带短生命周期的“主类字段/构造 BCI → 子类物理身份 → 完整成员”私有投影侧车，复用基础枚举投影的常量列表发射，在该项后写 `{ ... }`。方法体使用同次结构化恢复和现有 Java 声明/正文发射器；不能从已发射主类或子类源码中搜索替换字符串。仅在该项的所有方法均可发射后提交，失败不留下半个类体。主枚举的字段/方法报告继续保留原物理 item/outcome；子类经单独类/方法请求仍能取得原报告。父类源码中的投影标记只说明消耗了哪个选定子类，不伪造主枚举成员 source map，也不删除物理子类定义。普通无体 enum 无须新增读取。
5. **源码合法性由 Java 8 重编与行为约束。** `getClass()` 的确切匿名二进制名由 javac 决定，验收比较 `getClass() == Op.class`、`name()`、`ordinal()`、`values()`、覆写结果和异常/初始化次数，而不要求原 `$1` 名字重现。受支持样例 `-g`/`-g:none` 三方对照；额外使用、缺 `InnerClasses`/`EnclosingMethod`、父类不符、缺选定子类、额外成员/副作用和预算停止分别验证拒绝。原始物理 class 的 verifier 通过仅说明字节码可运行，不说明 Java 常量体可合法写出。

## Risks / Trade-offs

- **额外使用点漏扫会改变子类身份语义** → 仅在选定范围的 Code/XRef 完整且所有引用都归属该常量时投影；开放世界边界写在报告，不把未读类当作零使用。
- **子类类头/方法可写但主枚举投影未就绪** → 先独立验收基础 change 的直接主类/整数参数分支，再在本变更明确扩出零源参数/匿名 owner 分支；两分支共享候选和隐式成员证明，不能把基础分支的成功误记为 `Op` 的成功，也不能偷偷扩大基础 change。
- **保守首片拒绝带字段或辅助成员的合法专属体** → 先取得两常量独占覆写的正确闭环；以后在同一侧车逐项证明，不放宽未解释效果。
- **JADX 隐去物理子类使其额外引用断裂** → 只借用它的语法位置和构造关系方向，不删除物理报告，也不复制其 `$VALUES`/synthetic 启发式。
