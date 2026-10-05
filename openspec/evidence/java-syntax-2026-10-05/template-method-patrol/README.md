# 抽象类模板方法巡查（2026-10-05 root，负结果——框架核心模式全过）

## 探针

[fixture/TM.java](fixture/TM.java)（`--release 8`）：抽象类骨架（`abstract static class Task`——private final 字段/protected ctor/abstract 钩子/protected 默认钩子/**final 模板方法**）、两个具体子类（一个覆写 step、一个覆写 step+hook）、main 虚分派驱动。

## 结果：**健康，无缺口**（宿主与三伴生 quotes=0 / notrec=0）

- **宿主嵌套呈现**：`static abstract class Task` 完整（`abstract void step();` 声明、final 模板方法体 `this.step(); this.hook();` 虚分派序列、protected hook 默认实现、`this.label()` 调用）；子类 `extends Task` + `super(arg1)` 委派 + 覆写方法体；main 裸 `new Fast("F")`（嵌套内直呼）；
- **伴生单类渲染**：`TM$Fast extends TM$Task`（池名限定）、抽象声明/覆写同构；
- 行为八行 `begin:F/fast-step/default-hook/end/begin:S/slow-step/slow-hook:S/end` 逐行 IDENTICAL（虚分派语义精确——钩子可选覆写各自触发）。

## 处置

负结果归档，不立 spec。模板方法族（abstract 声明/final 模板/protected 钩子/子类覆写/虚分派）确认覆盖。
