## Why

[三元汇合类型巡查](../../evidence/java-syntax-2026-10-05/ternary-merge-type-patrol/README.md)实证：引用型**异型分支**三元 `c ? Integer.valueOf(1) : "s"`（分支呈现类型 `Integer`/`String`，汇合到 `Object`）整方法拒绝——"the two values joined … do not have a conditional Java type this run can prove"。**响亮拒绝（行为安全），非静默偏离**，但整方法损失。

**判别（已实测）**：同型分支（int 嵌套链 `a ? b ? x+1 : x-1 : x*2`、右结合链 `n<0 ? -1 : n==0 ? 0 : …`）**全部恢复**——缺口仅在引用型异型分支的 LUB 汇合形。**jadx 有解**：拆为 `if (z) { return 1; } return "s";` 语句形（[results/jadx-IF.java](../../evidence/java-syntax-2026-10-05/ternary-merge-type-patrol/results/jadx-IF.java)）——语句化回避了条件表达式静态类型证明，是既有合法路径。

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
