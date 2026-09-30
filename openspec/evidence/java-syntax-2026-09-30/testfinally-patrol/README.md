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

按证书邻近度排序推进中：Tf4（`recover-flag-conditional-finally`）、Tf1（`recover-local-null-conditional-finally`）与 **Tf2（`recover-null-lead-straight-finally`，落地记录见下节）** 均已落地（Tf4 于 f63f7634 全仓 2674/0；Tf1 于 402c9e94 全仓 2682/0、固定类唯一 `try/finally` + `if (local3 != null) { local3.close(); }`）。**Tf3 为当前片**：提前 `return null` + 正文条件流，与 Tf1 共享 `jre_region_exception_edge` 签名；形状已预读——异常表仅两行 `[2,18)→53` 与 `[24,47)→53`（分段绕过早返回块，**无自保护行**），共**三份**无条件 `invokestatic close` 副本（早返回 18–19、正常 47–48、异常 54–55），lead `[aconst_null, astore_1]`、正文同槽赋值@24–28、`bytes` 字段读改写与提前 `return null`——是分段 finally（Test13 族）+ 早返回（Test5 族）+ null lead 的复合形态，需在其条件模型上补正文条件流与提前返回并按证书交叠设计。上游 `TestFinally3.test2NoDebug` 标 `@NotYetImplemented`，JADX 自身亦未完成，属已登记分母调整项。

## Tf4 落地记录（change `recover-flag-conditional-finally`）

Tf4 已由该 change 落地为 Guard 内 `prove_flag_conditional_finally` 证书：两行 any 表 + 自保护绑定行、`[iconst_0; istore]` lead、正文恰一次置真、两份逐参数同形副本（可选共有的 `getstatic; ifeq; new; dup; ldc; invokespecial; athrow` 守卫抛出尾），折叠为唯一 `try/finally` + `if (!success) { result -= 2; }`。本目录新增证据：

- `results/Tf4.after.json`、`results/Tf4.jarde.java`：证书命中后的固定类恢复（structured、单一 finally region、36 个物理 BCI 全有来源；handler 副本字段访问按既有 `jre_field_not_emitted` 折叠记账）。
- `negatives/baseline-refusals.txt`：基线 `86f2e740`（无证书）对固定类整方法回退、对全部负例拒绝；实现后由 `p3_flag_conditional_finally` 全量拒绝（11 个负例，含补丁类 Tf4GuardFieldMismatch）。
- `probe/`：同布局探针变体三方对照。原 class 与 Jarde `javac --release 8` 重编后 `java -Xverify:all` 逐路径一致（正常 `result==1`、call 抛错 `result==-2` 且异常传播、清理抛错 `RuntimeException:cleanup` 覆盖，两探针全路径）；JADX Java-input 作参照——`Tf4CleanupProbe` 三路径一致，`Tf4Probe` 正常路径 `result=-1` 即上文登记的 `z`/`z2` 语义失真（JADX 把 `success = true` 死写为 `z2`，不合并进 `z`），不构成行为正例。输出 SHA 见 `probe/behavior-sha256.txt`。

## Tf1 落地记录（change `recover-local-null-conditional-finally`）

Tf1 已由该 change 落地为 Guard 内 `prove_local_null_conditional_finally` 证书：两行 any 表 + 自保护绑定行、`[aconst_null; astore]` lead、正文同槽赋值全集（均在受保护区间内且右侧值非 null 字面量）、两份逐参数同形副本（四指令 `aload s; ifnull exit; aload s; invokevirtual T ()V`，T 参数化、两份一致，可选共有的七指令守卫抛出尾——与 Tf4 平行），折叠为唯一 `try/finally` + 一份 `if (local3 != null) { local3.close(); }`。本目录新增证据：

- `results/Tf1.after.json`、`results/Tf1.jarde.java`、`results/recovery-sha256-tf1.txt`：证书命中后的固定类恢复（structured、单一 finally region、37 个物理 BCI 全有来源；lead 呈现为分离式 `Cursor local3; local3 = null;`，同 Tf4 的既有裁决）。
- `results/tf1-baseline-replay.txt`：任务 1.1 的可重放基线（class/源 SHA、双行 any 表、`Tf1.base.json` 的整方法回退诊断、固定类 `java -Xverify:all` 通过）。
- `negatives/src/`（9 个负例，每类独立子目录）、`negatives/patch-tf1.py`（4 个同长字节码补丁）、`negatives/verify-Tf1*.txt`（各负例 `java -Xverify:all` 输出）、`negatives/baseline-refusals-tf1.txt`（基线 `baff5438` 对固定类与全部负例均整方法回退、零 finally）、`negatives/recovery-tf1/`（实现后 9 个负例仍全部拒绝的恢复输出）、`negatives/fixture-sha256-tf1.txt`。
- `probe/src-tf1/`、`probe/run-behavior-tf1.sh`、`probe/behavior-tf1-sha256.txt`：同布局探针（正文可注入异常、query 可返回 null、清理可注入异常）三方对照。原 class 与 Jarde `javac --release 8` 重编后 `java -Xverify:all` 逐路径一致（正常关闭一次返回 `v`、正文抛错关闭一次且 `RuntimeException:body` 重抛、query 返回 null 不关闭、清理抛错 `RuntimeException:cleanup` 覆盖，4 路径全过）；JADX Java-input 侧 4 路径亦一致，仍仅作参照（其恢复带巡查登记的死代码 artifact）。三方 run 输出 SHA 相同（`9cc1eae6…`）。

root 复核（证书边界、副本折叠来源、账本标记）属该 change 任务 4.3，另行走查。

## Tf2 落地记录（change `recover-null-lead-straight-finally`）

Tf2 已由该 change 落地为 Guard 内 `finally_copy` 直体证书的 lead 准入扩展（不新增证书）：两行 any 表 + 自保护绑定行、第三种 lead 答案 `[aconst_null; astore s]`（`completed_null_local_lead`：恰两指令、存读 push 值、`single_statement` + `statement_boundary`）、正文同槽赋值 + 两份逐指令同形副本（`cleanup_sequence` 的 `admit_loads` 只由该 lead 授予——字段赋值 lead 与无前置路径的副本文法逐字节不变）、实参槽身份证明 `null_lead_copies_read_the_lead`（每个实参的 copy 内生产者必须是对 lead 槽的 `Load`，其读值经 Tf1 的 `local_null_handler_provenance` 形态展开，且整条语句内 lead 与正文赋值是该槽仅有的定义），折叠为唯一 `try/finally` + 一份 `this.closeQuietly(local2);`。本目录新增证据：

- `results/Tf2.after.json`、`results/Tf2.jarde.java`、`results/recovery-sha256-tf2.txt`：证书命中后的固定类恢复（structured、单一 finally region、`java.io.InputStream local2; local2 = null;` 分离式 lead——同 Tf4/Tf1 的既有裁决、26 个物理 BCI 全有来源；保存返回按既有 saved-return 呈现为正文内 `Tf2$Result local3 = new Tf2$Result(400); return local3;`，构造不跨段移动）。
- `results/tf2-baseline-replay.txt`：任务 1.1 的可重放基线（class/源 SHA、双行 any 表、lead/副本布局、`Tf2.base.json` 的 `jre_guard_finally_copy`@32 拒绝、固定类 `java -Xverify:all` 通过）。
- `negatives/src/`（9 个负例，平源 + 每类独立子目录 class）、`negatives/patch-tf2.py`（4 个同长字节码补丁：目标改指、保存返回改写 null、重抛改写 null、自保护行扩围 [32,41)）、`negatives/verify-Tf2*.txt`（各负例 `java -Xverify:all` 输出全过）、`negatives/baseline-refusals-tf2.txt`（基线 `7a312458` 对固定类与全部负例零 finally）、`negatives/recovery-tf2.txt`（实现后固定类唯一 finally、9 个负例仍全拒）、`negatives/fixture-sha256-tf2.txt`。
- `probe/src-tf2/`、`probe/run-behavior-tf2.sh`、`probe/behavior-tf2-sha256.txt`：同布局探针（正文可注入异常、getInputStream 可返回 null、清理可注入异常）三方对照。原 class 与 Jarde `javac --release 8` 重编后 `java -Xverify:all` 逐路径一致（正常关闭一次返回 400、正文抛错关闭一次且 `RuntimeException:body` 重抛、null 流 decode 容 null、closeQuietly 收到 null 不抛且仍关一次、清理抛错 `RuntimeException:cleanup` 覆盖，4 路径全过）；JADX Java-input 侧 4 路径亦一致（其 `inputStream2` 重命名失真按巡查登记仅作参照）。三方 run 输出 SHA 相同（`45300452…`）。

root 复核（lead 准入边界、实参槽身份、三方行为与账本标记）属该 change 任务 4.3，另行走查。
