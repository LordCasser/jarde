## ADDED Requirements

### Requirement: 捕获形伴生构造器的 super 次序 SHALL 可编译

当匿名/内部类伴生的构造器在字节码中把捕获字段赋值（`this.val$x = argN`）发射在 `super()` 调用**之前**、且满足（1）super 前语句**全部**是捕获字段赋值、（2）super() 实参不读取任何被赋值的 val$x（按实参值的 SSA 依赖闭包判定——实参只依赖参数/常量/this 外的值时成立）、（3）该调用无法观察到这些字段时，系统 SHALL 把渲染次序重排为 `super(); this.val$x = argN; …实例块语句…`——使渲染源在标准 Java（含 8）下合法。

（3）有两档证明，都取自本运行自身的事实：`java/lang/Object.<init>()V`（owner/name/descriptor 三重匹配——final 且空的构造器，构造期不可能运行任何用户代码）；或该类的**方法表除构造器与类初始化器外不声明任何成员**（合成捕获字段只有本类自己的代码能读到——没有父类是针对它编译的——故调用期能读到它的代码只能是本类自己的，经父类构造器的虚分派抵达；本类不声明这样的代码即不可观察。另一构造器不是正在运行的那个，类初始化器在任何实例存在之前已运行，JLS 12.4.2）。第二档同时要求（2）成立，因为实参在调用**之前**求值而赋值组移到调用**之后**。

任一条件不满足（super 前有非捕获语句、super 实参读取捕获字段或运行本运行不持有其方法体的代码、或调用可观察到捕获字段）时 SHALL 保持既有字节码次序呈现（渲染头声明 not claimed to compile——响亮）。无捕获形伴生的既有呈现 SHALL 逐字不变。

#### Scenario: 捕获形双括号恢复

- **WHEN** `new ArrayList<String>() {{ add(s); }}`（捕获 `s`）的伴生 `DB$2` 与宿主 `DB` 渲染拼接后经 `javac --release 8` 编译
- **THEN** 编译 exit 0（修复前因 super 前赋值 exit 1）；`main` 输出与原 class 逐行一致（`2/z`）

#### Scenario: 无捕获形零回退

- **WHEN** `new ArrayList<String>() {{ add("a"); }}`（无捕获）的伴生经呈现
- **THEN** 既有呈现（super 后内联）逐字不变

#### Scenario: 本类代码可在调用期读取捕获时不重排

- **WHEN** 伴生类的 super 目标为非 `java/lang/Object` 的类，且该类在构造器之外声明了成员（`AnonymousSuperDispatch$1` 的 `observe()`、`AnonymousSuperArgs$1` / `AnonymousCaptureCases$1` / `AnonymousTopLevel$1` 的 `render()`）
- **THEN** 呈现保持捕获写入早于 `super()` 的字节序（不可编译是响亮失败），重编要么被 javac 拒绝、要么与原 class 行为逐行一致——不得出现"可编译且行为不同"

#### Scenario: 依赖形保持现状

- **WHEN** super() 实参读取捕获值（合成负例：`ReadArg$1` 的 `getfield` 实参；该类只声明构造器）
- **THEN** 保持字节码次序呈现或整方法拒绝（不重排）——重排需数据流证明，非语法猜测；该形实测不可验证（JVMS 4.10.1.9 禁止 `uninitializedThis` 上的 `getfield`），本运行的分析在其帧阶段即拒绝

#### Scenario: 实参运行本运行不持有的代码时保持现状

- **WHEN** super() 实参是调用（合成负例：`CallArg$1` 的 `invokestatic` 实参；该类只声明构造器）
- **THEN** 保持字节码次序呈现（不重排）——被调方法体不在本运行的事实内，故不构成证明
