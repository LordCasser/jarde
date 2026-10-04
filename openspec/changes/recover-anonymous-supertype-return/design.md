# Design — `recover-anonymous-supertype-return`

见 [proposal.md](proposal.md)。本文件把 root 实测钉死的事实与不变量写清，使实现者不必在关键位置猜测。**本环是 5.3 匿名内联链的最后一环**（环 0/1/3 已落地：`e1c89d57` / `25f2589e` / `d906464e`）。

## Context（root 实测事实，非推理）

冻结锚 `tests/fixtures/proved-java-structure/anonymous-top-level/`：

```java
interface Renderer { String render(); }
abstract class Base implements Renderer { Base(long seed) { … } long seed() { … } }

static Renderer create() {                        // 根方法返回接口，父类是 Base
    final String captured = captureLocal();
    return new Base(choose()) {                    // 分配点直返，super 实参 choose() + 捕获 captured
        @Override public String render() { renderCalls++; return "top:" + captured + ":" + seed(); }
    };
}
```

javap 核实：child `AnonymousTopLevel$1` 的 `super_class` = `Base`（**顶层、不含 `$`**），ctor `(JLjava/lang/String;)V`（super 实参 `long` + 捕获 `String`）；根方法 `create` descriptor = `()LRenderer;`。当前拒绝码 `anonymous_super_return_type_unproved`，文本 "the root method return descriptor is not the exact superclass type"。

## 落点（环 3 已改造过，务必重验行号）

`src/facade.rs` 约 4735–4757（环 3 合入后的形态；**本会话 facade.rs 已两次因相邻片漂移 100+ 行，实现前须以锚点名重验**）：

```rust
if site_shape == jarde_java::report::AnonymousSiteShape::DirectReturn {
    let plain_return   = [b"()L", parent_name, b";"].concat();            // ()Lparent;
    let capture_return = anonymous_val_capture_field(&child_facts).map(|(_, fd)| [(b"(", fd, b")L", parent_name, b";")].concat());  // (P)Lparent;
    if root_method.descriptor.0 != plain_return
        && capture_return.as_deref() != Some(root_method.descriptor.0.as_slice())
    { return Err(unsupported("anonymous_super_return_type_unproved", …)); }
}
let Some((_parent_definition, parent_read)) = resolve_class_source_dependency_read_raw(…);   // ← 4758，门之后紧接
```

**两处返回段都硬编码 `parent_name`**，这就是本环要放宽的唯一位置。该处注释已明写 "a supertype return is ring 2's separate, unproven slice and keeps this refusal"。

## Decisions

1. **一层直接关系，不做传递闭包（MVP）**：声明返回类型 `T`（从 `root_method.descriptor` 的返回段取出）须满足
   `T == parent_name` ∨ `T == parent_read.facts.super_class` ∨ `T ∈ parent_read.facts.interfaces`。
   祖父类、间接接口（`interface A extends B`，而父类 implements A）**保持拒绝**并登记。理由：两 fixture 的关系都是一层直接（`Base implements Renderer`），一层判据即可闭合验收锚；传递闭包需要一个新的层级 walk 共享件（root 已盘点确认**无现成可用件**，见 proposal 事实 4），把它塞进本环会让单片同时改门与新建共享能力，验收无法定位失败原因。
2. **利用既有的 `parent_read`，不新增解析机制**：`resolve_class_source_dependency_read_raw` 在门**之后紧接**（约 4758）就已解析父类 class file。故本环的正确做法是**把返回段检查移到 `parent_read` 可用之后**（或在该处就地补一次解析），直接读 `parent_read.facts.super_class` / `.interfaces`——`ClassMemberFacts` 两个字段都已存在（`crates/jarde-reader/src/classfile.rs:49`）。**不得**为了判据而新建第二套父类解析通路。
3. **`T` 与 `parent_name` 是两个不同的名字，二者都要过可拼写检查**：复用既有 `anonymous_super_source_type_unproved` 判据（不含 `$`、每段合法 Java 标识符、与根类同包），**不得另写一套**。注意本环的典型形里 `T` 是接口名（`Renderer`）、`parent_name` 是类名（`Base`），二者都可拼写；而 `anonymous-capture` 的 `T` 与 `parent_name` **都含 `$`**，会在此判据被拒——**这是正确的**，本环不得放宽它（见 Non-Goals）。
4. **发射：返回位置按 `T` 拼写，分配点仍按 `parent_name` 拼写**。即目标是 `Renderer create() { return new Base(choose()) { … }; }`——与 Java 源码原形一致（声明用接口、new 用实现类）。**不得**把 `new` 的目标也改成 `T`（那会改变构造器绑定：`new Renderer(…)` 在 Java 里对接口是匿名实现，其 super 实参语义与 `new Base(…)` 不同）。
5. **不放宽 `LocalDeclInitializer` 分支**：环 1 的站点形（分配点在局部声明初始化位）本就无返回描述符检查（根方法可以是 `void`），本环与之正交。判据仍包在 `if site_shape == …DirectReturn` 内。
6. **与环 3 的参数表放宽正交且可组合**：环 3 允许 `(P)Lparent;`。本环放宽返回段后，四种组合都应成立——`()`/`(P)` × `Lparent;`/`LT;`。实现时须保持这四条判据的合取结构清晰（建议：先定返回段（`parent_name` 或已证超类型 `T`），再定参数表（空或单个已证捕获描述符），二者独立组合），**不得**写成四条平行的字符串恰等比较（那会让将来加第五种形时再次爆炸）。

## Goals / Non-Goals

**Goals**：`anonymous-top-level` 形（根方法无参、返回父类**直接实现**的接口、分配点直返）内联为 `Renderer create() { return new Base(choose()) { … }; }`；渲染源集 `javac --release 8` exit 0 且 `java -Xverify:all` 事件日志与原 class 逐行一致。

**Non-Goals（不得越界，越界即停手报告）**：
- 不放宽父类名含 `$` 的 `anonymous_super_source_type_unproved`——那是 `anonymous-capture` 的阻塞，属**嵌套父类名可拼写**域（`recover-parameterized-class-headers` 的嵌套名腿或独立片）。**故本环不能闭合 `anonymous-capture`，其验收锚只能是 `anonymous-top-level`。**
- 不做传递闭包（祖父类 / 间接接口）。
- 不放宽参数表（环 3 已交付）、不改站点形（环 1 已交付）、不改 `emit.rs`（环 1 已提供声明位重拼通道）。
- 不新建层级 walk 共享件。

## 不变量（验收必守）

- **环 0/1/3 的三个锚渲染逐字节不变**：`anonymous-super-mixed-direct`（环 0）、`anonymous-super-args` 与 `anonymous-super-args-debuginfo`（环 1）。
- **环 1 的遏制负例逐字节不变**：`anonymous-local-decl-interface-hold` 渲染源码区 SHA-256 须仍为 `1badfcb5b9dcb9a46bf017e3b285073e8424c8143a239f3a6bac9606efc98ce5`（接口路径只认 `DirectReturn`，本环改的是父类路径的返回段，不应触及它——若触及即为判据 5 遏制失效，停下报告）。
- **环 3 的五个负例仍响亮拒绝**，尤其 `supertype-return` 负例——**注意它与本环的锚形极相似**（都是返回超类型），区别在于它是**带参数**形且其父类/返回类型的可拼写性。实现者须先读该负例的源码，确认本环放宽后它**仍应拒绝**还是**应变为接受**：若按判据 1/3 它满足一层直接关系且两个名字都可拼写，则它**应当转为接受**——此时须把该 fixture 从"负例"重新归类为正例并更新环 3 的 spec 与账本（**这是范围变更，须停手报 root 裁决，不得自行改环 3 的 spec**）。
- **分配点唯一性与捕获值来源可证两条不变量不得削弱**（环 0/1/3 建立）。放宽返回段会让更多方法成为投影候选，故这两条是本环唯一的防越界护栏。

## Risks / Trade-offs

- **风险：放宽返回段可能让"返回类型与父类无关"的形被误纳**（例如 `T` 恰与父类的某个接口同名但实为不同类）。缓解：`T` 必须来自 `parent_read.facts` 的**实际** `super_class`/`interfaces` 字节串比较（internal name 全等），不是名字相似性；且 `T` 须过同包+可拼写检查。负例须含"返回类型与父类无任何层级关系"的形（须响亮拒绝）。
- **风险：与环 3 的 `supertype-return` 负例语义冲突**（见不变量第 4 条）。这是本环**最可能踩的坑**，实现者须在取证阶段先解决，不要实现到一半才发现。
- **风险：一层判据在真实代码里覆盖率有限**（`List<String> f(){ return new ArrayList<>(){…}; }` 是一层，但 `Iterable<String> f(){ return new ArrayList<>(){…}; }` 是两层）。缓解：MVP 先闭合一层并如实登记两层的拒绝，不为了覆盖率而仓促建层级 walk 件。

## Open Questions

- **环 3 的 `supertype-return` 负例在本环放宽后应转为正例（root 2026-10-04 已实测裁定，不再是开放问题）**：root 以 javap 核实该 fixture 的事实——child `ParameterizedSupertypeReturn$1` 的 `super_class` = `Base`；`Base` 的 `interfaces: 1`（即 `Renderer`）；三个名字 `Renderer`/`Base`/`ParameterizedSupertypeReturn` **均不含 `$`**（顶层可拼写）。故它**满足本环判据 1（一层直接关系：`Renderer ∈ Base.interfaces`）与判据 3（两名字都可拼写）**，放宽后**应当转为正例**。更强证据：该 fixture 的源码注释**自己写明** "Negative (recover-anonymous-parameterized-root tasks 1.3, **ring 2 boundary**): … this slice widens only the parameter table, never the return type." —— 即环 3 的实现者是**有意**把它冻结为环 2 的边界探针，预期环 2 接手。
  - 它与本环锚 `anonymous-top-level` 的**唯一区别是"根方法带一个捕获参数"**（`create(final String captured)` vs `create()`），而带参形由环 3 交付。故本环落地后它应变为接受，呈现应为 `private static Renderer create(java.lang.String arg0) { return new Base() { … }; }`。
  - **这是范围变更**（须把该 fixture 从环 3 的负例重新归类为正例，并更新环 3 的 spec/tasks 与账本）。按纪律，实现者**不得自行修改环 3 的 spec 文件**——须在报告中提出，由 root 在验收时裁决并执行归类变更。
  - **推论（实现者须注意）**：本环的四种组合（`()`/`(P)` × `Lparent;`/`LT;`）中，该 fixture 正是 `(P) + LT;`，而本环锚 `anonymous-top-level` 是 `() + LT;`。故 tasks 1.1 的实测同时验证了 design 决策 6 要求的"正交组合"结构确实成立——若实现写成四条平行的字符串恰等比较，这个组合会漏掉。
- **传递闭包的真实覆盖率待评估（root 登记，非本环）**：一层判据能闭合本环锚与环 3 的边界探针，但真实代码里 `Iterable<String> f(){ return new ArrayList<>(){…}; }` 这类**两层**形（`ArrayList implements List`、`List extends Iterable`）会被拒。本环 MVP 如实登记该拒绝；将来若实测证明两层形高频，再单独立项建层级 walk 共享件（root 已盘点：`members.rs::subtype_of` 概念匹配但为私有且绑定访问检查机制，提取它是可行路径但须评估所有权与可见性调整）。
- `T` 为**数组类型**或**基本类型**时（descriptor 返回段以 `[` 或非 `L` 开头）应直接拒绝——本环只处理 `L…;` 形的引用返回类型。实现者须确认判据不会误纳非 `L` 形。
