# EM-17/18 数组巡查：动态维度与计算填充首片（2026-10-01）

[EM-17/EM-18 账本](../../jadx-feature-inventory-2026-09-27/expressions-misc.md)登记"动态维度副作用与更多位置仍待测"的 JVM 侧扩验（主线 `fdedd36f`）。固定转录 [fixture](fixture/)（A1/A2，SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)）。

## 结果矩阵

| 场景 | 主线 Jarde |
| --- | --- |
| A1.dynDims：`new int[n()][]` + `new int[n()]` + 嵌套初始化器含 `new int[n()]`（维度副作用计数） | 完整恢复，行为与原 class 逐字一致（`2:3:3`/`3:5`） |
| A1.mixed：`Object[]{String, int, int[]}` 混合初始化器 | 同上 |
| A2.fillCalc：计算元素填充 `{side(), side()+1, side()*2}` + 增强 for + `boolean[]` 写入 | 恢复**不可编译**（见下） |

## 发现：数组槽复用跨元素类型的声明归属缺陷

A2 字节码里 `int[] a`（`newarray int` + `astore_2`…实际 `anewarray`/`newarray` 先后）与 `boolean[] f`（`newarray boolean` + 同槽 `astore_2`）复用槽 2。主线输出把槽声明拼为首个定义的类型：

```java
int[] local2;
int[] local0 = new int[]{...}; local2 = local0;   // 拷贝别名呈现
...
local2 = new boolean[3];                            // 不可编译：int[] 之下的 boolean[] 赋值
local2[side() - 1] = true;
```

根因与 [catch-param-slot-reuse](../catch-param-slot-reuse/README.md) 同族——**槽的类型决策被首个定义赢得，后续不同类型定义的赋值沿用槽声明**。正确呈现需要按值归属分声明（两次定义在源码上是两个作用域局部，或以重新声明/新名呈现）。附带的 `local0 = …; local2 = local0` 拷贝别名是呈现质量问题（正确但冗余），一并登记。

## 处置

`recover-array-slot-retype-locals`（spec 随后）：同一局部槽被多个**不同元素类型**的数组定义先后写入且各自读取段不交叠时，按定义分段呈现（每段自有声明）；类型相同或单一定义不受影响。串行排在 `recover-caught-value-argument-typing` 之后（同触 build.rs 呈现层）。

原 class 为行为基准；JADX 参照不作为语义正例。
