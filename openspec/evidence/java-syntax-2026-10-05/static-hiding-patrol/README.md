# 静态方法隐藏与接口静态巡查（2026-10-05 root，负结果）

## 探针

[fixture/sm.jar](fixture/sm.jar)（`--release 8`）：子类**隐藏**父类静态方法（`Kid.who()`）、继承静态经子类限定调用（`Kid.shared()`）、接口静态方法被 default 消费（`helper()`）。

## 结果：**健康，无缺口**（宿主+4 伴生 quotes=0）

- **隐藏/继承声明区分正确**：`SM$Kid extends SM$Base` 自带 `who()`（隐藏）、**无** `shared()` 声明（继承）——物理事实如实；
- **源限定者逐字保留**：javap 核实 javac 8 的 invokestatic owner **记录源限定者**（`SM$Kid.shared`——非解析 owner Base）；jarde 呈现 `SM$Kid.shared()` 逐字忠实（root 初判"可能是重限定"被 javap 纠正——字节码就是 Kid）；
- **接口静态方法**：`public static helper()` 声明 + default 体内 `helper()` 调用（编译为 `invokestatic SM$IF.helper`——接口静态不继承）全恢复；
- 拼接五类型 `javac` exit 0、行为 `base/base-shared/kid/base-shared/if-static-d` 逐行 IDENTICAL（隐藏派发精确：Kid.who=kid、Kid.shared=base-shared）。

## 处置

负结果归档，不立 spec。静态隐藏族（隐藏/继承/源限定/接口静态+default 消费）确认覆盖。
