## Context

固定 Tf1（[巡查证据](../../evidence/java-syntax-2026-09-30/testfinally-patrol/README.md)，SHA 见其 `results/fixture-sha256.txt`）的 `test(LContext;Ljava/lang/Object;)Ljava/lang/String;` 异常表：`[2,41)→52 any` 与 `[52,54)→52 any`（自保护绑定行，同 Test2/Tf4 模式）。lead：`0: aconst_null; 1: astore_3`。正文 [2,41)：`anewarray/ldc/aastore`（projection 数组）、`13–20: astore_3 ← context.query(...)`（**同槽重赋值**）、`getColumnIndexOrThrow`、`istore 5`、`moveToFirst`、`getString`、`39: astore 6`（保存返回）。正常副本 [41,49)：`41: aload_3; 42: ifnull 49; 45: aload_3; 46: invokevirtual Cursor.close()V`、`49: aload 6; 51: areturn`。handler：`52: astore 7`；副本 [54,62) 同形（exit 62）；`62: aload 7; 64: athrow`。

Test14 的 `conditional_cleanup_copy` 文法是 `this` 字段读 + 判空 + 同字段 close（六指令，`Definition::Entry` 接收者）；Tf1 的副本是**局部槽**读 + 判空 + 同槽 close（四指令），且清理对象在正文被赋值——SSA 上副本的 `aload_3` 读到的是"lead 的 null 与正文赋值的合并值"。null 初始化出处正是 `if (cursor != null)` 语义的依据。

## Goals / Non-Goals

**Goals:** 一个新的有界证书变体覆盖固定局部可空形态；lead 置 null、正文同槽赋值、两份副本的槽/调用目标/判空方向与保存/重抛身份全部被证明；清理折叠为一份 `if (local != null) { local.close(); }`；复用 `Plan::lead` 与既有 If/调用呈现。

**Non-Goals:** Tf3 的正文条件流与提前 `return null`；字段判空（Test14 已有）；布尔标志（Tf4 另片）；`ifeq`/反转判空、非引用槽、多槽、静态字段清理；任意可关闭资源类型要求（不做 Closeable 语义检查，调用目标由证书参数化）；DEX 输入。

## Decisions

1. **独立副本文法（四指令），与 Test14/Tf4 平行。** `aload s; ifnull exit; aload s; invoke target ()V`，两份逐参数同形（slot s、target 一致，仅 exit 与结尾保存/重抛不同）。SSA 证明两份 `aload s` 读同一槽的同一合并值流、`ifnull` 恰守 null 路径（跳过清理）、invoke 接收者即该值。正文同槽赋值次数不设上限但须全部位于受保护区间内且每次赋值的右侧值有出处（lead null 除外）；赋值在区间外或槽被其它槽别名则拒绝。
2. **lead 准入只认 `[aconst_null, astore s]`。** 与 Tf4 的 `[iconst_0, istore s]` 同构：两指令、常量 null、槽即副本条件槽。`Plan::lead` 记录 `(0, protected.0)`，Region 呈现 `Cursor cursor = null;`。通用前置语句机制不在 FINALLY 证书内（属 Catches 路径）。
3. **清理呈现复用既有 If + 调用。** 折叠后一份 `if (cursor != null) { cursor.close(); }`：null 比较与短路结构复用 Test14 清理体呈现通道；接收者为 lead 声明的局部。两份副本 BCI 均映射唯一清理体。不为局部判空新造表达式机制。
4. **行为基准分层。** 固定类只作恢复与物理证明对象；行为路径（正常关闭一次、正文抛错时关闭一次并重抛原异常、query 返回 null 时跳过关闭、清理抛错覆盖）用探针变体三方 `javac --release 8` 重编 + `java -Xverify:all` 对照。JADX Java-input 的死代码 artifact（空 `if (cursorQuery != null) {}`）不作为输出参照。

## Risks / Trade-offs

- **槽合并值伪造 null 出处** → null 出处必须唯一指向 lead 的 `aconst_null`；正文赋值与副本读经 SSA 同槽同值流证明；"无 lead null"变体拒绝（那是不条件清理或另一文法）。
- **与 Tf4/Test14 dispatch 交叠误触** → 文法形状判别在 prove 入口先行（行表、副本指令序、判空方向），互斥命中；负例含彼此的固定形状不误触。
- **清理调用抛错覆盖语义** → 证书证明自保护行只绑 `astore`（不含 close），清理异常沿 JVM 异常边覆盖原异常；探针双失败路径三方对照钉死。
