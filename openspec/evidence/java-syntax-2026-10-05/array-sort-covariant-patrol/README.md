# 数组排序/协变/clone 巡查（2026-10-05 root，负结果）

## 探针

[fixture/AS.java](fixture/AS.java)（`--release 8`）：**数组 `clone()` + `Arrays.sort`**（去重排序惯用法）、手写 max 循环、原地 sort 引用型（泛型 `sort(T[])` 擦除 → javac 发 `(Object[])` checkcast）、**数组协变赋值 + cast 往返**（`Object o = new Integer[3]; Integer[] back = (Integer[]) o`）、手动装箱循环。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- `(int[]) xs.clone()` ——clone 调用+协变返回 cast 如实；`Arrays.sort` void 调用作语句；
- 手写 max 完整（数组元素条件累积）；泛型 sort 的 `(Object[])` checkcast 如实呈现；
- **协变形**：`Object` 中转变量被消除（`Integer[] local0 = new Integer[3]; Integer[] local1 = (Integer[]) local0;`）——中转直投后 cast 成无操作，**语义等价**（aload 局部直达物理事实）；手动装箱循环恢复；
- 行为 `[1, 2, 3]/9/[a, b]/7/[1, 2]` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。数组惯用法（clone+sort/原地 sort/协变 cast/装箱）确认覆盖。
