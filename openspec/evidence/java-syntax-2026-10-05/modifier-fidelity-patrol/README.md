# 字段/方法修饰符巡查（2026-10-05 root，负结果）

## 探针

[fixture/MF.java](fixture/MF.java)（`--release 8`）：`transient`/`volatile`/`static final transient` 字段组合、`native`（无 Code）、`strictfp`、`synchronized`（static 与 private static 组合）、`final` 方法。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- **字段修饰符全恢复**：`transient int t`、`volatile int v`、`static final transient int SF = 3`（ACC 标志逐位映射，组合序正确）；
- **`native`**：无 Code 属性如实呈现声明 + "declared native … no Code attribute" 诚实注记（与 abstract 同通道）；
- **`strictfp`**：方法级标志恢复；体内 `x * 2.0` 呈现为 `arg1 * 0x1.0000000000000p1d`——**十六进制浮点位级精确**（2.0 的 double 位形）；
- **`synchronized` 方法**（ACC_SYNCHRONIZED，与语句级 monitorexit 不同通道）：`static synchronized`/`private static synchronized` 组合全对；`final` 方法恢复；
- 行为 `1/2/3/4.0/4/5` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。修饰符族（transient/volatile/native/strictfp/synchronized/final 及组合）确认覆盖。
