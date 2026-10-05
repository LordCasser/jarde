## Why

[数组初始化值位巡查](../../evidence/java-syntax-2026-10-05/array-initializer-value-patrol/README.md)实证（六位判别完整）：javac 为 `new T[]{…}` 发射 dance（`newarray; dup; <index>; <value>; iastore` 留引用于栈），该 dance 值在**任意消费方位**被拒——`a[0] = new int[]{7}`（预存数组元素存储的 RHS）与 `return new int[]{9}[0];`（新鲜数组的立即下标），诊断均为 "the copy at BCI N has no proved local assignment"。**jadx 两形皆完整恢复**。

**双重证伪**排除了两个候选机制：非 dance 本身（局部赋值/字段赋值/实参/外层初始化器元素**四位合格**）；非立即消费（`new int[2].length` 裸数组立即消费**合格**）——缺的是 **dup 值作为无名表达式值时对其后续消费方（aastore 值位 / arrayload 接收者）的建模**。

## What Changes

把初始化 dance 值的合格消费方从"具名存储目标"（局部/字段）与"封闭参数位"扩到**任意表达式消费位**：当 dance（newarray+dup+内层存储序列）是唯一写者且 dup 引用恰好有一个后续消费方时，呈现 `new T[]{…}` 于该消费位。判据与丢弃分配片的三分支（读者计数）同族：这里读者恰为 1（该消费方），是既有"单读者"不变量的**新值形**（dance 产生），非放宽。

## Impact

- **代码**：`crates/jarde-java/src/build.rs` 数组初始化呈现区（"copy … has no proved local assignment" 对 dup 的发出处，task 1.1 插桩定位）。
- **测试**：MD/MD2/MD3 fixture（六位判别已冻结）+ 四合格位零回退。
- **账本**：summary.md 登记行关闭。

## Non-Goals

- **不**做数组推导或简化（`{7}` 简写不恢复——呈现保持 `new int[]{7}` 显式形）；
- **不**动四位合格位的既有呈现（零回退锚）；
- **不**处理多消费方（dance 值被多处读——保持拒绝）。
