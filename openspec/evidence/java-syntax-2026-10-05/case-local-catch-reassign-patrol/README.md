# case 块局部与 catch 参数重赋巡查（2026-10-05 root，负结果）

## 探针

[fixture/CL.java](fixture/CL.java)（`--release 8`）：switch case 块内局部声明（各 case 同名独立作用域 `t`）、case 内声明+贯穿（合法源形）、catch 参数重新赋值（`e = new ...("wrapped:" + e.getMessage())`）、双参自定义异常 ctor super 拼接链。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- **case 块局部**：javac 为各 case 块的 `t` 复用**同一 slot**（物理事实）——jarde 呈现为"switch 前单声明 + 分支内赋值"（`int local1;` + 每 case `local1 = ...`）——**等价规范化**（合法且行为同）；
- **case 内声明+贯穿**：`case 1: int local1 = 5; case 2:` 直接呈现（源级合法形——声明在 case 直接体内合法，后续 case 不读即可）；
- **catch 参数重赋值**：逐字（`local1 = new IllegalStateException("wrapped:" + local1.getMessage())`）；
- 双参异常 ctor（`super(m + "#" + code)` 拼接实参）恢复；
- 行为 `11/42/wrapped:nil/22/22/bad#7` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。switch 局部/catch 参数域确认覆盖。
