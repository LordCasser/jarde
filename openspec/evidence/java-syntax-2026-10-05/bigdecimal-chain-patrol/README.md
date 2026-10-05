# java.math 不可变链巡查（2026-10-05 root，负结果——金额计算骨干全过）

## 探针

[fixture/BD.java](fixture/BD.java)（`--release 8`）：**BigDecimal 金额链**（`valueOf(units).multiply(...)` → `base.subtract(base.multiply(rate)).setScale(2, HALF_UP)`——**base 双消费**（第 4 族 2-消费者边界）+ 字符串 ctor + 枚举舍入实参）、BigInteger 阶乘累积（不可变乘循环重赋）、compareTo 链（else-if 双调用——非 equals 陷阱域）、divide 精度+舍入三实参。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- 金额链：`local2.subtract((BigDecimal) local2.multiply(arg1)).setScale(...)`——**base 双消费正确内联**（2-消费者边界仍恢复，与 #90 NI 的 ≥3 阈值判别吻合）；`valueOf((long) arg0)` 装箱提升 cast、`new BigDecimal("9.99")` 字符串 ctor、`RoundingMode.HALF_UP` 静态枚举字段如实；
- BigInteger 阶乘：`ONE` 静态字段+循环重赋值 `r = r.multiply(valueOf(i))`（不可变累积=局部重赋值，非复合 RMW——不触发实例复合失败面）；compareTo 链 else-if 双调用精确；divide(3实参) 直呈；
- 行为 `26.97/2432902008176640000/0/3.3333` 逐行 IDENTICAL（compareTo 语义精确：1.0 vs 1.00 → 0——equals 会是 false，此处保真）。

## 处置

负结果归档，不立 spec。java.math 族（金额链/阶乘/比较链/精度除法）确认覆盖。
