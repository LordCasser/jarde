## ADDED Requirements

### Requirement: 混合族中的直接静态成员类按嵌套声明折叠呈现

系统 SHALL 在外围类的直接成员家族同时含非静态成员时，仍将其中满足静态成员判据的子集按嵌套声明折叠呈现（域内引用源码拼写）；非静态成员保持既有分离路径。纯静态族输出与非静态成员分离呈现 SHALL 逐字不变。

#### Scenario: 混合族静态子折叠

- **WHEN** N1 形（静态 Stat + 非静态 Inner 混合）家族三方 Java 8 重编
- **THEN** `static class Stat` 嵌套声明呈现、`new Stat()` 源码拼写，运行与原 class 一致（Inner 分离路径不受影响）

#### Scenario: 既有族不变

- **WHEN** 输入为纯静态族（M1/M2）或纯非静态族
- **THEN** 前者与本变更前逐字一致；后者保持分离（不折叠）
