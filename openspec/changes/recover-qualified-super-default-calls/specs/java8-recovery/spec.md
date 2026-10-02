## ADDED Requirements

### Requirement: 限定 super 默认方法调用可呈现

系统 SHALL 在 invokespecial InterfaceMethod 的限定符接口为调用类快照 header 的直接超接口、且目标方法在该接口成员表为非 abstract 非 static 实例方法时，按 `X.super.m(args)` 呈现该调用；菱形多限定各自独立证明。普通 super/this 调用、静态接口方法与抽象限定 SHALL 保持既有行为；限定符非直接接口、目标 abstract 或接口不在快照内 SHALL 保持既有拒绝。

#### Scenario: 菱形调解恢复

- **WHEN** `A.super.name() + B.super.name()`（Diamond implements A, B）家族三方 Java 8 重编运行
- **THEN** 限定调用完整呈现，`java -Xverify:all` 逐路径与原 class 一致

#### Scenario: 直接性与负例不变

- **WHEN** 限定符经 extends 间接继承、目标为 abstract，或普通 super/this 调用
- **THEN** 前两者保持既有拒绝；后者与本变更前逐字一致
