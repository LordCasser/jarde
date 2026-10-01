## Context

[巡查证据](../../evidence/java-syntax-2026-10-02/compound-guard-patrol/README.md)：T4.nested 拒绝 `jre_guard_finally_copy`@64。字节码结构（取证义务 1：javap 冻结 T4.class 精确记录）——外层 TWR 降低（资源 a 的 close 副本 ×2+suppression）内嵌内层 finally 降低（中段清理 `log.append("mid")` 副本）+ 内层 TWR（资源 b）。三个证书域交叠：TWR(a)、TWR(b)、finally(mid)。第一个取证义务：判定哪两层先被现有 dispatch 尝试、内层 finally 候选在何种语境下进入 `prove_finally_copy`（"lacks the complete straight-body"具体差在哪一环——正文含内层 TWR 降低 or 清理含资源 close）。

## Goals / Non-Goals

**Goals:** T4.nested 恢复（`body[b]mid[a]` 行为一致）；纯两域形态零回退。**Non-Goals:** TWR 内 catch（具名/多 catch）；内层 finally 内抛错路径的 suppression 语义重排（呈现忠实序）；三层嵌套；monitor×TWR 复合。

## Decisions

1. **正文语境子证书**：TWR 正文证明（`Unproven::Body` 的语句子集）接受"内层显式 finally"为其一种正文形态——内层两行 catch-all 表在中层区间自洽、中段清理在正文内、退出汇合回正文；按嵌套 Try/finally region 呈现（复用 finally region 呈现）。不新建跨证书机制。
2. **呈现序忠实**：`mid` 在内层 b.close 之后的降低序按字节码序呈现（外层结构 `try(a){ try(b){body} finally{mid} }`）。
3. **验收锚定**：T4.nested + 变体（无内层 TWR 仅外层 TWR+finally、双层 TWR 无 finally、内层 finally 含 return）。

## Risks / Trade-offs

- **三层证书交叠顺序** → 先取证 dispatch 序再定接入层（可能是 TWR 证书先认、内层 finally 在其 Body 内证，而非 finally_copy 通道）。
- **suppression 边交错** → 判据要求内层 finally 行表不与资源 suppression 行交叠；交叠形态保持拒绝。
