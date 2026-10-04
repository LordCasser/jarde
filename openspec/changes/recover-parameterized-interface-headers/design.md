## Context

[桥准入实证](../../evidence/java-syntax-2026-10-04/bridge-method-patrol/README.md)：`BR$Impl implements Comparable`（裸）+ 桥隐藏 → javac 报 `未覆盖Comparable中的抽象方法compareTo(Object)`；`implements Comparable<ParamI>` + 桥隐藏 → javac 重建桥、exit=0。判据范围的 2×2 对照见 [header-invariant/README.md](../../evidence/java-syntax-2026-10-04/bridge-method-patrol/header-invariant/README.md)（root 实测四形，确认唯一失败形是"接口边 ∧ 参数 cast 形"）。

### 精确落点（root 已定位：reader 与拼写侧无需扩展，但需三道门放行 + facade 并列证明器）

- **类头投影处**：`src/class_source.rs:6361` 的 `parameterized_superclass`（由 `parsed.superclass.segments` 是否带 `arguments` 判定）与 6367 的 `direct_parent_candidate`（当前**硬编码** `java/lang/String` 单形参父类）。6372 的 `if parsed.type_parameters.is_empty() && !parameterized_superclass { return Ok(None) }` 是当前"无泛型即不投影"的总门。
- **接口事实已就绪**：`crates/jarde-reader/src/signature.rs` 的 `parse_class_signature`（134 行）**已完整解析** `ClassSignature.interfaces: Vec<ClassType>`（结构定义 22 行、填充 139–148 行），且 `class_references()`（27 行起）已把 interfaces 纳入引用收集。
- **擦除证明已就绪**：`ClassSignatureErasureProof`（signature.rs:222）**已含** `interfaces: Vec<Vec<u8>>`（225 行），且 `prove_class_signature_erasure`（263 行）已接受 `interfaces` 参数并逐个校验擦除（详见下节）。

### 修正：接口投影**不能复用**父类闭包（root 2026-10-04 三次取证后确认）

先前本片被描述为"接线"，实测**不成立**，三处阻塞：

1. **注入闭包显式排除接口**：`prove_direct_generic_superclass_parent`（`src/facade.rs:13736`）的准入条件含 `facts.access_flags & (ACC_INTERFACE | ACC_ANNOTATION | 0x4000) != 0 → false`（13765 行），并要求 `super_class == java/lang/Object` 且 `facts.interfaces.is_empty()`（13767–13770）——即它只证"单参数普通父类"，对接口一律返回 `false`。故本片需要一个**并列的接口证明器**（同样走 `resolve_class_source_dependency_read_raw`，但判据改为：`ACC_INTERFACE` 置位、形参个数与 Signature 实参个数相符、擦除与物理 `interfaces` 项一致），不能改父类闭包的判据（那会放宽已验收片的边界）。
2. **总门不看 interfaces**：`class_source.rs:6372` 是 `if parsed.type_parameters.is_empty() && !parameterized_superclass { return Ok(None) }`——**只看 `type_parameters` 与 `parameterized_superclass`**，故 `implements Comparable<Impl>` 且无自有类型参数、父类非参数化时，在总门就返回 `Ok(None)`，根本到不了拼写循环。需增加"参数化接口"这一条进入条件。
3. **`direct_parent_candidate` 分支互斥拒绝接口**：6435–6441 明写 `if !parsed.interfaces.is_empty() || !physical_interfaces.is_empty() → Err("direct parameterized superclass projection does not include interfaces")`；而 6455–6461 的 else 分支要求**全部** `parsed.interfaces` 的 `arguments.is_empty()`。即现有三条路径都把"参数化接口"排除在外，本片须新增第四条路径（无自有类型参数、父类非参数化、但接口参数化），并保持前三条逐字不变。

**拼写侧与 reader 擦除侧均已就绪（root 三次取证确认，本片工作量因此收窄）**：

- 6516–6519 的循环已对 `parsed.interfaces` 逐项调用 `spell_ordinary_signature_type` 并交给 `class_declaration_with_types`（6526）——接口拼写机制已存在。
- reader 的 `prove_class_signature_erasure`（`crates/jarde-reader/src/signature.rs:263`）**已接受 `interfaces: &[Vec<u8>]` 并逐个校验擦除**（315–340：`class_internal_name(interface)` 与物理项比对，不符即 `erasure_mismatch("interface {index}")`），且 `ClassSignatureErasureProof.interfaces: Vec<Vec<u8>>`（225 行）已被填充。**故 reader 侧无需扩展**——取证义务 (b) 关闭。
- 本片缺的只是"让参数化接口能走到既有拼写循环"的**准入放行**（上述三道门）+ **facade 侧一个并列的接口可解析证明器**（下述）。

**第一个取证义务（收敛后，仅剩一项）**：(c) 确认新增第四条路径与 `recover-nested-generic-class-headers`（类自有类型参数形）的互斥边界——本片 Non-Goals 已排除自有类型参数形，须验证新路径不会误纳（6361 的 `parsed.type_parameters.is_empty()` 前置应已保证，须测试钉死）。

**取证义务 (a) 已由 root 关闭**：`spell_ordinary_signature_type_with_member_path`（class_source.rs:4335）**已处理类型实参**——4465 行 `if !segment.arguments.is_empty()` 递归拼每个 `TypeArgument`（`Exact`/`Extends`/`Any`）并以 `<…>` 连接，`TypeArgument::Exact(Class(Impl))` 经同一函数拼为 `BR$Impl`。这正是既有 `direct_parent_candidate` 为 `Parent<String>` 用的同一路径（已验收），故 `Comparable<LImpl;>` 会拼为 `java.lang.Comparable<BR$Impl>`（单 segment 走 `simple_generic_class_name` 得全限定名 + `<…>`）。**拼写侧确认可直接复用，无需扩展。**

**facade 侧接口证明器**：`prove_direct_generic_superclass_parent`（facade.rs:13736）不可复用（13765 显式拒 `ACC_INTERFACE`）。本片需一个并列证明器，同样走 `resolve_class_source_dependency_read_raw` 取接口定义，但判据改为：`ACC_INTERFACE` 置位（而非要求非接口）、形参个数 == Signature 实参个数、擦除与物理 `interfaces` 对应项一致（此项已由 reader 的 `prove_class_signature_erasure` 覆盖，证明器只需确认接口定义可解析且 arity 相符）。**不改父类闭包判据**（那会放宽 `recover-proved-direct-parameterized-superclass` 已验收的边界）。

## Goals / Non-Goals

> **实施协调（root 2026-10-04 决策，派发前生效；行号已于主线 `a7d6a2b7` 态重验）**：~~本片将与姊妹片 `recover-parameterized-superclass-nested-headers` 合并为一片 `recover-parameterized-class-headers` 再派发~~。**root 更正（同日）：该"姊妹片"从未立项**（`openspec/changes/recover-parameterized-superclass-nested-headers/` 不存在，`ls -d` 核实），故"合并为一片再派发"的计划**无对象**。实际状态：父类嵌套名根治（放宽 `class_source.rs:6444` 的 `parent.binary_name.contains(&b'$')` 拒绝）**尚未立项**，与本片是**两个独立的待立项 change**，不是已存在的姊妹片。二者确实改**同一函数** `project_generic_signature` 的相邻分支、建立**同一不变量**（类头携带类型实参 → 桥可隐藏）、解锁**同一** bridge 前置——故 root 的原判断（"应合并为一片再派发以免 rebase 冲突"）在**技术上仍成立**，只是合并的对象须由 root **先立父类嵌套名片**、再与本片合并，而非假定其已存在。派发前 root 须先补立父类嵌套名片（落点 `class_source.rs:6444`，取证见 [bridge-superclass-rawheader-misdispatch](../../evidence/java-syntax-2026-10-04/bridge-superclass-rawheader-misdispatch/README.md)），届时再执行合并。**教训**：spec 引用另一 change 前必须 `ls` 核实存在，"计划要立的片"写成"待立项"而非"姊妹片"。
>
> 原合并理由（技术上仍成立，供补立父类片后参考）：二者改同一函数 `project_generic_signature` 的相邻分支（接口走三道门放行、父类走 `$` 拒绝放宽），串行实施必然 rebase 冲突（本会话已两次遭遇：ncl 使 bridge 的 facade 锚点漂移 170 行）；且二者建立同一不变量、解锁同一 bridge 前置（接口边 for `Impl`、父类边 for `Spec`/`BR$StrBox`）。
>
> **父类嵌套名根治片的落点取证（root 2026-10-04 零构建读码，供补立时直接用，勿再从零取证）**：`$` 拒绝**有两处**，放宽须**同时**改，只改一处不足——(1) `class_source.rs:6444` 的 `parent.binary_name.contains(&b'$')`（`project_generic_signature` 内，`direct_parent_candidate` 分支）；(2) `facade.rs:13761` 的 `parent_name.contains(&b'$')`（`prove_direct_generic_superclass_parent` 内，即 6444 通过 `prove_direct_parent` 闭包实际调用的父类解析器，闭包在 `facade.rs:6429-6438` 构造）。**关键事实（决定该片比"删一个 `$` 检查"复杂）**：`prove_direct_generic_superclass_parent` 除 `$` 外还要求父类定义 `super_class == java/lang/Object`（13769）且 `interfaces.is_empty()`（13770）；root 用 javap 核实 `BR$StrBox extends BR$Box<java.lang.String>`、`BR$Box<T>` 的 `super_class = java/lang/Object` 且 `interfaces = 0`——**故对 `BR$Box` 这个父类，13769/13770 都已满足，`$`（13761）是唯一阻塞**。但放宽 `$` 有**健全性前置**：`$`-拼写名不能直接作为源码文本拼写（须靠 nesting 元数据/InnerClasses 拼成 `Outer.Box<String>`），且直接关联 handoff 的**池形结构反射陷阱**（`$`-拼写字面量被 `getSimpleName`/`getEnclosingClass` 等消费会静默偏离）——故该片**不是**删检查，而是"证明嵌套父名可由 nesting 元数据忠实拼写为 `Outer.Box<实参>`，且拼写后不触发结构反射陷阱"。这是**中大颗粒**，须独立 design 处理拼写与陷阱交互，不能并入 interface-headers 片（后者只碰 interfaces 分支、不碰 `$` 拼写）。取证锚：`Spec`/`BR$StrBox` 形见 [bridge-superclass-rawheader-misdispatch](../../evidence/java-syntax-2026-10-04/bridge-superclass-rawheader-misdispatch/README.md)。
>
> **拼写侧的阻塞点（root 2026-10-04 追加取证，读 `names.rs` 确认）**：既有拼写机制 `nested_reference_spelling`（`crates/jarde-java/src/names.rs:168-212`）**已能把 `$` 名拼成 `.` 形**（`tail.split('$')` → `format!("{head}.{}", nested.join("."))`，209 行），但**自嵌套规则**（204-207 行：`let owner = current.split('$').next(); if head == owner { return Cow::Borrowed(name); }`）会在"被引用名的顶层 owner == 当前类文本所属的顶层 owner"时**保留池形**。对 `BR$StrBox` 的头，`current = BR$StrBox` → `owner = BR`，而父名 `BR$Box` 的 `head = BR` → **相等 → 保留 `BR$Box`**。即**该规则正是裸头的直接成因**，而它的存在理由是（函数文档 201-203 行）"the name's top-level owner is the class whose text this is, so the member's declaration belongs to this unit's own family and only a fold projection may spell it as source nesting"——**只有 fold 投影可以把它拼成源码嵌套形**。
>
> **故该片的真实设计问题是**：类头投影是否算"fold 投影"？若算，须让 `nested_reference_spelling` 在类头上下文接受自嵌套形（或走 `nested_member_reference_spelling`，225 行，它额外要求 `InnerClasses` 行成员资格——**这是它比 `nested_reference_spelling` 多出的唯一证据**，正是 218-224 行文档所述"`$` 单独不足以证明嵌套关系，因为 `$` 是顶层类名的合法字符"）；若不算，须另立拼写通道。**且无论走哪条，都必须与结构反射陷阱守卫交互**：拼成 `BR.Box<String>` 后，该文本不再含 `$`，`getSimpleName`/`getEnclosingClass` 等 11 个方法（`facts.rs` 的 `STRUCTURAL_REFLECTION_METHODS`）的答案会**与池形时不同**——须确认重拼后仍不产生静默偏离（对照 handoff「池形类型名的结构反射陷阱」与 ncl 片的 `rerun_pool_spelled_structural_reads` 做法）。**root 未做这一步取证**，故该片**尚不可派发**，须先由取证回答"类头投影是否属 fold 投影"与"重拼后结构反射答案如何变化"两问。
>
> **行号漂移已处置（root 2026-10-04）**：原决策写"合并推迟到临近派发时做，以免精确行号在队列等待期再次漂移失效"——漂移**确已发生**：`1cb359d6`（ncl 的结构反射重跑修复）触碰了 `class_source.rs`，使 `project_generic_signature` 6330→6334、`parameterized_superclass` 6349→**6361**、`direct_parent_candidate` 6354→**6367**、总门 6360→**6372**、接口互斥判据 6422–6428→**6435–6441**、else 分支 6437–6451→**6455–6461**、接口拼写循环 6504–6511→**6516–6519**、`class_declaration_with_types` 6513→**6526**；`facade.rs` 侧 `prove_direct_generic_superclass_parent` 13013→**13736**、`ACC_INTERFACE` 判据 13042→**13765**、`super_class`/`interfaces` 判据 13045–13047→**13767–13770**（drift 恒为 +723）。**以上九处 class_source + 三处 facade 引用已全部按现状更正并逐一核实**（`crates/jarde-reader/src/signature.rs:263` 的 `prove_class_signature_erasure` 未漂移，仍准确）。故"推迟合并以避免漂移"的理由已不成立——**合并须等 root 先补立父类嵌套名片之后才可执行**（见上"root 更正"：该姊妹片尚不存在），且合并后须按同一方法（锚点名重验，勿照抄行号）再核一遍。
>
> **优先级更正**：原写"低于在飞的 `recover-bridge-superclass-header-precondition`"——该片**已落地并 root 验收**（合并 `cc4b6f11`，验收记录 `9ca2db6a`；注意勿与 `recover-bridge-admission-gates` 的合并 `5f07e13c` 混淆，那是另一片）。本片仍属"呈现改善"而非正确性修复：bridge 前置修复落地后，裸头形已"可编译且行为正确"（桥保持可见、派发正确），本片只是让它进一步"隐藏桥 + 参数化头"更优。故优先级低于正在处理的**正确性/真 Java 8 覆盖**类缺口（DT-03 分配限定符残留、TWR javac 8 codegen、EM-15 写访问器），高于纯呈现润色。

**Goals:** 类头 `implements` 子句按 Signature 投影类型实参；不可解析时保留裸类型并拒绝该类桥投影。**Non-Goals:** 嵌套/多层参数化接口（`Map<K,V>.Entry` 形按既有裸回退）；类自身有类型参数的形（`class C<T> implements I<T>`——属 nested-headers 域）；接口方法声明的泛型签名（成员域，已由 9/9 片覆盖）；不改父类投影既有行为（含 6367 的窄边界）；不放宽 `recover-bridge-admission-gates` 的消隐前置（本片只**供给**其依赖的类头事实）。

## Decisions

1. **复用既有类头投影通道**：与 `recover-proved-direct-parameterized-superclass` 同一选定环境、同一"擦除 == 物理 header"核对、同一原子性（整个类头一次决定，不从调用或局部猜类型）。接口列表逐个投影：任一接口不满足判据则该接口保持裸类型（不整体回退已可证的其它接口，除非既有通道的原子性要求整体一致——**按既有实现的原子性口径执行，并在报告中说明选择依据**）。实现上是在 `class_source.rs:6361/6372` 的既有门处并列加入 `parsed.interfaces` 的判定，而非新建一条类头装配路径。
2. **消隐前置不变量（跨切面共享契约，已由 bridge 片实现）**：擦除桥的可重建性来自类头类型实参。`recover-bridge-admission-gates`（**已验收合入** `5f07e13c`）已实现该前置：参数 cast 形桥若其契约属主为接口边、且类自身 `Signature` 把该接口拼为带类型实参（=泛型接口）而投影头未携带，则**拒绝桥投影、保持桥可见**（`bridge_interface_contract_generic`，验收记录见其 tasks 3.3）。**本片不改该前置**，只提供它依赖的类头投影——即让 `BR$Impl` 的类头呈现 `implements java.lang.Comparable<BR$Impl>`，从而使 bridge 片的拒绝分支转为准入分支（owner 分离：类头文本 owner 是本片，消隐决策 owner 是桥准入片）。
3. **验收锚定**：`BR$Impl`（`implements Comparable<BR$Impl>`）类头投影 → 桥投影随之启用 → 整类 `javac --release 8` 通过、`-Xverify:all` 运行与原 class 一致（`0`，含经接口引用的 `compareTo` 调用）；多接口形（`implements A<X>, B`）部分可证时按决策 1 的原子性口径呈现；负例（接口不可解析、arity 不符、擦除不一致）保留裸类型且桥**保持可见**（bridge 片前置的现行为，不得回退）。

## Risks / Trade-offs

- **与 bridge 片的接缝回归**：bridge 片已合入并以 `BR$Impl` 桥**可见**为验收态（前置拒绝）。本片使类头带类型实参后，同一 fixture 的桥将转为**隐藏**——这是预期变化，但须确认 bridge 片的 `br_family_negative_shapes_keep_their_refusals` 等负例测试不被本片意外翻转（若某负例断言"桥可见"依赖于头是裸类型，本片须同步更新该断言并说明，不得静默放宽）。实施顺序上本片在 bridge 之后，接缝由本片负责验证。
- **裸类型保留的信息损失**：接口不可解析时类头仍是裸类型（现行为），且桥**必须**可见（bridge 片前置已保证），故不会出现"契约丢失 + 桥隐藏"的双重损失。
- **corpus 面变化**：类头文本变化会波及所有参数化接口实现类的输出——双腿扫描，差异应仅类头与桥家族；既有父类投影与成员参数化测试零回退。
