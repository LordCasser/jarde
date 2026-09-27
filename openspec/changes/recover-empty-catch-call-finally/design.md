## Context

见 `proposal.md` 与[固定 Test16 基线](../../evidence/java-syntax-2026-09-28/cf16-test16-empty-catch/README.md)。Java 8 目标方法从 BCI 0 的一次 `invokestatic` 开始，正常、具名 catch、catch-all 的清理分别是 BCI 3、10、17 的 `invokestatic ()V`，BCI 6/13 两个 `goto` 汇合到 22 `return`，异常路径在 16 保存 Throwable、20 加载同值、21 重抛。两条异常行均只覆盖 `[0,3)`，先具名 `Exception→9`，后 catch-all `→16`。当前 `guard::shared_finally_candidate` 的两行证书要求第二行延伸覆盖 catch 正文，故此同范围的空 catch 不能套用；三行共享证书也不能虚构一条 catch 保护行。

固定 JADX `MarkFinallyVisitor` 在 SSA 后按 handler 的路径终点寻找共有指令，并要求候选覆盖相应的所有 handler scope；这个“对每条路径核副本”的原则可借鉴。Jarde 的物理异常行、Canonical CFG、SSA 和来源映射已经可用，本任务用现有证明层核精确事实，不引入通用图同构/新 IR，也不把 JADX 的 `DONT_GENERATE` 标志模型移植到生产架构。解析与 JVM 验证仍由现有 reader/JVM 层负责；这里仅处理源级恢复，测试中的 `-Xverify:all` 是对构造样本的外部验收。

## Goals / Non-Goals

**Goals:** 在真实两行同范围、三份静态零参 void 调用、空具名 catch 的有界形态上，以同一 Guard 证书使 Region 拥有完整物理块，Builder 一次输出唯一 `try/catch/finally`，六条可观察路径与原 class 相同。

**Non-Goals:** 任意 catch 正文、调用带参数或接收者、条件清理、额外受保护段、多层 finally、D8/DX lowering、Test15 的寄存器合并或固定 `FinallyOnce.main`。这些形态保持各自已有证书或保守拒绝。

## Decisions

1. **独立的两行空 catch 证书，沿用 FINALLY pass。** 在 `guard.rs` 的两行分派中先识别精确 `(named, catch-all)` 同 `[start,end)` 形态，再交旧 `prove_nested_join_finally`。要求从方法入口、两行唯一且顺序准确、三份 resolved `Operation::Invoke` 完全一致且 `invokestatic ()V`、SSA 均无实参/栈结果。逐 BCI 核异常覆盖：只有正文 `[0,3)` 同时受两行保护；三个清理副本及两个 handler 后续都在保护外。空 catch 仅为保存具名异常参数，不能含第二个副作用。替代方案是放宽现有三行共享证书并补虚拟 row，这会破坏真实异常行与来源的一致性，故不采用。
2. **先闭合边和身份，再声明重复。** 证明 Canonical CFG 全块分区、每块入口/出口、正常与具名 catch 各一次调用后只到唯一返回、异常 handler 调用后只加载并重抛入口 Throwable；不允许外部边进入任一清理、不允许清理处于自身 catch-all、不能将调用抛错重新吞掉。对具名 handler 的 `astore` 参数做声明/无额外使用验证，对 catch-all 的 `astore→aload→athrow` 做 SSA 同值与唯一消费验证。精确有界模板比在所有方法上运行通用反向路径比较更便于拒绝近邻，并复用现有 `Facts`/预算计费。
3. **Region 表示一个真实的空 catch。** 增加私有 Guard shape，在现有 `Region::Guard` 内按计划边界走访正文与唯一具名 catch，形成 `Region::Try`，catch body 是经证明的空语句序列，handler 的 store 只供 catch 参数声明；两份正常清理和 catch-all 清理由 Guard 独占，不进入正文/handler 文本。Builder 复用现有共享 finally 的单 checkpoint、catch header 和 `StmtKind::Try` 语法通道，但显式区分二行与三行事实，不强行复用三行 row 数组。若当前 Region 边界无法表达空 catch，可在本 shape 内提供有界空 Region，而不向通用 Region 引入“伪语句”。
4. **完整来源与失败原子性。** 唯一 `Try` 的来源由正文、两个 handler、三份调用、两个转移、原异常加载/重抛和返回的物理 BCI 派生；断言所有 11 个 BCI 可查询。任一证书、Region、catch header、预算或输出失败，恢复现有 checkpoint 并回到完整字节码引文或 Stop，绝不能留下半个 try 或吞掉一份清理。
5. **先验收语义，再扩大形态。** 在[固定回放脚本](../../evidence/java-syntax-2026-09-28/cf16-test16-empty-catch/replay.sh)上保持原/JADX 六路径一致，新 Jarde 同样重编运行；再做 verifier 有效的调用目标变化、catch-all 范围扩张、异常行交换、原 Throwable 改写、外部清理入口及清理副本异常覆盖负例。检查 Test12–14、现有两/三/五行 finally、普通 named catch 与预算/取消。所有生产输入仍走已有预算，不自动执行用户 class；运行只限固定测试 fixture。

## Risks / Trade-offs

- **空 catch 的 `astore` 被误写为正文副作用** → 以唯一 handler 参数和空正文证书表示，源码必须保留 `catch (Exception e) {}` 且不多执行语句。
- **清理调用抛错被重复执行或错误捕获** → 逐 BCI 核覆盖与 CFG；异常优先级六路径回放，错误近邻拒绝。
- **共用 Builder 路径的两行/三行状态混淆** → shape 携带真实行 ordinal 和三副本边界，适配现有输出 checkpoint；相关回归并行验证。
- **固定源文件关闭了 JADX 重编检查** → 独立完整 probe 补 Java 8 重编与 `-Xverify:all`，但不把这一结果外推到 D8/DX profile。
