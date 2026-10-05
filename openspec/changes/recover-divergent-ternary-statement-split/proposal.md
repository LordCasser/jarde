## Why

[三元汇合类型巡查](../../evidence/java-syntax-2026-10-05/ternary-merge-type-patrol/README.md)实证：引用型**异型分支**三元 `c ? Integer.valueOf(1) : "s"`（分支呈现类型 `Integer`/`String`，汇合到 `Object`）整方法拒绝——"the two values joined … do not have a conditional Java type this run can prove"。**响亮拒绝（行为安全），非静默偏离**，但整方法损失。

**判别（已实测）**：同型分支（int 嵌套链

> **root 追加锚维度 4（2026-10-05，[clinit/unbox 巡查](../../evidence/java-syntax-2026-10-05/clinit-throw-unbox-patrol/README.md)）**：**布尔字面量分支（boolean 返回位）**——`b ? true : false` 拒但 javap 证实与恢复的 `b ? 1 : 0` **指令完全相同**（iconst join at ireturn）——差异纯在布尔返回位的类型检查；jadx 恒等折叠 `return b`。五数据点。

> **root 追加锚维度 3（2026-10-05，[catch-order/recursion 巡查](../../evidence/java-syntax-2026-10-05/catch-order-recursion-patrol/README.md)）**：**布尔字面量 vs 调用返回**——`n==0 ? true : odd(n-1)`（互递归三元）拒（同 "two values joined" 诊断）；判别矩阵：同型 int 三元+自递归✓、if/else 互递归✓、字面量-vs-调用✗——四变体（异型引用/上转型/短路条件/字面量-vs-调用）实现时插桩确认落点关系。

> **root 追加锚维度 2（2026-10-05，[ternary-chain 巡查](../../evidence/java-syntax-2026-10-05/ternary-chain-patrol/README.md)）**：**短路复合条件作三元条件**——`(a && b) ? "x" : "y"`（分支**同型** String/String！）也拒——判别矩阵：短路复合条件 if 消费✓/三元消费✗（简单条件两者✓）。jadx 直接还原 `(z && z2) ? …`。**注意**：该形与异型汇合前提无关（分支同型仍拒）——是"短路位技巧进三元"的独立维度；若实现发现三变体（异型汇合/上转型汇合/短路条件）不同落点，按变体分派报告。

> **root 追加锚（2026-10-05，多态巡查）**：**子类上转型汇合形**——`c ? new Q() : new R()`（Q/R 同父 P，返回类型 P）：修复前诊断不同（两 new 各自的 copy 无局部证明 + "entry state of stack depth 0"，非 "two values joined" 文本）但**同因**（汇合点无可证条件类型）。task 1.1 定位时注意该变体可能走不同代码路径（栈深 0 的汇合值 vs 表达式内汇合）——若实现发现两变体不同落点，先报告再动。见 [polymorphic 残留巡查](../../evidence/java-syntax-2026-10-05/polymorphic-remainder-patrol/README.md)。 `a ? b ? x+1 : x-1 : x*2`、右结合链 `n<0 ? -1 : n==0 ? 0 : …`）**全部恢复**——缺口仅在引用型异型分支的 LUB 汇合形。**jadx 有解**：拆为 `if (z) { return 1; } return "s";` 语句形（[results/jadx-IF.java](../../evidence/java-syntax-2026-10-05/ternary-merge-type-patrol/results/jadx-IF.java)）——语句化回避了条件表达式静态类型证明，是既有合法路径。

## What Changes

当条件表达式的两分支值呈现类型异构（无公共具体可证类型）**且**该表达式的消费方式允许语句化时，把三元**拆为 if/else 语句形**呈现（与 jadx 同构）；消费方式不允许语句化（如嵌在实参中）时**保持既有拒绝**。不新增类型证明机制——语句化是呈现层选择，绕开（而非解决）汇合类型证明。

## Impact

- **代码**：呈现层（`crates/jarde-java/src/build.rs` 或 `region.rs` 的条件表达式消费点——拒绝文本 "the two values joined … conditional Java type" 的发出处，实现者 task 1.1 定位）。
- **测试**：`IF.poly` fixture（巡查已冻结）+ 嵌套实参形负例（保持拒绝）+ 同型分支零回退。
- **账本**：summary.md 的三元异型汇合登记行关闭。

## Non-Goals

- **不**实现 LUB/条件类型证明（若未来有独立片，语句化仍是合法回退）；
- **不**改同型分支的既有三元呈现（零回退锚）；
- **不**触碰短路布尔的位技巧呈现（已判行为等价的呈现债，另域）。
