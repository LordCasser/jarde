## Context

固定 Tf3（[巡查证据](../../evidence/java-syntax-2026-09-30/testfinally-patrol/README.md)，SHA 见其 `results/fixture-sha256.txt`）的 `test()[B`：lead `0: aconst_null; 1: astore_1`。行表：`[2,18)→53 any`、`[24,47)→53 any`，无自保护行。段一 [2,18)：`2–6: getfield bytes; ifnonnull 38`、`9–13: validate(); ifne 24`、`16–17: aconst_null; astore_2`（早返回值）。间隙 [18,24)：`18–19: aload_1; invokestatic close`、`22–23: aload_2; areturn`（早返回副本，不受保护）。段二 [24,47)：`24–28: astore_1 ← getInputStream()`（同槽赋值）、`29–35: bytes ← read(…)`（字段写）、`38–46: astore_2 ← convert(bytes)`（正常值；38 块从段一的条件跳入）。正常尾 [47,52)：`47–48: close 副本`、`51–52: aload_2; areturn`。handler 53：`astore_3`、`54–55: close 副本`、`58–59: aload_3; athrow`。

关键结构事实：三份副本逐参数同形（`aload_1` + 同 `invokestatic close`）；早返回块不被任何行覆盖——它的 close 抛错直接传播（替换语义与正常路径一致，因正常路径 close 也不在行内）；两个保存值（`astore_2` 的 null 与 convert 结果）经同一槽；正文条件流产生段内跳转与跨段汇合（38 块被两段的正常流共用）。

## Goals / Non-Goals

**Goals:** 一个有界证书覆盖该复合形态：分段两行表与间隙早返回块的**结构**证明（间隙恰为"值保存 + 副本 + areturn"且不含其它指令）、三副本同形与实参槽身份（Tf2 判据）、null 出处（Tf1 判据）、双保存返回身份、条件正文（`ifnonnull`/`ifne` 语义保留为源码条件）与字段写在正文内的所有权；输出唯一 `try/finally`。

**Non-Goals:** 多于一个早返回、间隙含其它语句、三副本不同形、循环正文、自保护行存在的变体、DEX、`TestFinally3` 的 noDebug/smali profile。

## Decisions

1. **新 prove（家族第五证书），结构骨架取 Test13 分段先例。** `SegmentedFinally` 已证"多行同 handler + 段间隙语义"；本证书是其在"两行 + 间隙=早返回块 + null lead + 静态 close 副本"上的窄化：行表恰两行同 handler 且无自保护行；间隙区间逐指令为 `[aconst_null, astore v, aload s, invoke close, aload v, areturn]`；三副本（早返回/正常/handler）逐参数同形且实参槽即 lead 槽 s（复用 `null_lead_copies_read_the_lead` 形态）。条件跳转（`ifnonnull`→段二外、`ifne`→段二内）作为正文语句呈现，不做控制流改写。
2. **双保存返回身份分开证明。** 早返回值恒为 `aconst_null`（字面量证明）；正常值为 `convert` 调用结果（生产者证明）；两 `astore_2` 同槽但值不同属（源码上是两个 return 语句），呈现分别为 `return null;` 与 `return local值;`，不合并为单返回。
3. **无自保护行的语义按行表空缺陈述。** 证书要求行表**不含**任何覆盖 handler 区间的行，且三份 close 均不被行覆盖——清理抛错在所有路径直接替换（与 Java 语义一致）；若变体出现自保护行则不是本证书（属其它家族），负例钉死。
4. **行为基准分层。** 固定类只作恢复与物理证明对象；五路径（正常已缓存 `bytes!=null`、正常未缓存 `validate()==true`、`validate()==false` 提前 null、正文抛错、清理抛错覆盖）用探针变体三方 `javac --release 8` 重编 + `java -Xverify:all` 对照。JADX `inputStream2` 失真不作输出参照。

## Risks / Trade-offs

- **间隙被误当成普通未覆盖块** → 间隙结构逐指令证明 + 副本同形约束；"间隙含额外语句"负例拒绝（那是不可证形状，保持回退）。
- **双保存返回合并错误** → 两返回分开呈现与身份证明；"正常值改写早返回槽"负例拒绝。
- **与 Test13/Test5 证书交叠误触** → 行数（恰两行）、lead 形状、副本调用形态在 prove 入口先行判别；彼此固定形状负例互斥命中。
- **条件正文呈现丢语义** → `ifnonnull bytes / ifne validate` 保留为源码条件语句（复用既有 If 呈现），探针五路径行为对照钉死。
