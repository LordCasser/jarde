# 菱形 default 冲突巡查（2026-10-05 root，负结果）

## 探针

[fixture/DM.java](fixture/DM.java)（`--release 8`）：**菱形冲突显式解析**（`Both implements A, B` 双 default 冲突，`A.super.n()` 显式选择）、单继承 default（隐式继承）、二级继承 `super.n()`、**接口再抽象化**（`C extends A` override default）、接口内 default 方法体。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- **`A.super.n()`**——菱形冲突的显式解析调用**完整恢复**（这是 default 冲突场景的关键难点：`invokespecial` 到接口方法必须呈现为 `Iface.super.m()` 形，否则渲染源不编译）；
- 接口 default 体（`public default java.lang.String n()`——含正确修饰符）、`super.n()` 二级继承、隐式继承（Simple/ViaC 无 override 如实无方法）全部正确；
- 嵌套 interfaces 内联宿主呈现（既有池形域）；
- 行为：渲染源集 `javac` exit 0、`java -Xverify:all` 输出 `A:A/A/S+A/C` 与原 class **逐行一致**（菱形解析运行正确）。

## 处置

负结果归档，不立 spec。菱形 default 族（冲突解析/多级继承/接口再抽象化）确认覆盖。
