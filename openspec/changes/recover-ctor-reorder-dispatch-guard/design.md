## Context

[回归实证](../../evidence/java-syntax-2026-10-04/ctor-reorder-dispatch-regression/README.md)：`AnonymousSuperDispatch$1` ctor = `putfield val$captured` → `invokespecial Base.<init>` → return；Base 构造期虚调用 `observe()` 读该字段。

### 精确落点（root 已定位——纯接线，无需扩展 `Prologue`）

- **重排主体**：`crates/jarde-java/src/ctor_order.rs::present_prologue_first`（daa4fb31 新增的 255 行模块；`build.rs` 侧仅 +17 行接线，7601 行调用）。判据插入点在 78–101 行取得 `prologue`（`init::Prologue`，携带 `target`/`class`/`declared`/`bci`）与 `prologue_index` 之后、135 行 `stmts[..=prologue_index].rotate_right(1)` **之前**。
- **super 目标的 descriptor 可得**：`Prologue` 只带 `class`（owner 内部名）与 `bci`，**不带 descriptor**；但 `ctor_order.rs:237` 已有 `operations.get(bci)` 先例，且 `Operation::Invoke(CallTarget)`（facts.rs:624）的 `CallTarget` 携带 `kind`/`owner`/`name`/`descriptor`（facts.rs:455–459），`InvokeKind::Special`（facts.rs:449）即 `invokespecial`。故判据读 `operations.get(prologue.bci)` 即可核对 `owner == "java/lang/Object" && name == "<init>" && descriptor == "()V"`，**不必扩展 `Prologue` 结构**。
- `StmtKind::ConstructorCall`（ast.rs:800）只带 `target`/`args`、无 owner，故不能从已发射语句取判据——必须走 operations。
- **既有测试**：`tests/recover_synthetic_ctor_super_order.rs`（family 重编与文本序断言，`before(&child, "super();", write_line)`）与 `crates/jarde-java/tests/p3_patterns.rs`（`capture_ctor_class` 生成器，pool 第 4/8/43 行即 `java/lang/Object`+`<init>`+`()V`——全部既有正例落安全档，零回归）。

**判据的精确性依据**：已验收的 C1/C2 正例其 super 目标实测均为 `java/lang/Object."<init>"`（javap 记录于回归证据 README）；回归 fixture 的 super 目标为用户类 `Base.<init>`。二者以此单一事实区分。

## Goals / Non-Goals

**Goals:** 消除静默行为回归；既有重排正例逐字不变；反例进入永久测试。**Non-Goals:** 更宽的"可否虚分派"证明（需目标类全体覆写方法的分析——不做，保守即可）；`this(...)` 委托链重排；非合成字段移动；让回归 fixture 变为可编译（那需要 2.10 所说的类级匿名语法与捕获值流证明，属 `present-proved-java-structure` 5.3 域）。

## Decisions

1. **单档判据（root 2026-10-04 二次取证后修正——原判据三档需新机制且会反向回归，已否证）**：重排仅在 **super 目标恰为 `java/lang/Object.<init>()V`**（owner 与 descriptor 双匹配）时进行；任何其它 super 目标一律不重排，保持既有逐字呈现与诊断。
   - **健全性**：`Object.<init>` 是 final 的空实现，构造期不可能分派到用户代码，故移动捕获写入不改变任何可观察行为。这是唯一无需读别的类即可证成的档。
   - **为何不做"读 super ctor 体证明无 this 虚分派"的宽档**（原 (b)）：三条实证否证——(i) **跨类读方法体在本仓库无既有先例**：`prove_class_source_bridges` 走 `jarde_jvm::resolve_symbol` 只取**声明**、`prove_outer_super_bridge_use_closure` 消费**已恢复报告**、lambda 伴生读**同类**成员，`Inputs` 仅携带 `direct_super_class` 名字而无 Code；引入跨类读体+decode+计费是把窄修复扩成中大片。(ii) **headers-only 代理（覆写名匹配）会反向回归**：实测 `anonymous-super-args/AnonymousSuperArgs$1` 的 `render()` 与 super `Base.render()` 同名 → 代理判 UNSAFE，但该 fixture 的重排实际安全（其 super ctor 只有 `Object.<init>`+自身字段+新建 StringBuilder 的 invokevirtual+`invokestatic event`，无 `this` 分派）。(iii) **单类事实无法区分**：实测三处 fixture 都"有本类方法读被移动字段"，该判据对三者同为真。
2. **收紧不构成对已验收声明的回归**（关键取证）：`anonymous-super-args` 的既有验收证据 [report.md](../../evidence/java-syntax-2026-09-27/anonymous-super-args/report.md) 第 30–34 行记录的是 **verbatim 序**（`this.val$captured = arg3; super(arg1, arg2);`）与 **"完整源码编译因此退出 1"**——即"不可编译"是该 fixture 在重排切片（10-02）之前的既有如实登记状态。收紧后退回该状态（响亮失败），符合 `present-proved-java-structure` 2.10 的项目立场：*"独立二进制名类视图需明确不声称该构造器可编译；等 5.3 的类级匿名语法、捕获值流及整类运行证明齐全，才由 `new Base(...) { ... }` 让 javac 生成等价的前置合成写入"*。
3. **不重排时的行为**：保持重排切片之前的既有呈现（捕获写入在 `super()` 之前的逐字节序）与其诊断/引注——**不得**改为整方法拒绝（会丢失已恢复的其余成员），也不得静默丢弃该 ctor。
4. **回归测试三向钉死**：(a) `anonymous-super-dispatch` 断言重排**未**发生（捕获写入文本在 `super()` 之前）且不得"可编译且行为不同"；(b) C1/C2 与 `capture_ctor_class` 生成的全部既有正例断言重排**仍**发生且逐字不变（实测其 super 目标全为 `java/lang/Object`，故零回归）；(c) 新增负例：super 目标为非 Object 用户类时不重排（可用生成器构造，无需新 fixture）。
5. **宽档（super ctor 无 this 分派）登记为 5.3 的依赖**：若将来要让非 Object super 的独立二进制名类视图可编译，正路是 2.10 所述的类级匿名语法（`new Base(...) { ... }` 让 javac 自行生成前置写入），而非在呈现层猜测 super ctor 的分派行为。本片不引入。

## Risks / Trade-offs

- **过窄**（真实代码中内部类常 extends 用户类，其 ctor 将不可编译）→ 接受：不可编译 + 引注是响亮失败，符合项目既有口径（2.10 明写"独立二进制名类视图需明确不声称该构造器可编译"）。
- **与 bridge 片同时在 build.rs/facade.rs 附近改动** → 本片只动 build.rs 的重排判据（约 17 行范围），bridge 片动 `src/facade.rs` 准入段与 `crates/jarde-java/src/bridge.rs`；**串行实施**（bridge 落地后再派本片），避免同仓冲突。
