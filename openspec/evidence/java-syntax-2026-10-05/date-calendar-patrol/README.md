# java.util.Date/Calendar/SimpleDateFormat 旧日期 API 巡查（2026-10-05 root，负结果）

## 探针

[fixture/DT.java](fixture/DT.java)（`--release 8`）：Calendar 字段读写（`set/add/set` 链）+ SimpleDateFormat 格式化/解析（throws ParseException）+ Date.after 比较（嵌套 parse 链+显式 (Date) cast）+ getTimeInMillis 日差惯用法。

## 结果：**健康，无缺口**（quotes=0）

- 四方法全恢复：fmt（set/add 链+格式化返回）、parse（getInstance+setTime+返回）、after（嵌套调用链+cast）、daysBetween（long 减除）；
- **Calendar 常量按 javac 编译期内联呈现为字面量**（`Calendar.DAY_OF_MONTH` → `5`、`MONTH` → `2`、`HOUR_OF_DAY` → `11`）——静态 final int 常量的忠实呈现域（与常量族结论一致；可读性=报告质量域，非正确性）；
- 行为 `2026-10-01 00:30/29/true` 逐行 IDENTICAL（渲染原样编译运行）。

## 处置

负结果归档，不立 spec。旧日期 API 族（Calendar/SimpleDateFormat/Date 比较）确认覆盖。
