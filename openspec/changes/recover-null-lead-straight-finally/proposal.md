## Why

[TestFinally 家族巡查](../../evidence/java-syntax-2026-09-30/testfinally-patrol/README.md)中 Tf2（JADX `TestFinally2`）的 `test(byte[])` 主线在 `jre_guard_finally_copy`@32 拒绝：直体 `finally_copy` 候选已识别（单行 catch-all、直体正文、保存返回、两份相同清理调用），但 lead 准入只认"try 前无语句"或"try 前是完整字段赋值"（`completed_field_assignment`）。Tf2 的 lead 是 `[aconst_null, astore_2]`（`InputStream inputStream = null;`），正文同槽赋值 `getInputStream()`，清理为**无条件** `closeQuietly(inputStream)` 调用两份，返回值是 **try 内构造**并保存的 `new Result(400)`（`astore_3`@24，在保护区间内）。固定 JADX Java-input 结构可恢复（仅作参照）；原 class 是行为基准。

## What Changes

- 在既有 `finally_copy` 直体证书的 lead 准入处增加第三种回答：lead 恰为 `[aconst_null, astore s]`（两指令 null 局部初始化）时，按与字段赋值 lead 同样的"语句完整结束 + 无栈残留"判据（复用 `statement_boundary`/`single_statement` 一族）交由直体证书完整证明——两份清理副本是无条件同目标调用且实参槽即 s、保存返回为正文内构造、原 Throwable 身份重抛。
- 证明成功输出唯一 `try/finally`：lead 呈现局部声明与 `= null`，清理折叠一份 `closeQuietly(inputStream);`，保存返回呈现为正文 `return new Result(400);`（构造位置即正文内，无跨段移动）。
- 不新增 Shape、公开 IR 节点、CLI 开关或依赖；Test9 可空资源证书（guarded close）、字段赋值 lead 路径、`prove_flag_conditional_finally`/`prove_local_null_conditional_finally`（Tf4/Tf1，条件清理家族）互不触碰。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：受证的 null 局部 lead + 直体正文 + 两份无条件同目标调用清理 + try 内构造保存返回，可恢复为唯一 `try/finally`。

## Impact

仅 `crates/jarde-java` 私有 Guard（finally_copy lead 准入与副本文法的实参槽证明）及测试；Tf3（正文条件流 + 提前返回 null）不在本片。实施顺序：待 `recover-local-null-conditional-finally`（Tf1）落地后串行（同为 finally 家族文件触点，避免 dispatch/装配交叠）。
