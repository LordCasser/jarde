## Context

[sif 取证](../../evidence/java-syntax-2026-10-03/single-interface-fold-patrol/sif/README.md)：门命中普查（`results/fold-gate-census.txt`）与 WCallI/WCallC 配对。**第一个取证义务**：定位锚定匹配器的 CP 条目种类枚举位（agent 最小补丁所在——复核其判据正确性：仅加 InterfaceMethodRef owner、不放宽覆盖段健全性）；确认 4 个新折叠 corpus 类的形态。

## Goals / Non-Goals

**Goals:** WCallI 形折叠；4 类 corpus 变化全归类；既有零回退。**Non-Goals:** 根重投影门/Y1（`recover-fold-context-projection-preservation`）；覆盖段健全性变更；接口私有方法（Java 9+）。

## Decisions

1. **单判据位**：匹配器 CP 种类枚举补 InterfaceMethodRef owner（与其余 owner 同权）；健全性校验（覆盖段含 CP 索引）不动。
2. **验收锚定**：WCallI/WCallC 配对 + `F1` 折叠产物重编行为一致 + corpus 4 类变化逐类核对。

## Risks / Trade-offs

- 误锚（InterfaceMethodRef owner 与成员名巧合碰撞）→ 与既有 owner 命中同风险级，既有防御（覆盖段）不变。
