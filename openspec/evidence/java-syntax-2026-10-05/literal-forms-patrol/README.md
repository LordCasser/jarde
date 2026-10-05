# 数值字面量形式族巡查（2026-10-05 root，负结果）

## 探针

[fixture/NL.java](fixture/NL.java)（`--release 8`）：hex long（`0xCAFEBABEL`）、二进制+下划线（`0b1010_1010`）、下划线十进制 long（`1_000_000_000L`）、hex float（`0x1.91eb851eb851fp+1` ≈ π）、十六进制 char（`0x4E2D`=中）、char 算术实参（`'A'`）、`toBinaryString` 消费。

## 结果：**健康，无缺口**（quotes=0）

- 常量池只存值，渲染为**合法且值精确**的 Java：hexLong→`3405691582L`、binLit→`170`、under→`1000000000L`（十进制化=语义恒等呈现选择）；
- **hex float 位级保留**：`0x1.91eb851eb851fp1d`（池常量位级直呈——#35 strictfp 结论再确认）；
- **char 十六进制→unicode 转义**：`'\u4e2d'`（合法 Java 字面量，值精确）；
- `'A'` char 字面量消费位保留；`bits(255L)` 十进制化；
- 行为往返 `3405691582/170/1000000000/3.14/中/131/11111111` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。字面量形式域（hex/bin/underscore/hex-float/hex-char）确认覆盖。
