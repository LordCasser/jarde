## Context

见 [proposal.md](proposal.md) 与[冻结 EM-20 对照](../../evidence/java-syntax-2026-09-27/em20-local-scopes/report.md)。`jarde-java::reuse::typed_split` 已以 SSA 值类型、定义/使用、平凡 phi 和“后段不能回到前段”的 CFG 检查分割无 debug 的引用/整数复用。当前它遇到**任何**异常边即拒绝。受控 `synchronizedLoop` 的槽位 2 在 BCI 3 写监视器引用、BCI 20 写循环整数；槽位 3 在清理 handler 的 BCI 14 写异常引用、BCI 22 写循环整数。两个引用都在后段前结束，但同步语句带清理异常边，现有全局拒绝使完整源码在该方法缺返回。`report` 在 `region::recover` 完成后调用 `reuse::plan`，此时已存在 `Region::Guard` 的同步证明，后续命名和声明都消费同一复用计划。

## Goals / Non-Goals

**Goals:** 只在同步 guard 的物理拥有范围、SSA 值链和异常/正常边共同证明两段生命周期不重叠时，允许现有复用计划把引用临时量与之后的 `int` 局部分开；保持预算、取消、来源和现有声明发射。

**Non-Goals:** 通用异常 CFG 槽位分割；把 catch 参数或资源变量任意改名；JADX 的三元表达式、增强 for 与循环头样式；处理固定 Smali 测试未能由 Java 8 `javac` 证明的形态。

## Decisions

1. **复用已有分割计划和 guard 证明。** 将 `recovered.regions` 中已证明的同步 guard 范围作为 `reuse::plan` 的受控上下文，而不是从字节码的 `monitorenter` 名称直接猜。只对该 guard 的监视器/清理临时引用在 guard 结束后的整数写入准入。现有 `NameTable`、类型与声明位置逻辑继续消费 `LocalVariable` 身份；不增加 AST、重命名 pass 或依赖。
2. **异常边逐条验证，不全局放行。** 先保持 `typed_split` 对所有值分类、定义/使用闭合、栈上旧值、phi 和 BCI 边界的现有检查；只对源和目标都由同一个已证同步 guard 拥有的清理异常边放宽早退。连同正常边建立完整可达性，证明后段任何点不能回到前段引用访问，且清理 handler 及其后继不读到后段整数。Call/Return、未知或非 guard 异常边仍拒绝。单凭 BCI 顺序不能替代可达性。
3. **继续使用原子拒绝。** 任一访问无法按两个生命周期唯一归属，或中途预算/取消，复用计划不得发布部分分割；后续 build 继续以原有 fallback 和物理 BCI 报告。不要为了让全类编译而隐藏清理异常路径或凭类型文字强制转换。

## Risks / Trade-offs

- [异常 handler 的自边或多出口被漏算] → 以 guard 物理 block 集和全部异常边验证，测试异常路径及“后段可回前段”的拒绝反例。
- [一个槽位跨区域的旧引用在栈上继续被消费] → 保留现有 SSA uses 越界检查，并对该反例作拒绝测试。
- [局部身份分开但声明仍落在错误词法区域] → 用三方完整类源码 Java 8 重编和 `-Xverify:all` 校验，确认 guard 后循环局部、catch 参数与合流声明各在合法作用域。
- [增加证明成本] → 沿用请求预算计费与取消检查，仅遍历该槽位和相关 guard 的有界图。
