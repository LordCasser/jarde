## Context

见 `proposal.md` 和[冻结 Test14 基线](../../evidence/java-syntax-2026-09-28/cf16-test14-conditional-cleanup/README.md)。目标 class SHA-256 `8857b84944f1a0c8ec0d8805d2ba0e7430dfadfda7629b4a14ea4064eb1d4ece`，目标 `test()V` 只有异常行 `[0,14)→31 any`。正文 BCI 0–13；正常清理 BCI 14–28，`ifnull` 18 指向共同返回 48；异常 handler 31 保存 Throwable，条件清理 32–43，`ifnull` 36 指向 load/throw 46/47，正常返回 BCI 48。正常/异常副本各含用于判空及用于调用接收者的两次独立 `getfield t`。

`guard::prove_finally_copy` 以直线 `cleanup_sequence` 和保存返回值证明两副本；`Shape::Finally { structured: true }` 只让受保护正文有分支。`region::finally_body` 能恢复有界正文里的 `If`，但 Builder 在 `Shape::Finally` 下以 `body_range(normal_cleanup)` 平铺最终清理。因此只放宽 `cleanup_sequence` 既不能证明副本 CFG 等价，也不能正确写出 finally 内的条件。固定 JADX 的 `MarkFinallyVisitor` 按 try 边界及候选/handler 块图比较重复指令；这里借鉴“按路径比较完整副本”，但使用本项目的异常行、SSA 身份和物理来源证明，不直接抑制相似文本。

## Goals / Non-Goals

**Goals:** 对这个单行、void 正常返回、单次判空/可选调用的双副本建立完整证书，并在现有 `Try` AST 中写一份结构化 finally；三方七路径运行和有效近邻拒绝闭合。

**Non-Goals:** 一般图同构算法、任意分支 finally、多个异常行、带返回值/循环的清理、Test12/Test13、`FinallyOnce`、无关 synthetic accessor 恢复。

## Decisions

1. **用私有条件清理完成形态，留在 FINALLY pass。** Guard 候选要求唯一 catch-all 行、固定两份同构的一次判空/可选调用/唯一出口，记录成现有 finally 证书的受限完成变体或相邻私有 shape。选择最少承载字段；原直线 `FinallyCopyProof` 和其它已受证 shape 保持原约束。与放宽 `cleanup_sequence` 相比，这能一次证明分支两臂、返回位置和异常优先级，不引入新的公开语法机制。
2. **按边与 SSA 配对两份清理。** 对每份副本分别核入口、条件的字段读、判空边、调用臂的新字段读、相同已解析调用目标、空臂和非空臂的唯一汇合出口；两次读取不能被当成同一个 SSA 值。两副本间按操作语义和对应边而非物理 BCI 相等比较。禁止外部正常边进入副本内部、cleanup 自保护、额外异常行、回边与遗漏块。handler 保存的 Throwable 经异常清理正常出口原值 load/athrow；清理调用抛错无本行处理者。
3. **复用有界 Region 与现有 AST。** 受保护正文沿现有 `finally_body` 的 bounded walk 恢复，正常清理副本用私有有界 Region 子走访得到 `Region::If`，Builder 在同一 checkpoint 将其写到 `StmtKind::Try.finally_body`。异常副本仅作为已证等价与 source-map 派生来源，不再写第二次。若当前 Region/Builder 接缝无法完整认领则整个候选回退。无需新公开 IR、文本级重复语句删除或通用 CFG 重写。
4. **目标方法与整类验收分开记账。** 基线 fixture 有与 `test()` 无关的 synthetic accessor 缺口。制作保持 `test()` 每个 BCI/opcode/异常行完全一致的最小完整类，通过调整辅助方法消除该缺口；原、固定 JADX、Jarde 完整 Java 8 类都 `java -Xverify:all` 回放七路径。原类是行为真值，JADX 是参照；完整类通过只证明此同布局切片，不推定所有 Test14 profile。

## Risks / Trade-offs

- [把第一次 `getfield` 缓存为调用接收者] → 核 SSA 两次独立读取、生成文本及字段被正文置空的运行路径。
- [清理抛错被 catch-all 再捕获而重复执行] → 逐 BCI 核单行半开范围与所有异常边，并运行“正文和清理均抛错”路径。
- [一个分支或入口未被 Region 认领] → 对双副本的块/边/指令作集合闭合检查，失败整候选回退。
- [无关 accessor 导致整类失败被误判为本方法失败] → 使用同布局最小完整类作为运行验收，另记原固定类的独立债务。
