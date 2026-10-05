# 装箱 switch 与枚举合成方法巡查（2026-10-05 root，负结果）

## 探针

[fixture/BS.java](fixture/BS.java)（`--release 8`）：**Integer/Character switch**（javac 自动拆箱后 tableswitch）、**values()/valueOf() 合成方法**消费、`name()`/`ordinal()`。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- **装箱 switch 显式拆箱呈现**：`switch (arg0.intValue())`/`switch (arg0.charValue())`——字节码物理事实（javac 发 `intValue` 后 tableswitch）如实呈现（呈现拆箱调用而非隐藏——忠实且可编译）；
- **枚举合成方法消费**：`BS$Color.values()`（for-each 数组源）、`BS$Color.valueOf(arg0)`、`name()+":"+ordinal()` 全部恢复；
- 拼接宿主+枚举伴生 `javac` exit 0、行为 `one/many/2/2/GREEN/RED:0` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。装箱 switch（拆箱呈现形）与枚举合成方法族确认覆盖。
