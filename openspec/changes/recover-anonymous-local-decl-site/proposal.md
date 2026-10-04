## Why

`recover-anonymous-mixed-super-capture` 交付时把锚定在**直返位置**的混合参数匿名类内联为源级 `new Base(args…) { … }`（范围收窄由实现者提出、root 2026-10-04 独立核实技术事实后追认；此前记录为"root 裁决（选项 B）"属归因错误——root 从未下达该裁决，详见 mixed-super-capture tasks.md 2.3 的归因更正段）。实现取证发现冻结 fixture `anonymous-super-args/AnonymousSuperArgs$1` 的分配点在 `main` 内的**局部声明初始化位置**（`AnonymousSuperArgs$1 local2 = new …`），它还要越过**三道**本片未钉死的前置：

1. 站点扫描只认整方法直返形（`class_source_direct_return_new`）；`recover-anonymous-mixed-super-capture` 已把它放宽为"局部声明前奏 + **末条**直返"，但锚的分配点在**方法体中部**（其后还有两条语句），赋值初始化形也不产站点。
2. `project_class_source_anonymous_super` 要求 `root_method.descriptor == "()L{parent};"`；**锚的根方法 `main` 返回 `void`，该表述对其根本不适用**——父类源码名的信息来源必须改为"声明初始化值 `new` 操作数的 owner 类型"。
3. 即使发射，赋值左端拼出不可命名的匿名类型名（`AnonymousSuperArgs$1 local2 = new Base(…) { … }` 无法编译）——需要把**声明类型重拼为父类源码名**，这是 emitter 尚不具备的新能力。fixture 以 `-g:none` 冻结（**无 `LocalVariableTable`**），故左端类型只能来自 `new` owner，重拼是**载荷性**的而非装饰。

另有一道**既有**约束本片必须继续遵守（不属新增前置，故不计入上面三道）：父类 binary 名不含 `$`、每段是合法 Java 标识符、与根类同包（`anonymous_super_source_type_unproved`）——这正是 `anonymous-capture`/`anonymous-top-level` 仍被拒的原因，本片不得顺带放宽。

真实代码中局部声明初始化远多于直返（`present-proved-java-structure` 2.10 所指的终局解最终要覆盖它），故立此片，不静默丢弃。**四处关键判据（根方法门表述、站点扫描边界、左端重拼的健全性依据与三项必要检查、前置数目）已由 root 实测钉死于 [design.md](design.md)，实现者不得自行选择。**

## What Changes

- 站点扫描接受"局部声明初始化形"的已证分配点（分配表达式为唯一初始化值的局部声明，且该声明的其余形态完整）。
- `project_class_source_anonymous_super` 的根方法门放宽为接受该形态（返回门改为以声明/赋值目标类型可赋父类为准）。
- emitter 的 `AnonymousOverride` 增加赋值左端声明类型重拼能力：被投影分配点所在声明把匿名子类类型名改写为父类源码名；重拼不可证明时保持物理文本。
- 复用本片的捕获证明、参数角色划分与词法替换通道，不新增第二套机制。

## Capabilities

### Modified Capabilities

- `java8-recovery`：混合参数匿名类在局部声明初始化位置同样内联为源级 `new Base(args…) { … }`，其赋值左端类型名随投影重拼。

## Impact

`crates/jarde-java/src/report.rs`（站点扫描与 emitter）、`src/facade.rs`（根方法门与发射参数）。冻结锚是 `tests/fixtures/proved-java-structure/anonymous-super-args/`（其完整源集当前退出 1，本片后应退出 0 且事件日志逐行一致）。验收须包含该 fixture 的三方行为对照与 `recover-anonymous-mixed-super-capture` 的全部既有正负例零回退。
