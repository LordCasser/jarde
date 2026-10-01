## Context

[巡查证据](../../evidence/java-syntax-2026-10-02/compound-guard-patrol/README.md)：T4.sync 字节码 = 外层 `monitorenter@4 … monitorexit@N`（每异常路径均有），体内 `monitorenter@内 … monitorexit@内配对` + 循环。第一个取证义务：读 guard.rs `monitor` 证书的配对扫描（enter/exit 计数与"每路径 exit"检查），确认内层对的识别位与配对判据（同槽/同对象值）。

## Goals / Non-Goals

**Goals:** 恰一对内层（MVP）配对时嵌套呈现；T4.sync 恢复、行为一致；单层与既有 monitor 形态逐字不变。**Non-Goals:** 三层及以上（登记）；同锁可重入嵌套（monitorenter 同对象两次——javac 对 `synchronized(a){synchronized(a){}}` 会产生，验一形如实登记，不做泛化）；monitor 与 finally/TWR 复合（另片）；不同方法的 monitor。

## Decisions

1. **配对判据复用**：内层 enter 的对象值与某 exit（均在体内、值 SSA 同一）配对即认；配对对从外层计数排除；外层配对证明原样。嵌套呈现为内层 `synchronized (obj) { … }` 块（复用 monitor 呈现通道，无新呈现）。
2. **边界**：内层 enter 前置于外层 enter、或 exit 跨出外层体、或不配对——保持现拒绝；锁对象不同值各自配对（T4 形）。
3. **验收锚定**：T4.sync（`10`）+ 变体（同锁双对象字段、内层在循环内、内层含 return）；负例（内层 exit 缺失、跨出）保持拒绝。

## Risks / Trade-offs

- **可重入同锁形态误配对** → 验一形登记现状（Non-Goal），不猜语义。
- **异常路径外层 exit 证明复杂化** → 外层异常边证明原样不动，内层对仅在正常体路径内配对。
