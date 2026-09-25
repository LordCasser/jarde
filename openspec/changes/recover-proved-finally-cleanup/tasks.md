## 1. 固定完成语义与拒绝边界

- [x] 1.1 固定非覆盖型 `FinallyNormal` 自写 Java 8 完整类（593B/3Code、SHA-256 `6c1e6ee18ca8370ffc5ad44f8ad911a000ed34f6c258d95d603004d4aeab701a`），记录正常返回、try 中抛错、相同异常对象和调用顺序；对原/JADX/Jarde 完整类分别编译/验证/执行可执行者，root 独立复制重放 `non-overriding/run_audit.py`，不手改输出。
- [x] 1.2 固定覆盖型 return/throw 与 try 抛错的独立负边界（907B/6Code）；root 复制重放四种完成路径，确认原 class、JADX 与 Jarde 完整类差异，尤其 JADX 的 `566`/`8` 错副作用；不得把 JADX 的可编译结果当语义 oracle，或把 catch-all 表项当 finally 证明。
- [x] 1.3 固定返回值快照及 **显式** cleanup `throw` 覆盖完成的 947B/6Code 源类，并用 JVM 验证的异常副本不同值、保护范围缩窄两种受控变体证明等价/覆盖条件不可省；root 复制目录到 `/tmp/jarde-finally-snapshot-root-Njh6jV` 独立重放，三类 summary 逐字节一致，原类 `41/99` 快照、异常身份和 trace `34/56`、负变体 `13/1` 均重现。显式 `throw` 是本切片的负边界，不充当 3.1 中 cleanup 调用隐式抛错的正例；JADX 基线 cleanup trace `344` 不是 oracle，Jarde 三类完整源码当前均 javac 失败。
- [x] 1.4 固定 finally 正文仅有可能抛错的 `cleanup()` 调用的 787B/5Code 正面类；root 复制到 `/tmp/jarde-finally-implicit-root-07ws8h` 以独立 WORK 重放，summary 逐字节相同。原 class 的正常返回/try 异常与 cleanup 调用抛错覆盖两种待完成路径分别保留 trace `29/19`；JADX 的 return 覆盖路径错误重复清理为 `299`，只作对照；Jarde 当前两处引用、完整类缺返回。
- [x] 1.5 从 1.4 的 class 唯一异常表记录只把 `end_pc` 从 20 改为 23，固定 [verifier 有效的范围扩围反例](../../evidence/java-syntax-2026-09-24/finally-range-widened/analysis.md)（SHA-256 `c5407d3f29b135818f003e24b218b5e4b7348d261665f0c71605471a7492935e`）。正常清理 BCI 20 抛错后重新进入 BCI 25 handler，第二次清理使 trace 从 `29` 变 `299`；JADX 完整类保留两个调用并在四行 `-Xverify:all` 对照中与变体一致，Jarde 仍引用。此反例不允许折叠为 `finally`。

## 2. 有界证明与最小结构呈现

- [x] 2.1 在现有 `guard`/pass 机制中证明一个 straight 非覆盖 cleanup 的正常/异常副本等价、操作数与成员身份及每条出口恰好一次；须在比较副本前检查 catch-all 半开保护范围均不包含正常或 handler 清理指令，否则其自身抛错可能重入。以不同 cleanup 目标/参数、竞争 handler、范围缺口、1.5 的范围扩围、重复或额外消费者为拒绝测试，预算/取消沿既有停止契约。
- [x] 2.2 用现有 `Plan`/`Region::Guard` 半开指令范围认领 body、两份 cleanup、handler 与返回，给现有 `Try` AST/emitter 加最小可选 finally 正文；先对直线型正文设无分支/转移门槛，检查融合块中无独立重复语句、无遗漏物理指令，资源/monitor/typed-catch 回归不变。不得让平坦 `body_range` 静默吞掉分支。
- [ ] 2.3 证明 try 返回值在 cleanup 前保存、正常路径返回该旧值，handler 重抛捕获的同一异常；cleanup 自身抛错时按 Java 完成优先级覆盖。已冻结[直线调用抛错四路径样本](../../evidence/java-syntax-2026-09-24/finally-straight-cleanup-throws/analysis.md)；再以[同块前置赋值与返回快照样本](../../evidence/java-syntax-2026-09-24/finally-lead-snapshot/analysis.md)检验已有 `Plan.lead`，不把 lead 错放入受保护范围。`ImplicitCleanup` 的受保护正文含 `if`/`throw`，须通过既有 Region 结构表达或继续引用，不能以平坦范围冒充完整正文。对于覆盖型 return/throw、未知值稳定性或不能呈现的生产者，保守引用完整候选而非生成错义 finally。
- [ ] 2.4 核对默认/完整来源正文相同且涵盖 try、正常和异常 cleanup、保存返回、异常表及 rethrow 的真实 BCI/成员；正文/来源预算、取消和深度界限得到既有有界结果，不留下半个 finally。

## 3. 整类对照与主代理验收

- [ ] 3.1 用重建 Engine/CLI 不修改生成文本地编译/运行非覆盖型完整类，与原 class/JADX 的正常返回、try 抛错与 cleanup 抛错逐行比较；重放 1.2 覆盖型拒绝、资源/monitor/typed-catch 与现有 guard 回归。
- [ ] 3.2 root 独立审查副本等价、完成优先级、区域归属与失败来源，冻结重建 CLI 复跑整类/负例；运行受影响 Rust/Java 测试、reader census、fingerprint、fmt、Clippy、OpenSpec strict 与磁盘核查，分开登记既存债务。
