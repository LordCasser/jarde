# 接口静态常量继承链巡查（2026-10-05 root，负结果）

## 探针

[fixture/IA.java](fixture/IA.java)（`--release 8`）：三接口链 `IA←IB(extends)←IC(implements)` 跨两跳继承常量（`BASE+MID`）、接口内嵌套接口常量持有者（`IA.Inner.DEEP`）、限定直引（`IA.BASE`）、编译期常量折叠链（`DEEP*2`）。

## 结果：**健康，无缺口**（IC/IB quotes=0）

- 全部按 **javac 编译期常量折叠内联为字面量**（`return 3/10/1`、`LOCAL_CONST = 20`）——常量族既有忠实呈现域（接口常量恒 static final = ConstantValue 可折叠，与字段族结论一致）；
- IB 的 `extends IA` 接口继承头正确；嵌套接口持有者 `IA.Inner` 拼写在池形路径验证（IC 消费位已折叠）；
- 行为 `3/10/1/20` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。接口常量域（继承链/嵌套持有者/折叠链）确认覆盖。
