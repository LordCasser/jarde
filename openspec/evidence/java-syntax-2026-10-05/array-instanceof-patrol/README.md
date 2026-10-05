# 数组类型 instanceof + 数组 cast 巡查（2026-10-05 root，负结果）

## 探针

[fixture/AR.java](fixture/AR.java)（`--release 8`）：原始数组 `instanceof int[]` + cast 消费 `.length`、引用数组 `instanceof String[]` + cast 元素取值、数组/类混合四臂 instanceof 链、数组协变上转型（`Object f(int[])` 返参）。

## 结果：**健康，无缺口**（quotes=0）

- 数组 instanceof 谓词与类 instanceof 同机制（instanceof-chain 结论外推到数组维度成立）；
- `((int[]) o).length` / `((String[]) o)[0]` 数组 cast 消费位完整恢复（checkcast 到数组描述符如实）；
- 协变上转型按隐式 widening 呈现（`return arg0;`——无操作转换不生成显式 cast，语义等价）；
- 行为 `3/-1/x/Object[]/other/true` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。instanceof/cast 域数组维度补格。
