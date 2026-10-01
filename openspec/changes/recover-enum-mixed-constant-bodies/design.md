## Context

[巡查证据](../../evidence/java-syntax-2026-10-01/inner-enum-args-patrol/mixed-bodies/README.md)：`p.Combo` 的 `<clinit>` 带体常量步为 `new Combo$1; dup; ldc "ADD"; iconst_0; iconst_1; invokespecial Combo$1.<init>(String,int,int)`；子类 ctor `aload_0; aload_1; iload_2; iload_3; aconst_null; invokespecial Combo.<init>(String,int,int,Combo$1?)`（桥带 marker）；普通常量 `ID(0)` 直构主 ctor。既有两通道：constant-bodies 义务（子类/桥恰零源参）、arbitrary-arguments grammar（无体常量直构）。第一个取证义务：javap 冻结 jar 核对组合形态的桥/子类/主 ctor 描述符与转发序列，定位现有义务证明的首个拒绝点（大概率是常量步骤的 invokespecial 目标子类 ctor 描述符校验或关系解析的 marker 子句）。

## Goals / Non-Goals

**Goals:** 组合形态折叠（带体+带参常量、与普通带参/纯带体常量混排）；`p.Combo` 全量折叠且三方重编运行一致；两纯能力逐字不变。**Non-Goals:** 双基声明词表（另一片）；嵌套+混合叠加（嵌套名修复后应自然工作，验一形登记，不做深度组合矩阵）；>3 参、数组/表达式实参（沿用 arbitrary-arguments 边界）；体方法非 structured（沿用保守）。

## Decisions

1. **义务参数化而非新义务**：把 constant-bodies 的"子类/桥零源参"形状参数化为 arbitrary-arguments 的参数集；委托链证明逐参对齐（第 i 用户参的转发指令序列按参数种类核：int 族 load/ldc/getstatic + 末尾 marker null）。marker 绑定沿用 nested-bodies 修复后的结构事实。
2. **呈现合成**：常量体呈现 = `NAME(` + args 拼写 + `) {` + 体方法文本 + `}`；两段拼写各自复用既有函数，不写第三份。
3. **验收锚定**：`p.Combo`（三常量混合）折叠 + 三方一致；diff 断言 `demo.Op`（纯体）、`N0`/`N3`（纯参）不变；负例（转发链断参、体方法拒绝）整组逐字段。

## Risks / Trade-offs

- **委托链证明复杂化** → 逐参种类分支复用 arbitrary-arguments 的实参判定函数；上限 ≤3 参。
- **呈现重复** → 决策 2 强制复用；测试断言拼写与两纯能力一致。
