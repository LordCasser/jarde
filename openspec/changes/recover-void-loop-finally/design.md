## Context

[固定 Test2 证据](../../evidence/java-syntax-2026-09-28/cf16-test2-loop-finally/README.md)将 `test(OutputStream):void` 的异常表固定为 `[9,153)→160 any`、`[160,162)→160 any`。正常清理从 BCI 153 开始，handler 在 local 12 保存 Throwable、从 local 2 读取同一 `DataOutputStream` 完成清理后重抛；local 2 的定义在 BCI 8，位于受保护段之前。三处正文循环回边为 `61→35`、`150→76`、`144→115`，均从正常入口可达，不是 Test11 的异常 handler SCC。现有 `Shape::Finally` 只呈现保存返回或具名 catch 后的合流，Test5 的新三行证书则专门证明两个保存返回；把 Test2 归入其中任一个现有完成类型都会虚构返回值。

## Goals / Non-Goals

**Goals:** 将固定双行 void 完成作为现有私有 finally 证书的一个独立完成变体，复用现有 CFG、SSA、Region、Builder、来源和停止预算；在 Java 8 完整类上验证三循环与异常覆盖。

**Non-Goals:** 任意循环数或编译器 lowering、DEX 输入、自动提取 try-with-resources、Test5 的双返回或 Test9 的可空关闭；不增加公开 pass、Region/AST 节点或依赖。

## Decisions

1. **扩展现有单正文/双副本 finally 的完成类型。** Guard 对固定两行顺序、handler 绑定自保护范围、两份清理位于保护范围外、最终 void return 与原 Throwable rethrow 建立有界 `Void` 完成证书。复用 `Shape::Finally` 与其计划/来源链，只给该完成类型携带自保护行和最终完成位置；旧保存返回、具名 catch 证明不放宽。相比为每个循环数或资源类型新增一个 Shape，这保持一个 finally 语义实体；相比直接复用 `SavedReturn`，不会伪造操作栈值。
2. **以 SSA 和边闭合证明资源与循环。** local 2 在 BCI 8 的定义须到达正常/异常两处 `close:()V` 的相同接收者，调用目标也须相同。handler 绑定 local 12、重载和 `athrow` 必须是同一异常值；清理若抛错按 JVM 的正常异常边覆盖待传播异常。Guard 枚举受保护的全部物理块及 canonical 正常/异常边，核三处正常可达循环及所有出口，不根据 `close` 名称或相似指令推断等价。JADX 的重复副本寻找次序可作候选搜索参考，不能替代这些证据。
3. **在现有 Region/Builder 中有界呈现正文。** 参考 Test5 已验收的受保护正文 loop walk，将三处普通循环纳入一个 Region body，并使它恰在正常清理前结束；自保护 handler 不成为源码循环或 catch。Builder 在局部 2 声明、所有正文块/BCI、两份清理来源和预算均可提交时，写一个 `try/finally`、一个 `close()`、最终 void 完成。若既有局部声明规划无法证明 local 2 从 BCI 8 活到两份清理，只为该已证计划提供窄作用域证明，不能全局放松跨 fallback 规则。失败回滚 region 访问及 Builder 状态，保留解释性结果。
4. **物理、行为与输入 profile 分层验收。** 上游集成测试默认 DX；本变更仅对固定 Java 11 classfile 证明。原 class 为语义基准；原转写、固定 JADX Java-input 与新 Jarde 完整类用 `javac --release 8` 重编，以 `java -Xverify:all` 对照零/多个类、正文写入抛错、关闭抛错及同时抛错。private `writeString` 的 Java 11/8 调用 opcode 差异按已记录编译器 profile 解释，不要求字节级同一 class；有效近邻必须各自通过 verifier。

## Risks / Trade-offs

- **正文循环被错认成 handler 循环或漏掉嵌套父循环** → 对三个回边、入口、出口、保护覆盖与所有 canonical 边逐项核验；Region 完整拥有每个正文块后才提交。
- **跨保护范围的 local 2 被拆成不同资源** → 由同一 SSA 定义约束两份 `close()` 接收者，并核源码声明与生命周期；证明失败整体回退。
- **handler 自保护范围误含清理** → 只允许 `[160,162)` 的绑定行，排除 `close()`；以扩围近邻和清理失败路径验收。
- **固定证书被外推到其他 profile** → CF-16 账本只记录 Test2 JVM 切片；DX/DEX 与 Test9 等形态继续单列。
