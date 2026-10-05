# 小 N switch 巡查（2026-10-05 root，负结果——switch 族收尾确认）

## 探针

[fixture/SS.java](fixture/SS.java)（`--release 8`）：1-case String switch、2-case String、1-case int、稀疏 int（1/1000——lookupswitch）。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- javap 确认：**javac 对 1-case String switch 仍发 lookupswitch**（hashCode 单桶 + equals），无 if-else 退化形——jarde 统一折回 switch（单 case 形完整）；2-case/稀疏同；
- 行为 `1/?/2/10/20` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。switch 族至此**全覆盖确认**（int/String/enum/装箱拆箱/贯穿/空叠加/碰撞/default 位置/非局部出口（已立项）/小 N）。
