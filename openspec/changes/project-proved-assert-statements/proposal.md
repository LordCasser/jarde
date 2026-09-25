## Why

Java 8 的 `assert` 会编译成类级合成开关、`<clinit>` 初始化与方法内条件抛错。现有 Jarde 已能在最小样本中保留这套行为，却仍把编译器中间结构写回源码；`AssertCore` 的原源码只有一条 `assert guard(ok) : detail()`，Jarde 输出显式字段、`if` 和 `throw`。这是源码形态缺口，不应靠字段名或单个抛错片段猜测修复。

## What Changes

- 在完整类源码请求中，只有同次类事实证明合成开关的定义、初始化与所有使用，以及每条断言的条件、消息、异常和异常边均等价时，原子地投影源级 `assert`；多个断言共享开关时整组准入或整组保留。
- 正确投影时，源码省去会由 Java 编译器重新生成的合成字段与赋值；物理字段、方法、属性、BCI、独立恢复报告和拒绝原因仍可查询。无法证明时保留现有显式代码或明确拒绝，不输出半投影。
- 用原类、JADX 与 Jarde 完整类重编并在 `-ea`、`-da`、选择性启停下对照调用次数、抛错与反射字段；错误状态来源、非 0/1 开关及额外读写作为拒绝边界。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：完整类源码在可证明时恢复 Java 8 `assert` 语句，并保证不能证明时的语义保守性、物理证据与预算原子性。

## Impact

复用 `jarde-java` 的 CFG/SSA、现有条件/字段/抛错形状和 AST 来源，以及 `src/class_source.rs`/`src/facade.rs` 的类级同次证明与投影接缝；不新增 crate、通用跨方法推断器、CLI 协议或运行时依赖。实施前提是 `recover-conditional-values` 对原有显式 `<clinit>` 赋值的行为忠实输出已验收；本项不替它修复 Phi，也不修改独立方法正文。证据起点为 `openspec/evidence/java-syntax-2026-09-24/assert-core/` 及其错误 owner、非 0/1 补丁。
