## Context

[巡查证据](../../evidence/java-syntax-2026-10-02/init-block-return-patrol/README.md)：F2 `<clinit>` else 臂内 `return;`。**第一个取证义务**：定位初始化块体呈现的语句发射点（grep `static`/`<clinit>`/初始化块呈现通道；方法末尾 return 的发射处）与"初始化块语境"在该点的可得性，确认抑制落点（很可能是发射前按语境过滤一类）。

## Goals / Non-Goals

**Goals:** 静态与实例初始化块呈现无 return；F2 可重编。**Non-Goals:** 方法/构造器 return（合法忠实呈现）；`<clinit>` 折叠/序（既有）；return 之外的语句。

## Decisions

1. **语境过滤**：初始化块呈现通道在语句发射处过滤 `return`（void 无值形）——语境标识沿块呈现已有分派；不动方法通道。
2. **验收锚定**：F2（重编过 javac、行为 `5:42`）+ 变体（多静态块、实例初始化块含 return、静态块正常路径无 return）；方法/构造器既有输出 diff 逐字不变。

## Risks / Trade-offs

- **误过滤方法 return** → 过滤仅初始化块语境标识下生效；全量 diff 守护。
