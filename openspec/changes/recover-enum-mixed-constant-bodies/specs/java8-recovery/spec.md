## ADDED Requirements

### Requirement: 构造实参与常量专属体的组合折叠

系统 SHALL 在枚举常量同时带构造实参与专属匿名体（`NAME(<args>) { <body> }`）时，按参数化后的常量体义务（子类/桥 ctor 接受既有实参 grammar、委托链逐参转发证明）折叠该常量：实参与体各自复用既有拼写。纯带体、纯带参能力 SHALL 逐字不变；义务不满足（转发链断、实参种类不符、体方法非 structured）SHALL 保持整组逐字段呈现。

#### Scenario: 混合常量折叠

- **WHEN** `ADD(1) { @Override … }, MUL(2) { … }, ID(0);` 家族三方 Java 8 重编运行
- **THEN** 三常量按源形态折叠，`java -Xverify:all` 逐路径与原 class 一致

#### Scenario: 纯能力与负例不变

- **WHEN** 输入为纯带体、纯带参枚举，或义务不满足的混合形状
- **THEN** 前者与本变更前逐字一致；后者保持逐字段呈现与既有诊断
