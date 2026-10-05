## Why

[后缀旧值巡查](../../evidence/java-syntax-2026-10-05/postfix-old-value-patrol/README.md)实证（四形判别完整）：javac 为 `i++`（值被消费时）发射快照序列——**旧值 load 先于 iinc 入栈、跨 iinc 被消费**。`int j = i++;` 与 `return i++ + 10;` 拒绝："the value at BCI N is the value local 0 held at BCI M, and the slot does not hold it at BCI N"（时间引注）+ "reads `local1`, and no statement of this body declared that local"。

**判别**：`++i` 前缀（无快照）

> **root 追加锚（2026-10-05，链式赋值巡查）**：**下标消费位变体**——`arr[idx++] = 10`/读位 `arr[idx--]`（旧值作数组下标）同因拒；jadx 以展开 temp 形解（`int i = idx; idx = i+1; iArr[i] = 10;`）——消费位扩至下标位，呈现可取 temp 形（不必内联 `i++`）。见 [chained-assign-sideeffect-patrol](../../evidence/java-syntax-2026-10-05/chained-assign-sideeffect-patrol/README.md)。、`i++; j = i;` 拆两语句（无快照）、复合赋值表达式捕获（捕获**新**值）**全部恢复**——缺口仅在"旧值快照被消费"这一时间序形。**jadx 有解**（语句级 `i++` 或 `i2 = (i - 1) - 1` 旧值算术等价形）。



> **root 追加锚（2026-10-05，[postfix-self-assign 健全性巡查](../../evidence/java-syntax-2026-10-05/postfix-self-assign-soundness-patrol/README.md)）**：三锚同族实证——`i = i++`/`i = i--`（自赋陷阱）/`a[i] = i++`（数组存 RHS 旧值，存语句整条丢失）当前产出**可编译但行为不同**的文本（6≠5/4≠5/2≠102）；恢复落地即覆盖；恢复前的健全性守卫由姊妹片 preserve-postfix-fallback-soundness 承担。**再追加（第 7 锚，[array-store 巡查](../../evidence/java-syntax-2026-10-05/array-store-soundness-patrol/README.md)）**：`elems[size++] = t`（字段数组+字段 post-inc 下标——dup_x1 舞蹈、依赖链诊断族）整语句拒且可编译错文本（null/null vs x/y）——恢复片实现时一并列锚。**静态字段形（[cond-assign 巡查](../../evidence/java-syntax-2026-10-05/cond-assign-soundness-patrol/README.md)）**：`pos < src.length ? src[pos++] : null`（`dup;iconst_1;iadd;putstatic;aaload`——静态字段后缀作数组下标，在三元臂）整方法拒——`arr[idx++]` 消费位变体的字段版，同一旧值双读者判据。



> **root 追加锚（2026-10-05，[postinc-condition 巡查](../../evidence/java-syntax-2026-10-05/postinc-condition-patrol/README.md)）**：**条件位三形**（do-while 扫描 `while(xs[i++] != 0 && …)` / while 复合条件 / if 短路条件位）现全部整方法拒（local crosses——SAFE 但可恢复）；jadx 全解；同族旧值机制，恢复时条件位呈现 `i++` 内联。

## What Changes

把后缀自增识别为语句+表达式双形：当 iinc 的**旧值 load** 跨 iinc 被恰一个消费方读取时，呈现 `i++`（消费位收到旧值语义）——即把快照值建模为"iinc 前的 SSA 值"（distinct 值），呈现层选择 `i++` 表达式形而非两个赋值。**不引入新时间机器**：快照值在 SSA 里本就是 distinct 节点（load 先于 iinc），缺的是呈现层把它归因为后缀形而非"无来源的 local"。

## Impact

- **代码**：`crates/jarde-java/src/build.rs` iinc/局部赋值呈现区（时间引注 "the value at BCI N is the value local … held at BCI M" 的发出处，task 1.1 定位）。
- **测试**：CM/CM2 fixture（四形判别已冻结）+ 健康形零回退。
- **账本**：summary.md 登记行关闭。

## Non-Goals

- **不**做 `i = i++`（自赋值后缀—— famously undefined-ish 形）——负例保持拒绝；
- **不**动复合赋值/前缀/拆语句的既有呈现（零回退锚）；
- **不**处理多消费方共享旧值快照（保持拒绝）。
