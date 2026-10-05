# 私有方法 access$ 合成族巡查（2026-10-05 root，负结果）

## 探针

[fixture/PA.java](fixture/PA.java)（`--release 8`）：嵌套类调用外类**私有实例方法**（经参数实例 + 经 `PA.this` 双形）与**私有静态方法**——javac 分别合成 `access$000(PA, I)I`（实例转发）与 `access$100(I)I`（静态转发）。

## 结果：**健康，无缺口**（宿主+全部伴生 quotes=0）

- **私有实例方法 access$**（`access$000(PA arg0, int arg1) → arg0.secret(arg1)`）：静态嵌套（`PA.access$000(arg0, arg1)`）与成员内部类（`PA.access$000(PA.this, arg1 + 1)`——实参窗口含 `PA.this` 表达式）双形全部恢复——**方法转发访问器与 EM-15 的字段写访问器不同形**（转发是普通方法调用呈现，非 RMW 赋值）；
- **私有静态方法 access$**（`access$100(int) → ssecret(arg0)`）恢复；
- 拼接三类型 `javac` exit 0、行为 `12/15/17` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。access$ 族至此全覆盖：字段读（既有健康）/字段写（EM-15 在飞）/**方法实例转发**（本巡查健康）/**方法静态转发**（健康）/ctor 访问器（已登记独立缺口）。
