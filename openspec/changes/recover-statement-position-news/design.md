## Context

[巡查证据](../../evidence/java-syntax-2026-10-03/statement-new-patrol/README.md)：B6 四形判别与 B5.main 复合。既有切片 `refuse-unconsumed-construction-invokes` 的拒绝位与其 CST 反例（`ordinary-new-void-effect`——实参含真实调用的 `CST` 序）。

### 前次实现的取证结论（2026-10-04，root 固化——实现者未落码即被终止，结论可直接复用）

首次派发（qwen 通道）在 128 分钟取证后未落码即终止，但其取证已定位全部落点与两处对原判据的**字面修正**，重派时按此执行、不必重复取证：

1. **真实触发点不是 reader 检查，而是 `written.is_empty()` 分支**：B6.argless/withArg 与 B5.main 三 new 的实拒理由均为 `jre_new_shape` "is read only by instructions this build quotes (BCIs 7/9/17/25)"——reader 是裸 `pop`，而 `renders_its_reads`（`crates/jarde-java/src/init.rs:1329`）不含 `pop`（decode 把 `0x57` 归 `Operation::Other`）。**故须同时动两处**：init.rs 的 pop 认领 + reader 集合，非单点。
2. **实参分类数据面已定位**（无需新机制/新 IR）：`init.rs::verify` 内即可得——`operands` + `value_dependency_bcis`（约 553 行）+ `operations`（`Push`=常量 / `Load`=直读 / `Field` / `Invoke`）+ `ssa`（`Definition::Entry`=参数或局部直读）；已证嵌套构造经 `nested_sites` + `is_the_instance`。
3. **呈现落点**：`build.rs::instruction()` 的 `sites.owns(at)` 提前返回处，在构造器 BCI 写 `StmtKind::Expr`；并确保 pop 侧不被重复成文——先确认 `DiscardedEvaluations`（P3 2c.31，build.rs 约 18919/19400+，其 first arm 已覆盖 call→pop）是否已认领该 pop。
4. **CST 保护面确认无需触碰**：VoidBetween 反例的拒绝来自实参分支的 `Invoke ∉ argument_dependencies`（`jre_new_interleaved_effect`，点名 BCI 4），该分支**先于** reader 检查执行；保持原样并加测试钉死拒绝码与 BCI 4 不变。
5. **字段读实参同 chained 一并保持拒绝（root 裁决，取证已实证）**：`new Foo(o.n)`/`new Foo(静态字段)` 现以 `jre_new_interleaved_effect` 拒绝——`getfield`/`getstatic` 触发声明类 `<clinit>` 是真实副作用；干净对照 `new Ord(Ord.m())` 的源语义为 `Ord.clinit → m() → Ord.ctor` 证明此类顺序问题真实存在。本片判据严格限于 `Push` 常量、`Load`+`Definition::Entry` 直读、已证嵌套构造值三类。

## Goals / Non-Goals

**Goals:** 无序敏感实参语句位 new 呈现；B5/B6 恢复行为一致。**Non-Goals:** CST 形（实参真实调用）保持拒绝；接口注解形（interface 无 ctor）；数组 new 语句位；消费位通道。

## Decisions

1. **判据（沿既有证明）**：new@1 构造证明通过 ∧ 全部实参∈{常量（Push）、局部/参数直读（Load，SSA def=Entry）、已证嵌套构造自身指令（nested-ctor-argument-sites 既有证明）}→ 呈现 `new X(args);`；任一实参含链外 Invoke → 保持原拒绝文本（CST 保护不变，由 `Invoke ∉ argument_dependencies` 分支产生，先于语句位 reader 检查）。
2. **验收锚定**：B5 复合（main 三 new + init 块平铺不变）、B6 的 argless/withArg（consumed 已恢复不变）；变体（多语句 new 混合消费位、静态嵌套类语句 new）；负例（CST 冻结反例逐字不变且拒绝码不变、**`chained` 形 `new B6(new B6(1).n)` 保持拒绝**——实参含 `getfield` 会触发字段声明类的 `<clinit>`，安全性依赖"声明类已初始化"证明（独立事实），本片不引入；登记为遗留边界）。

## Risks / Trade-offs

- **实参含 `getfield` 的形（chained）**：看似"无副作用读"，实际 getfield 触发声明类初始化——其安全需类初始化状态推理。判定为超本片范围，保持拒绝（宁可少恢复一形，不引入跨类初始化证明）。
- **语句位 reader 放行 `pop`**（decode 把 0x57 归 `Operation::Other`，`renders_its_reads` 不含）：放行须与 CST 保护正交——CST 拒绝在实参效果扫描阶段先产生，测试钉死其拒绝码不变。
- **单点修改不足**（取证实测）：真实拒绝出自 `written.is_empty()` 分支，仅改 `renders_its_reads` 或仅改 reader 集合都不足以恢复；实现须同时处理 pop 认领与 reader 集合，并以 B6.argless（最简形）先验证判据打通，再验 B5.main 复合形。
