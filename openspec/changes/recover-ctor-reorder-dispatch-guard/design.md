## Context

[回归实证](../../evidence/java-syntax-2026-10-04/ctor-reorder-dispatch-regression/README.md)：`AnonymousSuperDispatch$1` ctor = `putfield val$captured` → `invokespecial Base.<init>` → return；Base 构造期虚调用 `observe()` 读该字段。重排落点在 `crates/jarde-java/src/build.rs`（daa4fb31 的 ctor 呈现改动，+17 行），既有正例测试在 `crates/jarde-java/tests/p3_patterns.rs`（合成捕获/enclosing/双捕获三正例 + 非合成同名、pre-ctor 写两负例）与 `tests/recover_synthetic_ctor_super_order.rs`。

**判据的精确性依据**：已验收的 C1/C2 正例其 super 目标实测均为 `java/lang/Object."<init>"`（javap 记录于回归证据 README）；回归 fixture 的 super 目标为用户类 `Base.<init>`。二者以此单一事实区分。

## Goals / Non-Goals

**Goals:** 消除静默行为回归；既有重排正例逐字不变；反例进入永久测试。**Non-Goals:** 更宽的"可否虚分派"证明（需目标类全体覆写方法的分析——不做，保守即可）；`this(...)` 委托链重排；非合成字段移动；让回归 fixture 变为可编译（那需要 2.10 所说的类级匿名语法与捕获值流证明，属 `present-proved-java-structure` 5.3 域）。

## Decisions

1. **重排安全判据（root 2026-10-04 取证后修正——原判据"仅 Object super"过宽，会造成反向回归）**：重排仅在不改变构造期可见性时进行，分三档：
   - **(a) super 目标 == `java/lang/Object.<init>()V`** → 安全（Object ctor 不可能分派到用户代码）。覆盖 C1/C2 全部既有正例。
   - **(b) super 目标类在本快照有物理定义，且其构造器可证明构造期无法到达子类覆写**——两条都要成立：**(b1)** ctor 内无 receiver 为 `this`（`aload_0`/slot 0）的 `invokevirtual`/`invokeinterface`；**(b2)** ctor 不把 `this` 作为实参传给任何调用（`invokestatic`/`invokespecial`/`invokevirtual` 均算——被调方可能再分派，例如 `invokestatic helper(this)`）。允许的形：`invokespecial Object.<init>`、对**新建对象**的虚调用（receiver 来自 `new`+`dup`，非 slot 0）、无 `this` 实参的 `invokestatic`、以及 slot 0 仅作 `putfield` 的 receiver。**实测该档覆盖 `anonymous-super-args/AnonymousSuperArgs$1`（super `Base` ctor = `Object.<init>` + 自身字段 putfield + 新建 StringBuilder 的 invokevirtual + `invokestatic event(String)`）与 `anonymous-capture/AnonymousCaptureCases$1`（super `AnonymousCaptureCases$Base` ctor，`invokevirtual` 均在新建 StringBuilder 上、`invokestatic access$008:()I` 无实参）**——二者当前重排后既正确又可编译，判据必须保住它们。
   - **(c) 其它**（super 类不在快照、或其 ctor 含 `this` 虚分派）→ **不重排**，保持既有逐字呈现与诊断。`anonymous-super-dispatch/Base`（ctor 内 `invokevirtual observe()` on `this`）落此档，回归消除。
   - 读取 super ctor 走既有快照依赖读取与 A16 计费（`recover-snapshot-hierarchy-widening` 先例），不新增机制；读不到即落 (c) 保守档。
2. **不重排时的行为**：保持本片之前的既有呈现（`this.val$captured = arg1; super();` 逐字）与其诊断/引注——**不得**改为整方法拒绝（那会丢失已恢复的其余成员），也不得静默丢弃该 ctor。
3. **回归测试三向钉死**：(a) `anonymous-super-dispatch` 断言重排**未**发生（捕获写入文本在 `super()` 之前）且不得"可编译且行为不同"；(b) `anonymous-super-args`/`anonymous-capture` 断言重排**仍**发生且文本逐字不变（守 (b) 档不被过窄化）；(c) C1/C2 断言 Object 档逐字不变。

## Risks / Trade-offs

- **过窄**（真实代码中内部类常 extends 用户类，其 ctor 将不可编译）→ 接受：不可编译 + 引注是响亮失败，符合项目既有口径（2.10 明写"独立二进制名类视图需明确不声称该构造器可编译"）。
- **与 bridge 片同时在 build.rs/facade.rs 附近改动** → 本片只动 build.rs 的重排判据（约 17 行范围），bridge 片动 `src/facade.rs` 准入段与 `crates/jarde-java/src/bridge.rs`；**串行实施**（bridge 落地后再派本片），避免同仓冲突。
