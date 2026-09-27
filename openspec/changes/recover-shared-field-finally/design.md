## Context

见 `proposal.md` 和 `tests/fixtures/p3-shared-catchall-finally/README.md`。固定 `SharedFinally.handled` 的三行异常表按 ordinal 为 `[4,21)→31 IllegalArgumentException`、`[4,21)→45 any`、`[31,35)→45 any`。正常清理是 BCI `21,24,25,26`，具名 catch 清理是 `35,38,39,40`，共用 handler 清理是 `46,49,50,51`；两份返回保存于 20/34，随后在 29/43 加载，handler 在 45 保存原异常并于 54/55 重抛。原 class SHA-256 `8677378307e4d1bdd422b6dbe6860c9fe1aca6c3eff944258c19078b71bfd0bd`。当前主线实时生成的 Jarde 完整源码仍在 `handled` 引用并缺返回；原/JADX 运行分别为 `normal:1/caught:1` 与 `normal:2/caught:1`。

现有共享 finally 私有证明已完整约束三行、两个子 Region、两个保存返回、原异常重抛和原子 Builder checkpoint，但把每份清理硬编码为单条 `invokestatic ()V`。单独的 `count++` 方法已可由既有字段/算术 Builder 呈现为 `count = count + 1`。固定 JADX 的 `MarkFinallyVisitor` 比较各路径的候选指令并抑制副本；它在此样本的正常路径重复清理，故只借鉴候选匹配顺序，不借用其效果结论。现有 Rust facts/SSA、Region 与 Builder 已提供所需能力；不需要新依赖。此更改属于源码恢复，不涉及 class 解析、Java 方言选择、运行时类解析或对原 class 的验证执行。

## Goals / Non-Goals

**Goals:** 在同一个共享 catch-all 证书中支持相同静态 `int` 字段的三份加常量清理，使固定 `SharedFinally.handled` 的完整类源码可编译且与原 class 的正常/catch 路径行为一致；旧调用型切片保持通过。

**Non-Goals:** 任意副作用序列、实例字段、转换/复合算术、多个具名 catch、`FinallyOnce.escaping()`、通用异常 IR 或对 JADX 输出做语义修补。

## Decisions

1. **扩展现有共享证书，不建平行 finally 机制。** 将单指令清理锚点改为三份有界清理 span；调用型仍是长度为一的证书分支。对字段型，三份各要求精确的 `getstatic`、`int` 常量 Push、`iadd`、`putstatic`，读写同一 `I` 字段且三份字段、常量、运算完全一致。比新建 Guard/Region/AST 路径更少状态，也避免放宽通用单出口 `cleanup_sequence` 影响其他已受证形态。可以把现有 `SharedCallFinally` 私有命名调整为 `SharedFinally`，但不引入新的公开实体。
2. **等价性以值流和效果位置为准。** 对每份副本确认字段读值与常量各仅进入该 `iadd`，结果仅进入同一字段的写回；无外部消费者、额外指令或分支中途入口。读写目标按完整字段身份（static、owner、name、`I` 描述符）比较，不凭相邻 BCI 或文本相同推断；三份清理均在各自 catch-all 半开范围外。两个字面量返回仍在清理前保存并通过各自 SSA 定义加载；handler 仍重抛原异常。字段 `getstatic` 的潜在类初始化效果保留在唯一 source `finally` 的相同路径位置。
3. **保持原子 Region/Builder 边界。** 仍分别恢复 `[4,21)` 与 `[31,35)` 的有界正文，三行由一个证书认领；Builder 在现有 checkpoint 中用首份完整清理 span 形成 `finally_body`，两条返回按原值重建。source map 把其他两份清理及 handler 的 BCI 作为派生来源；若字段更新不能呈现为单句、局部声明不闭合或任何来源遗漏，则回滚整个候选。
4. **用原 class 而非 JADX 判定行为。** 原/JADX/Jarde 完整类分别 `javac --release 8 -g:none`、`java -Xverify:all`。Jarde 两条运行计数必须与原 class 相同；固定 JADX 的 `normal:2` 明示为对照错误。用 class 文件中 verifier 有效的一处字段目标、增量值或异常行变异检验证书拒绝，并复跑调用型、单出口 finally、typed catch、预算/取消回归。

## Risks / Trade-offs

- [只比较指令外形却合并不同目标或别名值] → 完整字段身份与三份 SSA 生产/消费逐一核对，并加字段/增量变异负例。
- [字段更新被错误放入 try 或正常路径执行两次] → 仅由已证明首份 span 写入一个 `finally`，三方完整类运行比较次数。
- [四指令副本内有异常或外部中途入口] → 逐 BCI 异常表覆盖、同块连续性和正常/异常边封闭检查；拒绝扩围 row。
- [改动破坏调用型证书] → 保留长度一分支并复放 `SharedFinallyCall` 的 source map、完整类运行及负例。
