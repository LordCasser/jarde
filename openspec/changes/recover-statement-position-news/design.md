## Context

[巡查证据](../../evidence/java-syntax-2026-10-03/statement-new-patrol/README.md)：B6 四形判别与 B5.main 复合。既有切片 `refuse-unconsumed-construction-invokes` 的拒绝位与其 CST 反例（`ordinary-new-void-effect`——实参含真实调用的 `CST` 序）。**第一个取证义务**：读该切片的语句位判定处——"实参依赖真实调用"判定的精确位置与当前语句位 new 的完整拒绝路径；确认实参分类数据（常量/局部读/消费链）在哪一层可得。

## Goals / Non-Goals

**Goals:** 无序敏感实参语句位 new 呈现；B5/B6 恢复行为一致。**Non-Goals:** CST 形（实参真实调用）保持拒绝；接口注解形（interface 无 ctor）；数组 new 语句位；消费位通道。

## Decisions

1. **判据（沿既有证明）**：new@1 构造证明通过 ∧ 全部实参∈{常量（Push）、局部/参数直读（Load，SSA def=Entry）、已证嵌套构造自身指令（nested-ctor-argument-sites 既有证明）}→ 呈现 `new X(args);`；任一实参含链外 Invoke → 保持原拒绝文本（CST 保护不变，由 `Invoke ∉ argument_dependencies` 分支产生，先于语句位 reader 检查）。
2. **验收锚定**：B5 复合（main 三 new + init 块平铺不变）、B6 的 argless/withArg（consumed 已恢复不变）；变体（多语句 new 混合消费位、静态嵌套类语句 new）；负例（CST 冻结反例逐字不变且拒绝码不变、**`chained` 形 `new B6(new B6(1).n)` 保持拒绝**——实参含 `getfield` 会触发字段声明类的 `<clinit>`，安全性依赖"声明类已初始化"证明（独立事实），本片不引入；登记为遗留边界）。

## Risks / Trade-offs

- **实参含 `getfield` 的形（chained）**：看似"无副作用读"，实际 getfield 触发声明类初始化——其安全需类初始化状态推理。判定为超本片范围，保持拒绝（宁可少恢复一形，不引入跨类初始化证明）。
- **语句位 reader 放行 `pop`**（decode 把 0x57 归 `Operation::Other`，`renders_its_reads` 不含）：放行须与 CST 保护正交——CST 拒绝在实参效果扫描阶段先产生，测试钉死其拒绝码不变。
