## Context

[固定证据](../../evidence/java-syntax-2026-09-28/cf16-testemptyfinally/README.md)把目标 `test(FileInputStream)` 钉在 15 个 BCI：`close()` 位于 0–3；`IOException` handler 为 7–8；catch-all handler 为 11–13 的 `astore; aload; athrow`；两行异常表均保护 `[0,4)`，按具名、any 的顺序指向 7 和 11。正常与具名 catch 都到 14 `return`。原 class、原 Java 8 转写、固定 JADX 的成功、受检异常、运行时异常路径一致；Jarde 当前把 BCI 11–13 留成未覆盖块，整类源码无法重编。

`guard::catches` 已能把具名行映射为普通 catch，`RegionWalker::try_level` 与 Builder 已能表达这个语法。`finally_copy` 必须看到非空清理；`exception_only_catch` 只处理单独 catch-all 与特定清理；二者的合同都不覆盖这里的无效果重抛。证据对象是 JVM classfile；不据此推断 DEX/D8 的空 finally 形态。

## Goals / Non-Goals

**Goals:** 在固定两行、同范围、透明重抛的布局下输出完整 `try { f1.close(); } catch (IOException e) {}`，保持所有可观察路径、异常优先级和全部 BCI 来源；近邻与停止原子拒绝。

**Non-Goals:** 发明空 `finally` AST/Region 节点、吸收任意 catch-all、把源级空 finally 的存在当作从 classfile 唯一可知的事实、改动类外调用或新输入 profile。

## Decisions

1. **在现有 catch 判定中加入有界透明行证书。** 只接受恰好两行：相同 `[0,4)` 范围、先具名 `IOException` 后 catch-all、两个不同 handler、完整覆盖目标受保护指令；handler 11 必须由 `astore` 绑定进来的 Throwable，随后以同一个 SSA 值经 `aload; athrow` 重抛，中间没有调用、字段访问、分配或其他 effect。额外行、目标、入口、handler 自保护范围都拒绝。证书需要核对 canonical 异常与正常边的精确集合、两条正常路径的共同 return、catch-all 无正常出口以及全部物理块归属。复用 `Facts`、`handler_binding` 和已有 catch 结构；不把非空 `finally_copy` 放宽成可接受任意空副本，因为那会让旧 FINALLY 证书失去清理效果前提。
2. **让普通 `try/catch` 明确拥有透明 handler。** `Catches` 的已证形态携带少量物理 handler/BCI 证据，Region 遍历把它作为编译器脚手架消费，而不是写成 `catch(Throwable)` 或独立 fallback；Builder 把被吸收 BCI/异常行锚定到生成的 `try` 来源。没有新增 Region kind 或 AST kind。若 Region 不能完整取得受保护正文、具名 handler、共同出口及透明块，就回滚整个候选，不能先删 handler 再留下半个 catch。另一方案是输出空 `finally`，但源码只需普通 catch，空块会多造一个无法从字节码唯一证明的源级结构。
3. **行为以固定物理类为准。** 钉住原 class 与固定 JADX 版本，分别编译 Jarde 完整类、直接执行固定 class，并对 `close()` 成功、`IOException`、`IllegalStateException` 比较调用次数与异常对象。JADX 的 `MarkFinallyVisitor` 对空副本的消除可用作输出参照，但 Jarde 不靠指令相似性忽略 catch-all；以异常表、SSA 和 edge closure 作证。

## Risks / Trade-offs

- **误吸收有副作用的 handler** → 逐指令白名单、SSA 异常身份与完整边集合同时成立；verifier 有效近邻改变 handler 效果/范围/入口时保持拒绝。
- **具名 catch 的 `close()` 被回退成注释** → 结构与 Builder 一次提交，完整类 Java 8 重编和可观察路径都作为硬验收，不能以漂亮文本或报告状态替代。
- **来源丢失** → 透明指令虽无独立 Java 语句，仍作为结构证书来源附在 `try`；检查目标所有 BCI 与物理行，无未覆盖块。
- **旧 FINALLY/catch 规则受影响** → 本证书只在精确两行布局命中；普通 catch、单行 catch-all、既有 finally 切片、预算/取消回归逐项重放。
