## Context

固定 Tf4（[巡查证据](../../evidence/java-syntax-2026-09-30/testfinally-patrol/README.md)，class SHA 见其 `results/fixture-sha256.txt`）的 `test()Ljava/lang/String;` 异常表为 `[2,21)→37 any` 与 `[37,39)→37 any`（自保护绑定行，同 Test2 模式）。lead 是 `0: iconst_0; 1: istore_1`（标志初始化）；正文 [2,21) 含 `call`、`result++`、`17: iconst_1; 18: istore_1`（恰一次置真）、`19: astore_2→20: astore_3`（保存返回）；正常副本 [21,37) 为 `iload_1; ifne 35; aload_0; dup; getfield result; iconst_2; isub; putfield result; aload_3; areturn`；handler 37 `astore 4` 后副本 [39,53) 同形以 `aload 4; athrow` 结束。现有 `prove_conditional_finally` 的 `conditional_cleanup_copy` 是六指令 `this` 字段判空+close 文法且要求 `row.start_bci == 0`，两者均不适配。

## Goals / Non-Goals

**Goals:** 一个新的有界证书变体覆盖固定标志形态；lead（标志初始化）、正文置真与副本条件之间的 SSA 关联全部被证明；复用 `Plan::lead`、Region 的 finally 正文通道与 Builder 的字段复合更新（`spell-proved-field-unit-updates`）。

**Non-Goals:** Tf1/Tf3 可空局部条件、Tf2 构造返回直体、条件反转（`ifeq`）、非 int 字段或非减法读改写、`iinc` 形态、任意标志次数、DEX 输入；不放宽 Test14 或任何既有两行证书。

## Decisions

1. **独立 prove 函数 + 新副本文法，不扩 Test14 文法。** 与 `conditional_cleanup_copy` 平行新增八指令副本文法：`iload s; ifne exit; aload_0; dup; getfield F; iconst K; isub; putfield F`，两份副本逐指令同形（slot/F/K 一致，仅 exit 与保存/重抛结尾不同）。参数化 slot/F/K/op（首片仅 `isub` 与 int），负例覆盖同形破坏点。
2. **lead 准入证明初始化，不放松区域规则。** 证书要求 lead 恰为 `[iconst_0, istore s]` 且 s 即副本条件 slot；`Plan::lead` 记录 `(0, protected.0)`，Region 在正文前置呈现 `boolean success = false;`。正文恰一次 `iconst_1; istore s` 在受保护区间内、保存返回之前，作为 `if (!success)` 语义的 SSA 依据；两次写、写后仍有语句或写在区间外均拒绝。
3. **清理体复用既有复合更新呈现。** 两份副本折叠为一份 `if (!success) { this.result -= 2; }`；`result -= 2` 的拼写与来源映射复用 `spell-proved-field-unit-updates` 的既有证明，不在本证书内新造表达式机制。
4. **行为基准分层。** 固定类只作恢复与物理证明对象；行为路径（正常 `check()==1`、`call()` 抛错时 `result==-2` 且异常传播、清理自身抛错覆盖原异常）用同布局探针变体（静态开关控制 `call()` 抛错）三方重编对照。JADX Java-input 的 `z`/`z2` 重命名按巡查登记视为疑语义失真参照，不作为行为正例。

## Risks / Trade-offs

- **标志置真与副本条件的关联被伪造** → slot 身份、恰一次写、写位置区间由 SSA 逐项证明；"无置真写"变体拒绝（那是无条件清理，属另一文法）。
- **lead 准入被外推成通用前置语句机制** → 本证书 lead 只接受两指令常量 false 初始化；通用前置语句属 `recover-preceded-statement-catches` 的 Catches 路径，不混入 FINALLY 证书。
- **字段读改写呈现与来源错位** → 复用既有字段更新呈现的来源契约；折叠后两份副本 BCI 均映射到唯一清理体，测试断言全覆盖。
