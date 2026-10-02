## Context

[巡查证据](../../evidence/java-syntax-2026-10-03/inner-class-folding-patrol/README.md)：N1 fam.jar（`member_family.state="prepared"`、无嵌套声明）。上片机械：`StaticMembers(Vec)` 纯静态族折叠 + 投影四件套。**第一个取证义务**：读 `scan_family_root` 现返回序——混合族当前走哪个分支（`Candidate`? `prepared`?）、静态行在混合族中被哪个先行条件挡出 `StaticMembers`；确定"族内静态子集"的提取点与非静态候选的并存路径（非静态走分离呈现时静态子折叠文本与其互引如何锚定——Stat.use 里 `outer.new Inner(9)` 引用非折叠 Inner，池拼写保留〔第二片域〕）。

## Goals / Non-Goals

**Goals:** 混合族静态子集折叠；Stat 域内源码拼写；N1 家族整 jar 重编行为一致（Inner 分离路径不受影响）；纯静态族与既有通道零回退。**Non-Goals:** 非静态成员折叠与 this$0/限定 new/消桥（第二片）；Stat 体对非折叠 Inner 的引用源码化（分离语境池拼写保持——编译需家族类集，行为一致即可）；孙代。

## Decisions

1. **子集提取**：静态行收集与"非静态候选存在"解耦——静态子集满足上片逐行判据即折叠，非静态候选照旧走分离（互不阻塞）；混合族 reason 文本相应更新（可审计）。
2. **投影边界**：折叠作用域=外围+静态子文本；子对非折叠成员（Inner）的引用保持池拼写（与分离家族平铺同口径）；跨锚（catch/CP owner）沿用上片泛化。
3. **验收锚定**：N1（Stat 折叠、`new Stat()` 源码拼写、行为 `10/7/13`）+ 变体（纯静态+非静态三子混合、单静态+单非静态）；负例（纯非静态族不折叠、M1/M2 纯静态族输出逐字不变）。

## Risks / Trade-offs

- **互引锚定复杂化** → 非静态分离成员的引用不进重写域（池拼写），锚定仍按上片 CP owner 机械；出现无法锚定即拒绝折叠该子（保守）。
- **reason 文本变化影响测试** → 同步更新引用该 reason 的既有断言（可追溯）。
