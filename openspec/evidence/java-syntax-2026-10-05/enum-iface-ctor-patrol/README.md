# 枚举实现接口与构造器字段巡查（2026-10-05 root，负结果）

## 探针

[fixture/ei.jar](fixture/ei.jar)（`--release 8`，8 类文件）：枚举**实现接口且常量各自带体**（`Basic implements Op`，PLUS/MINUS 各自实现 `apply`）、枚举**构造器 + final 字段 + 常量实参**（`BIG(100)/SMALL(1)`）、嵌套接口。

## 结果：**健康，无缺口**（宿主+全部伴生 quotes=0）

- **枚举实现接口带常量体**：`enum EI$Basic implements EI$Op` + `PLUS { public int apply(int arg1) { return arg1 + 1; } }` 常量体**各自完整**（匿名子类 `EI$Basic$1/2` 被内联为常量定义——"selected enum child definition" 选择链如实）；
- **带构造器枚举**：常量实参（`BIG(100)`）、`final int scale` 字段、`private EI$WithCtor(int arg0) { this.scale = arg0; }` 构造器、`apply` 体全部正确；
- 枚举类 Signature（`Ljava/lang/Enum<LEI$Basic;>;LEI$Op;`）拒绝注释 = 既有报告质量债（枚举 Signature 标记），非能力缺口；
- 拼接四类型 `javac` exit 0、`java -Xverify:all` 输出 `6/4/300/3` 与原 class **逐行一致**。

## 处置

负结果归档，不立 spec。枚举复杂形（接口实现+常量体、构造器+final 字段+实参）确认覆盖。
