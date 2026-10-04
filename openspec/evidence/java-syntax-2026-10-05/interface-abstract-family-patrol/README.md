# 接口继承与抽象族巡查（2026-10-05 root，负结果）

## 探针

[fixture/AB.java](fixture/AB.java)（`--release 8`）：接口继承（`B extends A`）、抽象类部分实现（`Part implements B` 实现 `a()` 留 `b()` 抽象）、具体类补全（`Full extends Part`）、接口常量（隐式 `public static final` 三型）、消费方经接口链调用。

## 结果：**健康，无缺口**（宿主 + 全部 5 伴生 quotes=0）

- **接口继承**：`interface AB$B extends AB$A` 如实；方法 `public abstract int b()` 呈现；
- **抽象类部分实现**：`abstract class AB$Part … implements AB$B` + 已实现 `a()` + `public abstract int b()`——**正确区分**已实现/抽象成员；
- **接口常量**：`AB$Consts` 三个常量带完整 `public static final` 修饰与初始化值（`int X=10`、`long L=20L`、`String S="cs"`——ConstantValue 还原）；
- **常量消费方**：`useConsts()` 中 `Consts.X`/`Consts.L`/`Consts.S` 被 javac 内联为常量（`30 + "cs".length()`）——**忠实呈现内联结果**（源写法无法从字节码恢复，等价）；
- 拼接六类型 → `javac` exit 0 → `java -Xverify:all` 输出 `3/32` 与原 class **逐行一致**。

## 处置

负结果归档，不立 spec。接口继承/抽象部分实现/接口常量族确认覆盖。
