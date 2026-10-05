# Map 遍历三形巡查（2026-10-05 root，负结果——最常见集合迭代惯用法全过）

## 探针

[fixture/MP.java](fixture/MP.java)（`--release 8`）：**entrySet 遍历**（`Map.Entry<K,V>` 泛型 for-each——真实代码最高频形）、keySet+get、values、raw entrySet（无泛型形）。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- **entrySet 形**：`Iterator` 显式化（hasWhile 链）+ `Map.Entry` cast + getKey/getValue 的擦除 cast 链（`((String) local2.getKey()).length() + ((Integer) local2.getValue()).intValue()`）——物理事实逐层如实；泛型与 raw 两形呈现**一致**（raw 源的 for-each 泛型语法糖编译后同形）；
- **keySet+get** 形（`m.get((Object) k)` 实参 cast + `(Integer)` 拆箱）与 **values** 形恢复；
- clinit 的 `put((Object) "a", (Object) valueOf(1))` 双 cast 实参如实；
- 行为 `5/5/3/5` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。Map 迭代族（entrySet/keySet/values/raw）确认覆盖——真实业务代码（HashMap 遍历）核心形态健康。
