# 重载解析歧义三级巡查（2026-10-05 root，负结果——与 be2cf7fd 重载保真巡查互补）

## 探针

[fixture/OL.java](fixture/OL.java)（`--release 8`）：同一名字四级重载（int 精确/Integer 装箱/Object 宽化/int... varargs）、**null 调用点的最特化**（String vs Object——javac 选 String）、**宽化优先于装箱**（int→long 优先于 Integer）、源级显式 cast 调用点。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- 调用点呈现保留 javac 记录的描述符：`pick(5)` 直呈（I 精确）、`pick((Integer) valueOf(5))`（装箱+cast 如实）、`nullRef((String) null)`（null 特化选择以 cast 物理呈现——checkcast 即 javac 的选择记录）；
- 行为 `int/boxed/object`、`string/int` 逐行 IDENTICAL——三级解析（精确>装箱>varargs）、null 特化、宽化优先全部经重编译复现；
- 附注：`nullRef(String)` 与 `nullRef(Integer)` 并存时源级本身二义（编译失败）——不构成场景。

## 处置

负结果归档，不立 spec。重载歧义域（与 be2cf7fd 的重解析证明法互补——彼证明选择零漂移，此证明歧义边界三级保真）。
