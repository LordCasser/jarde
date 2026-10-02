## ADDED Requirements

### Requirement: 非静态成员类按嵌套声明折叠并消隐合成物

系统 SHALL 将非静态直接成员类按 `class Inner { … }` 嵌套声明折叠，并在折叠作用域内消隐 javac 合成物：`this$0` 构造首参与字段声明、`access$NNN` 直通访问桥（其调用位重写为限定字段访问）；构造调用按限定语法呈现（外围语境 `new Inner(args)`、显式限定 `qualifier.new Inner(args)`）。分离家族呈现、纯静态族与既有折叠通道 SHALL 逐字不变；非直通桥体与不可证的限定形 SHALL 保守呈现或登记。

#### Scenario: 非静态内部类折叠

- **WHEN** N1 形（Inner 捕获 base + access 桥 + this/限定两种构造语境）家族集三方 Java 8 重编
- **THEN** `class Inner` 嵌套呈现、合成物消隐、构造限定语法正确，`java -Xverify:all` 逐路径与原 class 一致（`10/7/13`）

#### Scenario: 分离与既有不变

- **WHEN** 输入为逐类分离输出、capture-ctor 家族或纯静态折叠族
- **THEN** 输出与本变更前逐字一致
