# 枚举家族前沿巡查（2026-10-05 root，负结果）

## 探针

[fixture/EN.java](fixture/EN.java)（`javac --release 8`）：简单枚举、**带体枚举**（常量级匿名子类覆写 + 默认方法体）、**枚举实现接口**、**带构造器与字段的枚举**（`BIG(10)` 实参）四形。DT-13 此前只验收过"两层普通嵌套 enum"与"enum implements I"两个窄子形态；本巡查把家族面铺满。

## 结果：**健康，无缺口**

每个枚举类型单独渲染（`--class 'EN$X'`，硬自述头断言通过）**quotes=0 / notrec=0**：

- `EN$Plain`：`A, B, C;` 常量表；
- `EN$WithBody`：常量级匿名子类体（`X { void extra(){…} }`，含 "selected enum child definition" 选子注释）+ 默认 `extra()` 体——**带体形完整恢复**；
- `EN$Impl`：`implements java.lang.Runnable` + `run()` 体（`"run:" + this.name()` 拼接如实）；
- `EN$WithCtor`：`BIG(10), SMALL(1);` **实参传递** + `final int size` + private 构造器。

宿主 `EN`（家族渲染）quotes=0；`main` 中 `EN$WithBody.X.extra()`、`EN$Impl.R1.run()`、`WithCtor.BIG.size`、`values()[1]` 全部如实呈现。

**拼接编译 + 行为验证**：五类型渲染拼接为单文件 → `javac` **exit 0** → `java -Xverify:all` 输出 `A/3/2`、`X-extra`、`run:R1`、`10/SMALL` 与原 class **逐行一致**（[results/](results/)）。

已知呈现注记（非缺口）：每个枚举带 `class Signature Ljava/lang/Enum<…>; not projected` 注释——javac 对枚举恒发该合成 Signature，其投影被 `class_generic_source_unproved` 拒绝是**既有登记的枚举 Signature 标记债**（dt13 的 enum-signature-marker-debt.md，纯报告质量、不影响编译运行——本探针拼接编译 exit 0 佐证）。

## 处置

负结果归档，不立 spec。枚举家族前沿（简单/带体/实现接口/带构造器实参四形）确认覆盖；`values()`/`name()` 等合成成员未在渲染中出现（按既有域处理）。
