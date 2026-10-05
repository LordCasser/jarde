# 数组↔List 桥/不可变包装巡查（2026-10-05 root，负结果）

## 探针

[fixture/AL.java](fixture/AL.java)（`--release 8`）：`Arrays.asList(T[])`（泛型数组→List）、`toArray(new T[0])`（空数组惯用法）、`Collections.unmodifiableList` 包装、空守卫三元 `isEmpty() ? dflt : get(0)`。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- `Arrays.asList((Object[]) arg0)`——泛型方法擦除的 `(Object[])` cast 如实；`toArray((Object[]) new String[0])`——空数组实参的双重 cast（擦除+协变）物理呈现；
- `Collections.unmodifiableList(arg0)` 直传；空守卫三元恢复（`(String) arg0.get(0)` cast 如实）；
- main 局部 List 双消费正常（2-消费者边界）；
- 行为 `[x, y]/[x, y]/2/E/x` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。集合桥接族（asList/toArray/unmodifiable/空守卫）确认覆盖。
