# Design：捕获形伴生 ctor 的 super 重排

## Context（root 已实测）

- 字节码：`aload_0; aload_1; putfield val$s; aload_0; invokespecial ArrayList.<init>` —— val$ 赋值先于 super（javac8 捕获惯例）。
- 渲染现状逐字跟随字节码次序 → 源级非法（super 前 this）。
- 无捕获形健康；jadx 以分配点双括号形彻底解决（本片取最小重排路径）。

## 决策 1：重排条件 = pre-super 全捕获赋值 + super 实参无依赖

可证安全当且仅当：
1. super() 调用之前的语句**全部**形如 `this.val$x = argN`（捕获字段赋值，无其它副作用）；
2. super() 的实参（若有）**不读取**任何被赋值的 val$x（数据流判定——实参只依赖参数/常量/this 外的值时成立）。

满足则重排：`super(); this.val$x = argN; …`。任一不满足 → 保持现状（渲染头已声明 not claimed to compile，响亮）。

## 决策 2：呈现次序与源形

重排后 val$ 赋值组保持原相对次序，置于 super() 之后、实例块语句之前（与 javac 的源级匿名类语义一致：捕获字段先初始化再跑实例块）。

## 决策 3：零回退与负例

- 无捕获形（`DB$1`）伴生逐字节不变；
- 宿主呈现（`new DB$2(arg0)` 调用形）零改动；
- 负例：super 实参依赖捕获值的合成形（如匿名类 `extends Base` 且 `super(s)` 用捕获 s——手工字节码或混淆产物）保持现状；
- corpus 双腿扫描：差异类仅为捕获形伴生（如实记录数量）。

## 验证标准（可证伪）

1. 主锚：`DB` 双形拼接 `javac --release 8` exit 0（修复前 exit 1）、行为 `2/z` 逐行一致；
2. 零回退/负例如上；corpus 差异边界记录；
3. 门禁全量（基线以合并态为准）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + fingerprint 再生。

## Open Questions

1. 伴生 ctor 渲染的落点（member_inner vs facade——task 1.1 定位 val$/super 次序逻辑所在）；
2. 真实语料捕获形伴生数量（corpus 扫描给出——决定该缺陷的暴露面）。

## 实现补记（2026-10-07，实施者）

**1.1 定位结果**：次序逻辑不在 `member_inner.rs`/`facade.rs`，而在 `crates/jarde-java/src/ctor_order.rs::present_prologue_first`（`build.rs::build` 尾部唯一调用点，`build.rs:8082`）。`member_inner.rs`/`facade.rs` 拥有的是**分配点匿名投影**（`DB$2` 因父类 `java.util.ArrayList` 不可拼写而拒为 `anonymous_super_source_type_unproved`）与**伴生实例化文本**（`new DB$2(arg0)`）；二者都不写伴生 ctor 的语句序。证据：[results/01-locate-ordering-logic.md](results/01-locate-ordering-logic.md)。

**决策 1 的安全前置（实施中必要的收窄，已写入 spec）**：决策 1 原文只列（1）前缀全捕获赋值、（2）super 实参不读捕获值。实施时必须再加**（3）该调用无法观察到这些字段**，否则 `recover-ctor-reorder-dispatch-guard` 的既有判据（及其冻结反例 `anonymous-super-dispatch`）会被削弱：该 fixture 的 `super()` 无实参、前缀也全是捕获赋值，（1）（2）都成立，但 `Base` 构造期的虚分派会读到被移动的字段。故（3）保留原 `java/lang/Object.<init>()V` 档逐字不变，并新增一档**只依赖单类事实**的证明：

- 该类的**方法表**（`Inputs::class_methods`，与 `is_synthetic_field` 读字段头同一份声明视图）除构造器与类初始化器外不声明任何成员 → 调用期能读到合成捕获字段的代码（只能由本类声明——没有父类是针对它编译的）不存在；
- 该档同时要求（2），因为实参在调用前求值、赋值组移到调用后；实参的判定是**数据流闭包**（SSA 定义链），不是表达式拼写；闭包内出现调用即拒绝（被调方法体不在本运行事实内）。

**为何不用"本类方法是否读被移动字段"作判据**：该事实需要类内其它方法的方法体，`class_source` 路径的 `Inputs` 不持有（`members` 只为按需 callee 读取填充）；引入类内跨方法读是新机制，超出本片（且 `recover-ctor-reorder-dispatch-guard` 决策 1 已按同一理由拒绝跨类读体）。

**已知保守代价（实测）**：实参含调用（`super(compute())`，该类只声明构造器）保持字节序呈现——被调体不可读，不构成证明（`CallArg$1` 负例，`tests/double_brace_capture.rs`）。javac 自身不会产生该形（它把调用提升到调用者、把结果作为参数传入），故真实语料代价为零。

**"super 实参依赖捕获字段"负例的实测性质**：该形（`getfield` 于 `uninitializedThis`）**不可验证**——JVMS 4.10.1.9 只允许 `uninitializedThis` 用于本类字段的 `putfield` 与初始化它的 `invokespecial`，JVMS 8/23 实测均 `VerifyError`，本运行的帧阶段亦拒（`ir_frame_deferred`）。故它今天的状态是"整方法拒绝"（不是重排），本片以 `ReadArg$1` 冻结该事实并标注"恢复日须重判"。这是决策 3"手工字节码或混淆产物"负例的可实现形式；同时说明"被移动字段在调用期被读"在可分析输入中不可达，参数走的调用臂是唯一可触发的臂（`CallArg$1`）。
