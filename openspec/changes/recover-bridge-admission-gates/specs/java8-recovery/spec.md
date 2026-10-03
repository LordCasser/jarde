## ADDED Requirements

### Requirement: 接口擦除返回与参数 cast 形的编译器桥可投影隐藏

系统 SHALL 在编译器桥满足既有全部准入条件（ACC_BRIDGE 声明事实、同次 `bridge@1` 纯转发证明、同类唯一源级目标、继承需求、效果不纯即拒）且额外满足以下之一时将其投影隐藏：源返回类型是桥擦除返回类型的已证快照层级子类型；或桥体为"参数按位 cast 到源级参数类型 + 转发"的规范擦除形。真实源级覆写成员 SHALL 保留呈现。源返回与桥返回无层级关系、cast 目标不等于源级参数类型、效果不纯或既有负例形状 SHALL 保持既有拒绝。

#### Scenario: 协变接口覆写类可重编

- **WHEN** `class Base implements Node { public Base next() { … } }`（桥擦除返回为接口类型）三方 Java 8 重编
- **THEN** 桥被隐藏、仅源级 `Base next()` 呈现，`javac --release 8` 通过且 `java -Xverify:all` 运行与原 class 一致

#### Scenario: 泛型特化类可重编

- **WHEN** `StrBox extends Box<String>` 与 `Impl implements Comparable<Impl>`（桥体含参数 cast）三方重编
- **THEN** 两桥均被隐藏、源级成员保留，运行与原 class 一致（`s`/`0`）

#### Scenario: 既有边界与负例不变

- **WHEN** 输入为既有桥投影正例、`negative/`/`orphan/` 负例、无层级关系的源返回或 cast 目标不匹配
- **THEN** 输出与本变更前逐字一致（保持拒绝）
