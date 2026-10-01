## Context

[巡查证据](../../evidence/java-syntax-2026-10-01/inner-enum-args-patrol/README.md)：现白名单在 `src/enum_constants.rs`（`CTOR_DESCRIPTOR` 等四常量 + 委托链），单构造分支 1668 处拒绝；N0 证明 String 单参折叠通路健康。ctor 体证明（`prove_terminal_constructor_body`）与常量步骤纪律（new/dup/ldc/iconst/args/invokespecial/putstatic）是既有机制，本片只放宽 descriptor 形状与实参语法、呈现侧拼窄化与限定名。

## Goals / Non-Goals

**Goals:** 任意 `(String,int,<≤3 参>)` descriptor 的折叠；四实参语法如 proposal；N3 两探针 + N1 全量形态折叠且整类可重编运行一致；N0 与四固定形逐字不变。**Non-Goals:** 匿名子类常量体（N2，另片）；>3 用户实参（按需扩，避免组合爆炸）；double/float/long 字面量实参（出现时另片，如实登记）；数组/new 表达式实参；跨类验证 getstatic 目标（拼写即忠实，不读兄弟类表）；跨 snapshot 引用。

## Decisions

1. **grammar 参数化而非枚举新 descriptor 常量。** 单构造分支接受"前两参为 (String,int)、总参 ≤5、其余参 descriptor ∈ {int 族/Z/B/C/S、String/L-java-lang-String、任意对象 L…;}"的 descriptor；按参数 descriptor 分类实参期望（int 族 → 字面量窄化拼写；String → ldc 字符串；对象 → getstatic 限定名或 aconst_null）。四固定形走原路径或统一到新 grammar，以"N0 逐字不变"验收。
2. **ctor 体与常量步骤证明复用现有函数**，参数化其期望宽度；实参在常量步骤中的位置逐条对齐 ctor descriptor（第 i 用户参 → 步骤第 i 实参位）。
3. **呈现**：int 族窄化按参数类型（B→`(byte) v`、S→`(short) v`、C→char 字面量、I→裸值）；getstatic 拼 `Owner.name`（内部名转源名沿用既有拼写通道）；null 拼 `null`。
4. **验收锚定**：N3/N1 整类三方重编运行一致（n1.out/n3.out 基线）；负例（实参种类不符、ctor 体有额外语句、>3 参）保持逐字段呈现。

## Risks / Trade-offs

- **getstatic 目标并非静态常量也折叠** → 等价性不依赖目标语义（源码亦写 `Owner.name`，值即字段值）；负例不设此项（非缺陷）；文档注明。
- **组合爆炸** → ≤3 用户实参上限 + 每参独立判定，预算计费沿用。
- **呈现窄化错型** → 参数 descriptor 为准（B/S/C/I），测试钉死 `(byte) 1`。
