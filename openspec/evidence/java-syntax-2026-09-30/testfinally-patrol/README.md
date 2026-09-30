# TestFinally 家族巡查：四个待取证文件的首次三方快照（2026-09-30）

[CF-16 里程碑](../../java-syntax-2026-09-28/cf16-milestone.md)"待按文件取证"组的 `TestFinally`、`TestFinally2`、`TestFinally3`、`TestFinallyExtract` 首次取证。主线 `1170c943`（行为与 `b3a01e24` 代码一致）。这是证据快照，不是 OpenSpec change；每个文件的 JVM lowering 形态按固定转录类（[fixture](fixture/)，SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)，`javac --release 8 -g:none`）三方对照。固定 JADX revision `2fb1b16386941660fda07e9017285aec40fcb37f` 的 CLI Java-input 捕获在 [jadx/](jadx/)。

## 结果矩阵

| 转录 | 形态（相对已验收证书的差异） | Jarde 主线 | 固定 JADX Java-input |
| --- | --- | --- | --- |
| Tf1（TestFinally） | 可空局部 `cursor` 前置 null 初始化、**try 内赋值**、条件清理 `if (cursor != null) close()`、保存返回 | `test` 整方法回退：`jre_region_exception_edge`（BCI 0）+ 5 未覆盖块 | 结构恢复；残留死代码 `if (cursorQuery != null) {}` artifact |
| Tf2（TestFinally2） | 可空局部 + try 内赋值 + **无条件** `closeQuietly` 调用清理 + **构造返回**（无保存局部） | `test` 回退：`jre_guard_finally_copy`@32（"lacks the complete straight-body, copy…"） | 结构恢复 |
| Tf3（TestFinally3） | Tf1 + 正文条件流（`if (bytes == null)…`）+ try 内提前 `return null` | `test` 回退：同 Tf1 签名 + 3 个 `jre_field_not_emitted` | 结构恢复 |
| Tf4（TestFinallyExtract） | **正文写局部布尔标志**门控清理（`if (!success) result -= 2;`，清理是字段更新而非调用）+ 保存返回 | `test` 回退：同 Tf1 签名 | 结构恢复，但 `z`/`z2` 标志重命名疑似语义失真（`if (!z)` 读前置值）；上游断言依赖 DX 源码 profile，此处仅作参照 |

原 class 是语义基准；JADX Java-input 输出不构成行为正例（Tf4 的标志重命名即反例）。

## 归因（对照 guard.rs 现有证书）

- Tf1/Tf3/Tf4 共享签名：`prove_conditional_finally`（Test14 条件清理证书）要求 `row.start_bci == 0`——三者 try 前都有**前置语句**（null 初始化或 `iconst_0; istore` 标志初始化），与 [finallyonce-main-catches](../finallyonce-main-catches/README.md) 的 guard 前置语句问题同族但发生在 FINALLY 证书层。此外条件模型只证字段判空（Tf1/Tf3 需要可空局部、Tf4 需要正文写的布尔标志），清理副本只证调用（Tf4 是字段更新）。
- Tf2 是 `finally_copy` 直体证书（Test9 可空资源家族）的邻居：返回值为新构造、清理为无条件调用。

## 下一步（不自动开工）

按证书邻近度排序推进中：Tf4（`recover-flag-conditional-finally`，实施中）与 Tf1（`recover-local-null-conditional-finally`，[spec 已立](../../../changes/recover-local-null-conditional-finally/)，排在 Tf4 后串行——共享 `guarded()` dispatch 注册点）。Tf2 形状已预读：异常表 `[2,25)→32 any` + 自保护 `[32,34)→32`，lead `[aconst_null, astore_2]`，副本为**无条件** `closeQuietly` 调用（三份：正常 25–30、异常 34–39 各一），返回值是 **try 内构造**并保存的 `astore_3`@24——失败点是 `finally_copy` 直体证书的 lead 门槛只认字段赋值，属 Test9 直体家族的"null 局部 lead + 构造保存返回"变体，随该家族扩验另片。Tf3（提前 `return null` + 正文条件流）最后。上游 `TestFinally3.test2NoDebug` 标 `@NotYetImplemented`，JADX 自身亦未完成，属已登记分母调整项。
