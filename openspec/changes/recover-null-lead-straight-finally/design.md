## Context

固定 Tf2（[巡查证据](../../evidence/java-syntax-2026-09-30/testfinally-patrol/README.md)，SHA 见其 `results/fixture-sha256.txt`）的 `test([B)LTf2$Result;` 异常表：`[2,25)→32 any` 与 `[32,34)→32 any`（自保护绑定行）。lead：`0: aconst_null; 1: astore_2`。正文 [2,25) 直线：`4: invokespecial getInputStream → 7: astore_2`（同槽赋值）、`10: decode → pop`、`14–21: new Result(400)`、`24: astore_3`（保存返回，区间内）。正常副本 [25,32)：`25/26: aload_0; aload_2; 27: invokespecial closeQuietly(LInputStream;)V`、`30: aload_3; 31: areturn`。handler：`32: astore 4`；副本 [34,41) 同调用；`39: aload 4; 41: athrow`。

`resources()` 的 finally_copy 认领门槛（`completed_field_assignment` + `single_statement`）回答了"字段赋值 lead"；`statement_boundary`/`statement_ends`（preceded 修复引入）提供了语句边界的通用判据。Tf2 需要的是把 null 局部初始化纳入同族回答，并证明副本调用的实参槽身份。

## Goals / Non-Goals

**Goals:** lead 准入第三变体（null 局部初始化）；两份无条件调用副本的目标与实参槽同形证明（实参槽即 lead 槽 s，读同槽合并值流）；保存返回为正文内构造（不跨段移动）；输出唯一 `try/finally` + 一份清理调用。

**Non-Goals:** 条件清理（Tf1/Tf4 家族）；Test9 guarded close；lead 多于两指令或非 null 常量；清理调用带多参/不同实参；保存返回非构造值（普通值已有直体证书覆盖则不重复）；Tf3 的条件正文与提前返回；DEX。

## Decisions

1. **扩展 finally_copy lead 准入，不新增证书。** 在认领门槛处并列第三种 lead：`[aconst_null, astore s]` 且满足 `statement_boundary`（语句完整结束、无栈残留）。进入既有直体证明链后，正文直线性、两份副本逐指令同形、保存/重抛身份、自保护绑定行、canonical 边全集均由现有证明核验；新增的仅有"副本调用的**每个实参槽**读 lead 槽 s 的合并值流"这一槽身份证明（复用 Tf1 证书将建立的 null 出处与合并值流判据的既有形态）。
2. **保存返回留在正文内呈现。** `new Result(400)` 的构造与 `astore_3` 都在保护区间内，呈现为正文末 `return new Result(400);`，不做跨段移动；`astore_3`/`aload_3` 仅作身份证明，不产生额外局部声明（若呈现层需要，遵循既有 saved-return 呈现）。
3. **行为基准分层。** 固定类只作恢复与物理证明对象；行为路径（正常关闭一次返回 400、正文抛错关闭一次并重抛原异常、getInputStream 返回 null 时 closeQuietly 收到 null 不抛、清理抛错覆盖）用探针变体三方 `javac --release 8` 重编 + `java -Xverify:all` 对照。JADX Java-input 把 `inputStream = getInputStream(...)` 重命名为 `inputStream2` 再 `inputStream2 = inputStream` 的失真不作为输出参照。

## Risks / Trade-offs

- **null lead 被外推为任意局部初始化 lead** → 只认两指令 `[aconst_null, astore s]`；其它初始化（方法调用赋值等）继续走资源检查/拒绝路径，负例钉死。
- **与 Tf1 条件清理文法混淆** → 本片副本**无**判空门（无条件调用）；Tf1 负例集已含"文法增删"，两证书互斥命中由副本形状先行判别。
- **实参槽身份伪造** → 副本调用实参必须读 lead 槽 s（SSA 同槽同值流）；实参来自其它槽/常量/调用结果的变体拒绝。
