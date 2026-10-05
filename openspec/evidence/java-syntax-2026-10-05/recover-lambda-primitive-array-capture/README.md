
## Root 验收后记（2026-10-05）：P02_multianewarray = critical 第 19 锚（依赖链族 lambda 体位点）

int[][] 控制腿的渲染幸存文本（baseline↔fixed 逐字节相同，先前就有）经 javac 编译 **exit 0**，运行打印 **`0` vs 原 class `6`**——lambda companion 体内 `arg0[0][0] += arg1.intValue()`（二维下标复合赋值）整条被引注吞掉，宿主幸存 `sum` 数组保持初值——**compilable-wrong（第一不变量违反）**。Root 独立复现（jar 输入渲染→剥 JSON 尾→编译→运行）。属"lambda 体内数组元素复合赋值"既有域与 critical 依赖链族的交汇位点；DT-26 已把该腿冻结为控制 fixture（`v8/v23 P02_multianewarray.class`）。
