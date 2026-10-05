# format/解析惯用法巡查（2026-10-05 root，负结果——字符串处理全家桶全过）

## 探针

[fixture/FM.java](fixture/FM.java)（`--release 8`）：`String.format("%04d-%s", n, s)`（varargs+装箱数组）、**SB 变异链**（append/insert/replace/deleteCharAt/reverse/setLength——void/链式混合调用作语句）、`Integer.parseInt(s.trim()) + catch NumberFormatException`（解析惯用法）、`charAt` 循环 + `Character.isDigit`、substring/valueOf/case 链。

## 结果：**健康，无缺口**（quotes=1 / notrec=0，行为全同）

- `String.format` → `new Object[]{valueOf(n), s}` varargs 数组内联+装箱精确；
- SB 变异链逐语句还原（含 void 调用 `reverse(); setLength(4);` 作语句、链式 insert/replace/deleteCharAt）；
- parseInt+trim+catch（`(String) arg0.trim()` cast 如实）、charAt 循环、substring/case 链全恢复；
- 唯一引注：`s.valueOf(42)` 的 `aload_0;pop` 死指令对（实例限定静态调用的求值副产物——**已登记呈现质量数据点**，非新缺口；重限定 `java.lang.String.valueOf` 行为等价）。

## 处置

负结果归档，不立 spec。字符串处理族（format/SB 变异链/parse/charAt/valueOf）确认覆盖。
