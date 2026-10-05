## Why

[链式赋值巡查](../../evidence/java-syntax-2026-10-05/chained-assign-sideeffect-patrol/README.md)实证：`CH.a = CH.b = CH.c = 5` 拒——javac 发射 `iconst_5; dup; putfield c; putfield b; putfield a`，dup 值跨 putfield 存活（"copy … has no proved local assignment"）。**jadx 有解**：三个独立赋值（`c = 5; b = 5; a = 5;`——**从右到左求值序保留为语句序**）。

**判别（完整）**：局部链 `x = y = 7`（dup 跨 **istore**）**已恢复**（拆双赋值）——唯一变量是存储目标为字段（putfield）。这是与数组 dance 片（#8）、后缀旧值片（#9）同族的第三个 dup/快照值形状：**dup 跨 putfield**。



> **root 追加锚（2026-10-05，[field-compound 巡查](../../evidence/java-syntax-2026-10-05/field-compound-soundness-patrol/README.md)）**：**dup 跨 getfield+putfield 的复合 RMW**（`this.flags |= 1 << bit`——receiver dup 双读者：getfield 与 putfield 各一次）当前整语句拒；静态形（无 dup）恢复。实现时核实与本片 dup-跨-putfield 机制是否同门（同门则一并覆盖，异门则如实记录移交）。**String 复合形（[field-string-compound 巡查](../../evidence/java-syntax-2026-10-05/field-string-compound-patrol/README.md)）**：`this.field += "…" + x`（javac 发 `new SB; dup_x1; getfield; append…; toString; putfield`——receiver dup_x1 跨整条 SB 链，copy+依赖链双族诊断）整方法吞剩 `return this;` 可编译错（f vs f[x][y]）——最高频字符串累积模式；局部版已健康（SB 链折回 + 链），字段版为本片核心锚。

## What Changes

把链式字段赋值识别为多赋值语句组：当 dup 值跨**恰好 n 个 putfield** 存活且最终无剩余消费时，呈现为 n 个独立字段赋值（按字节码序=源求值序从右到左）。判据=dup 单源（常量/表达式求值一次）+全部消费是 putfield——**求值一次语义**由拆分后各赋值共享同一已求值表达式文本保证（jadx 同构）。

## Impact

- **代码**：`crates/jarde-java/src/build.rs` putfield 呈现区（"copy … has no proved local assignment" 对 dup 的发出处——与 #8/#9 片同区不同形状，task 1.1 定位确认）。
- **测试**：`CH` fixture + 局部链零回退 + 单字段赋值零回退。
- **账本**：summary.md 登记行关闭。

## Non-Goals

- **不**恢复链式语法形（`a = b = c = 5`）——呈现为拆分赋值（jadx 同构，等价）；
- **不**动局部链既有呈现（零回退锚）；
- **不**处理混合链（`a = (b = 5) + 1`——表达式内消费）——保持拒绝待实测。
