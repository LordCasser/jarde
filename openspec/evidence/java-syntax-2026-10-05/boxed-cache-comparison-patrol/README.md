# 装箱缓存边界比较巡查（2026-10-05 root，负结果——反编译器经典保真陷阱全过）

## 探针

[fixture/IC.java](fixture/IC.java)（`--release 8`）：`Integer == Integer` 在缓存内（127）/外（200）、`Integer == 字面量`（一边拆箱）、Long/Double 同形、equals/intValue 对照——`==` 语义分界（引用比较 vs 值比较）是反编译器最经典的保真陷阱（错误重排会翻转真假）。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）——语义分界精确

- **装箱 vs 装箱**（`local0 == local1`，两个 Integer）：渲染源保留**引用比较形**（重编译后 `if_acmpne` 同语义——127 缓存内真、200 缓存外假的分野精确保真）；
- **装箱 vs 字面量**：呈现 `local0.intValue() == 200`——**显式拆箱调用**（字节码里的 `invokevirtual intValue` 如实呈现），编译回同指令；
- **Long/Double 同形**（Double 恒假——无缓存）保真；equals（`(Object)` cast 实参）与 intValue 对照恢复；
- 行为 `true/false/true/false/false/true` 逐行 IDENTICAL——**缓存边界分野（127真/200假）精确复现**，这是 == 语义未被破坏的最强证据。

## 处置

负结果归档，不立 spec。装箱比较族（缓存内/外/拆箱边/Long/Double/equals）确认覆盖。
