## Why

[异构数组初始化巡查](../../evidence/java-syntax-2026-10-05/heterogeneous-array-init-patrol/README.md)实证：`Arrays.asList(1, 2L)`（泛型上界推断形）生成 `anewarray java/lang/Number` + 元素 0 为 `Integer.valueOf(1)`、元素 1 为 `Long.valueOf(2L)`——**元素静态呈现类型互异**（各自是组件 `Number` 的子类）——呈现层要求元素呈现类型与数组组件一致而整方法拒绝："the array initializer element at BCI 23 is presented as `java.lang.Integer`, while the array component …"。响亮拒绝（行为安全）但整方法损失。

**判别（已实测）**：同构装箱数组（`Arrays.asList("a","b")`，String 元素）**恢复**——缺口仅在元素类型互异且各需上转型到组件类型的初始化器。**jadx 有解**（`Arrays.asList(1, 2L).get(0)` + cast 呈现）。

## What Changes

数组初始化器的元素类型判据放宽为**赋值兼容**：元素呈现类型是组件类型的子类（向上赋值合法）即接受，呈现为按元素自身类型的初始化（`new Number[]{ Integer.valueOf(1), Long.valueOf(2L) }` 形或既有等价呈现）。不新增类型系统——赋值方向（子→父）是 Java 语言既有事实，比"类型一致"判据更准确而非更宽。

## Impact

- **代码**：呈现层数组初始化器的元素-组件一致性检查（拒绝文本 "array initializer element … while the array component …" 的发出处，实现者 task 1.1 定位；root 未预定位）。
- **测试**：`CT.cov` fixture（巡查已冻结）+ 同构数组零回退 + `up()`（List 上转型）零回退。
- **账本**：summary.md 异构数组登记行关闭。

## Non-Goals

- **不**做元素间 LUB 计算或泛型推断重建（组件类型来自字节码 `anewarray`，已由 reader 事实给出）；
- **不**改同构数组呈现（零回退锚）；
- **不**触碰协变返回/通配符读取域（`cov` 的返回 cast 已恢复）。
