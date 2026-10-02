## ADDED Requirements

### Requirement: 直接静态成员类按嵌套声明折叠呈现

系统 SHALL 在外围类的 InnerClasses 直接行存在任意数量的源码可写静态成员类/接口时，将各子类完整恢复文本按行序嵌入外围文本（`static class/interface X …`），并在折叠作用域内把对这些成员的引用按源码拼写重写。分离家族呈现、匿名与枚举投影通道 SHALL 逐字不变；孙代嵌套、非静态成员类与单类输入 SHALL 保持既有行为。

#### Scenario: 多子静态家族折叠

- **WHEN** M1 形（五个直接静态成员：类/接口混合、兄弟继承）家族三方 Java 8 重编
- **THEN** 外围文本含各 `static` 嵌套声明、作用域内引用为源码拼写，`javac --release 8` 通过，运行与原 class 一致

#### Scenario: 既有通道与负例不变

- **WHEN** 输入为逐类分离输出、匿名/枚举投影、孙代嵌套或非静态成员类
- **THEN** 输出与本变更前逐字一致
