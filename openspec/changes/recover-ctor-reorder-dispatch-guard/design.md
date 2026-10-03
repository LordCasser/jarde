## Context

[回归实证](../../evidence/java-syntax-2026-10-04/ctor-reorder-dispatch-regression/README.md)：`AnonymousSuperDispatch$1` ctor = `putfield val$captured` → `invokespecial Base.<init>` → return；Base 构造期虚调用 `observe()` 读该字段。重排落点在 `crates/jarde-java/src/build.rs`（daa4fb31 的 ctor 呈现改动，+17 行），既有正例测试在 `crates/jarde-java/tests/p3_patterns.rs`（合成捕获/enclosing/双捕获三正例 + 非合成同名、pre-ctor 写两负例）与 `tests/recover_synthetic_ctor_super_order.rs`。

**判据的精确性依据**：已验收的 C1/C2 正例其 super 目标实测均为 `java/lang/Object."<init>"`（javap 记录于回归证据 README）；回归 fixture 的 super 目标为用户类 `Base.<init>`。二者以此单一事实区分。

## Goals / Non-Goals

**Goals:** 消除静默行为回归；既有重排正例逐字不变；反例进入永久测试。**Non-Goals:** 更宽的"可否虚分派"证明（需目标类全体覆写方法的分析——不做，保守即可）；`this(...)` 委托链重排；非合成字段移动；让回归 fixture 变为可编译（那需要 2.10 所说的类级匿名语法与捕获值流证明，属 `present-proved-java-structure` 5.3 域）。

## Decisions

1. **单一前置条件**：重排准入要求 super 目标恰为 `java/lang/Object.<init>()V`（descriptor 与 owner 双匹配）。不引入"目标类是否声明可覆写方法"的推断——那需要读目标类且对平台类无源码，保守边界更诚实。
2. **不重排时的行为**：保持本片之前的既有呈现（`this.val$captured = arg1; super();` 逐字）与其诊断/引注——**不得**改为整方法拒绝（那会丢失已恢复的其余成员），也不得静默丢弃该 ctor。
3. **回归测试双向钉死**：(a) 反例断言重排未发生（呈现中捕获写入文本在 `super()` 之前）；(b) 正例（Object super）断言重排仍发生且文本逐字不变。任一失败即回归。

## Risks / Trade-offs

- **过窄**（真实代码中内部类常 extends 用户类，其 ctor 将不可编译）→ 接受：不可编译 + 引注是响亮失败，符合项目既有口径（2.10 明写"独立二进制名类视图需明确不声称该构造器可编译"）。
- **与 bridge 片同时在 build.rs/facade.rs 附近改动** → 本片只动 build.rs 的重排判据（约 17 行范围），bridge 片动 `src/facade.rs` 准入段与 `crates/jarde-java/src/bridge.rs`；**串行实施**（bridge 落地后再派本片），避免同仓冲突。
