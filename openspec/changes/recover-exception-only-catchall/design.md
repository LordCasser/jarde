## Context

见 [proposal.md](proposal.md) 与固定 [仅异常完成边界](../../evidence/jadx-feature-inventory-2026-09-27/finally-exception-only-debt.md)。`FinallyOnce.escaping()V` 的唯一异常行是 `[4,15)→14 any`；BCI 13 主动 `athrow`，handler BCI 14 `astore_0`、15–20 更新计数、23–24 加载同一异常并重抛。行虽包含不抛异常的入口 `astore_0`，清理效果本身在行外。固定 JADX 输出 `catch(Throwable)`，原 class/该输出均为 `state:1`。现有 `finally_copy` 要求正常清理副本；普通 `catches()` 与 Region `starts_catch()` 只读有 CP class index 的具名行，Builder 的 catch header 也从该 index 拼写类型。

## Goals / Non-Goals

**Goals:** 证明固定单行 catch-all 的完整异常独占完成，复用现有 Try/Catch Region 和 Builder 输出等价普通 catch；源映射覆盖所有物理指令及行，失败仍安全拒绝。

**Non-Goals:** 不泛化所有 catch-all 为 `catch(Throwable)`，不为它增加新 finally 形态，不碰 `FinallyOnce.handled()` 的三行/正常清理问题，也不把此切片当作 CF-16 全部追平。

## Decisions

1. **在已有 catch 候选入口建立私有单行证书。** `starts_catch` 可把唯一 catch-all 行的受保护起点送给现有 `try_region`，但 `guard::catches` 只有在证书成立时返回普通 `Catches`。其余 catch-all 仍由原 finally/TWR 路径审阅或安全拒绝；不能把全局 `catch_type_index=None` 等同于源码 `Throwable`。
2. **证明所有入口出口和异常值。** 要求仅一行、无竞争/重叠行，保护范围内完整可达正文没有正常完成或无保护的 throw；所有异常边指向唯一 handler，handler 无外部正常入口、没有分支或其它异常出口；入口 store 对应异常 SSA 值，最后 `aload; athrow` 重抛同一值，清理在中间且可完整呈现。行可以包含 handler 的纯 `astore`，但不可把可能抛出的清理指令纳入自身保护；否则 Java `catch` 的异常优先级可能不同。验证覆盖表与 canonical 边，而非只比字节码文本或方法名。
3. **以显式类型事实表示 catch-all。** 普通 catch clause 当前只有 CP class index 列表，而 catch-all 没有此 index。使用最小的显式替代值区分“具名类型列表”和“经证书认可的任何 Throwable”，交给既有 `CatchClause` 和 Builder 的 catch header 拼写 `java.lang.Throwable`；避免空 `Vec` 或哨兵 index 偷带新语义。普通 multi-catch 行保持原顺序和拼写。
4. **复用 Region/Builder 的原子性。** Region 仍构造一个 `Region::Try` 和一个 `CatchClause`，逐块 owner 不重叠，body 与 handler 都须完整；Builder 仍经原 checkpoint/预算输出 catch。若 handler 的清理、异常身份或来源有缺口，不可发布看似完整的 catch。保护段及 handler 的每个 BCI/异常 row 都有来源，编译出的 catch 在清理抛异常时自然以清理异常替换原异常，与原 finally lowering 等价。
5. **以同布局完整类验收。** 原冻结 `FinallyOnce.class` 的 `handled()` 仍有独立缺口，不能用其整个 Jarde 类的 javac 失败推断本方法。创建只替换无关方法的 Java 8 验收类，逐项核 `escaping()V` 的 BCI/opcode/异常行与原 class 相同；原/JADX/Jarde 三方完整类重编和 `java -Xverify:all` 比较 `state:1` 与异常对象/计数。另测 verifier 有效的正常出口、行扩围/缩窄、竞争类型行、外部 handler 入口、错误重抛和分支清理近邻，拒绝错误归并。
6. **不引入依赖。** 现有 class reader、canonical CFG、SSA、异常行和 Java AST/emit 已有全部所需事实；新库无法替代异常来源及区域所有权证明。解析、运行时解析、验证状态和源码恢复仍各自独立，复放脚本运行的是测试夹具，不是产品自动执行目标代码。

## Risks / Trade-offs

- **单行 catch-all 被误认为所有异常区都可输出 Throwable** → 入口只对精确证书开放，普通具名 catch、TWR/monitor 与共享 finally 定向回归。
- **异常行与入口 store 的单指令重叠** → 单独允许已证无抛出的 store，严格排除清理自身被保护。
- **handler 分支/竞争入口改变异常优先级** → 全图边与逐 BCI 覆盖校验，错误近邻保持拒绝。
- **原冻结整类另有 `handled()` 缺口** → 同布局独立完整类跑三方，不把无关 helper 改动算入 `escaping` 证书。
- **输出一半 catch 或丢来源** → 预算/取消与回滚测试，源映射逐 BCI/row 校验。
