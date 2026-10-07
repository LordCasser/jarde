# 嵌套锁 finally 体与可中断锁（recover-nested-lock-finally-bodies）

## Why

[多锁巡查](../../evidence/java-syntax-2026-10-08/multi-lock-patrol/README.md)：`a.lock(); b.lock(); try { … } finally { b.unlock(); a.unlock(); }`（嵌套双锁——死锁避免/分段锁高频形）与 `a.lockInterruptibly(); try { … } finally { a.unlock(); }`（可中断获取）均整方法拒（`prove_finally_copy` 四件套）。jadx 两形完整解。**多等待点实测已恢复**（边界关闭）。两缺口：

- **A（多语句 finally 体）**：lock/resource-guard 证书的 finally 体只证单调用；嵌套锁的 finally 含两 unlock。
- **B（行前可抛调用）**：四件套拒绝行前可抛调用——但 `lockInterruptibly()` 的异常行范围 `[7,24)` **不含**该调用（字节码自证：调用完成才入保护区，抛出时 unlock 不跑），拒绝过保守。

## What Changes

- 证书族扩展两形状（sibling 准入，LK/IO 判据逐字不动）：
  - **A**：finally 体=语句序列（≤2 语句 MVP），序列中每一 unlock 的接收者与某一行前 lock 调用 **SSA 同一**（释放序=获取逆序，嵌套锁不变量）；
  - **B**：行前可抛调用准入条件="调用在全部异常行范围**之外**"（字节码行集自证）——`lock()`（不抛）与 `lockInterruptibly()`（行外可抛）统一；
- throws 子句如实呈现（InterruptedException 声明在 B 形）；
- 嵌套体（try 内再 lock/try/finally）与 >2 锁登记边界。

## 硬不变量

1. LK/IO 全锚渲染逐字节不变（两证书判据未触）；
2. 真两份清理副本形/顺序异常形保持拒绝；
3. 不得产出"可编译且行为不同"文本（解锁序=控制流事实，normal/exception 双驱动对照必测）。

## 验收

- ML.nestedLocks/interruptibly 恢复（0 该族诊断），整类剥离编译 exit 0、`-Xverify:all` 输出 `2` 与原一致；multiAwait 零回退；
- 门控实验先行（A/B 各自独立门控）；全门禁 + oracle ignored 腿 + corpus 指纹。

## Capabilities

### Modified Capabilities

- `java8-recovery`：嵌套锁的多语句 finally 体与行外可抛获取调用按源码形态呈现，方法行为完整。
