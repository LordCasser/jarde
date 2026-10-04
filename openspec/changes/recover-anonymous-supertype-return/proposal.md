## Why

`present-proved-java-structure` 5.3（匿名类内联）链的**最后一环未立项**。环 0/1/3 已落地（混合参数形、局部声明初始化位、根方法带一个捕获参数），四处 ctor-reorder fixture 中两处已闭合；剩余的 `anonymous-top-level` 撞 `anonymous_super_return_type_unproved`——**根方法返回父类的超类型**（返回接口 `Renderer`，而父类是 `Base`），该门要求根方法返回描述符**恰为** `()Lparent;`。

真实代码里"工厂方法返回接口类型、内部 new 匿名实现类"极其常见（`List<String> f() { return new ArrayList<>(){…}; }`、`Runnable r() { return new Runnable(){…}; }`），故这是**高频形**，不是窄 fixture。

**root 已实测钉死的事实**（见 [anonymous-chain-rings-2-3](../../evidence/java-syntax-2026-10-04/anonymous-chain-rings-2-3/README.md)，含 2026-10-04 第三次更正）：

1. **落点**：`src/facade.rs` 约 4749–4756。环 3 已把该门条件化为 `if site_shape == …DirectReturn { … }`，内含 `plain_return`（`()Lparent;`）与 `capture_return`（`(P)Lparent;`）两条恰等判据，**两者的返回段都硬编码为 `parent_name`**。该处注释已明写"a supertype return is ring 2's separate, unproven slice and keeps this refusal"。
2. **关键有利事实**：`parent_read`（父类 class file 的 `ConfirmedRead`）在**该门之后紧接的 4758 行**就由 `resolve_class_source_dependency_read_raw` 解析出来了。故环 2 的一层判据**不需要任何新的解析机制**——只需把门的检查下移到 `parent_read` 可用之后（或在该处补取），即可直接读 `parent_read.facts.super_class` 与 `parent_read.facts.interfaces`。
3. **两处未闭合 fixture 的阻塞门不同**（root 以 javap 核实 `super_class` 常量与根方法返回描述符）：
   - `anonymous-top-level`：父类 `Base` **顶层可拼写**，返回 `Renderer` 是 `Base` **直接实现**的接口 → **只需本环**。
   - `anonymous-capture`：父类 `AnonymousCaptureCases$Base` 与返回类型 `AnonymousCaptureCases$Renderer` **均含 `$`**（都是嵌套类）→ **先撞 `anonymous_super_source_type_unproved`**（父类须为"同包、可直接拼写的源码类型"），本环**无法单独闭合它**，还需嵌套父类名可拼写能力。

   **故本环的验收锚只能是 `anonymous-top-level`**；把 `anonymous-capture` 当锚会让实现者误判完成而实际仍拒绝。
4. **无现成的类级可赋值性证明件**（root 已逐个核实四个候选，详见上述证据的设施盘点表）：`prove_no_body_generic_hierarchy` 是**保守拒绝**而非层级 walk；`prove_snapshot_hierarchy_widenings` 证 IR 内的**值**放宽不证**类**关系；`members.rs::subtype_of` 概念匹配但为私有 `fn` 且绑定 `HeaderClosure`/`Search`/`ClassSite` 访问检查机制；`jarde-query` 无公开超类型 API（`scan_hierarchy` 只做单层 xref 元数据）。**但按事实 2，本环 MVP 不需要传递闭包**——两 fixture 的关系都是一层直接关系。

## What Changes

- 放宽 `anonymous_super_return_type_unproved` 的返回段判据：**从"返回段恰为 `parent_name`"放宽为"返回段是父类的一个已证超类型"**，MVP 只做**一层直接关系**——声明返回类型 `T` 满足 `T == parent_name` ∨ `T == parent_read.facts.super_class` ∨ `T ∈ parent_read.facts.interfaces`。传递闭包（`T` 是祖父类/间接接口）**不在本环**，保持拒绝并登记。
- 返回类型的**源码拼写**：`T` 必须可在源码中拼写（不含 `$`、每段合法 Java 标识符、与根类同包），复用既有 `anonymous_super_source_type_unproved` 判据，**不得另写一套**。注意 `T` 与 `parent_name` 是两个不同的名字，二者都要过可拼写检查。
- 发射时返回位置/声明位置按 `T` 拼写，分配点仍投影为 `new Base(…) { … }`（`Base` 是父类源码名，不是 `T`）——即 `Renderer create() { return new Base(…) { … }; }`，这与 Java 源码原形一致。
- **不放宽** `LocalDeclInitializer` 分支（环 1 的站点形无返回描述符检查，本环与之无关）。

## Capabilities

### Modified Capabilities

- `java8-recovery`：匿名类内联在"根方法返回父类的直接超类型（父类自身、其父类、或其直接实现的接口）"时同样成立，返回位置按该超类型拼写；传递闭包关系保持拒绝。

## Impact

`src/facade.rs`（返回段判据与拼写来源；可能需把门检查移到 `parent_read` 解析之后）。`crates/jarde-java` 预期无改动（`ClassMemberFacts.interfaces`/`super_class` 已可读）。**不改** `emit.rs`（环 1 已提供声明位重拼通道）、**不改**站点扫描。

**实现修订（施工取证追加，详见 design 的补充决策与 tasks 2.5/2.6）**：锚在返回门之后还撞共享 owner 普查 `prove_anonymous_owner_xrefs`（child 体 `invokevirtual 自身.seed` 的符号 owner 引用）——本环同时为该普查新增"父类路径 ∧ DirectReturn 站点形"的匿名体自调用允许臂，以具名判别类型显式遏制（接口路径与 LocalDeclInitializer 形保持既有拒绝），`InvokeSpecial` 分支不开放；判据、健全性与归因见 design/tasks 与证据目录 `root-replies-verbatim.md`。

冻结锚：`tests/fixtures/proved-java-structure/anonymous-top-level/`（`static Renderer create()`，父类 `Base` 顶层、`Base implements Renderer`）。其当前状态须实测记录（root 已测：撞 `anonymous_super_return_type_unproved`，渲染为物理文本 `return new AnonymousTopLevel$1(choose());`，渲染源集 `javac` 应 exit 1）。

**顺序约束**：本环改的门与环 3 改的是**同一处**（环 3 刚把它条件化为 `plain_return`/`capture_return` 两条恰等判据）。环 3 已合入主线（`d906464e`），故顺序满足；但实现前须**重验行号锚点**（本会话 facade.rs 已两次因相邻片漂移 100+ 行）。

**不得顺带做的事**：不放宽父类名含 `$` 的 `anonymous_super_source_type_unproved`（那是 `anonymous-capture` 的阻塞，属嵌套名可拼写域，须另立项）；不做传递闭包；不放宽参数表（环 3 已交付）；不改站点形（环 1 已交付）。
