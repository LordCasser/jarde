# long/double 循环计数器与宽型比较巡查（2026-10-05 root，负结果）

## 探针

[fixture/LC.java](fixture/LC.java)（`--release 8`）：`for(long i…)` long 循环计数器（时间戳/ID 生成循环骨干）、lcmp 三向（嵌套三元 long 比较）、循环内 double 累积（`1.0/(i+1)` 提升除法）、Math.toIntExact 风格显式收窄守卫（MAX/MIN 边界 if-else + `(int) v`）。

## 结果：**健康，无缺口**（quotes=0）

- **long 循环计数器完整恢复**：`local4 = 1L; while(local4 <= arg0){ …; local4 = local4 + 1L; }`——for→while 归一 + `L` 字面量后缀 + lcmp 谓词如常；
- lcmp 三向：嵌套三元 `a < b ? x : a > b ? -x : 0`（右结合嵌套，long 比较三元位恢复）；
- **double 累积**：`0x0.0p-1022d`/`0x1.0p0d` 十六进制浮点字面量（池常量位级精确——#35 strictfp 结论一致）+ `(double)(local3+1)` 提升 cast；
- **收窄守卫惯用法**：MAX/MIN 边界 if-else 链 + `(int) arg0` 显式收窄 cast——Math.toIntExact 手写形完整；
- 行为 `5050/-2/2/0/2.083333333333333/2147483647/42` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。64 位循环/宽型比较域确认覆盖。
