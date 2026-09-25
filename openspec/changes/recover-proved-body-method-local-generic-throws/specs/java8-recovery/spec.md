## ADDED Requirements

### Requirement: 空正文方法的局部泛型异常来源保留

对于有正文且完全证明无副作用的无参 `void` 方法，若其方法 `Signature` 声明一个合法的局部类型变量并在异常后缀引用同一变量，系统 SHALL 在类源码中同时保留方法类型参数和 `throws` 变量；重编后的泛型方法和异常反射结果 MUST 与已证原 class 一致。若方法体、变量作用域、物理异常擦除或 Java 源绑定证据不足，系统 MUST 局部拒绝整份泛型声明并保留物理成员，不得只发布 `<X>` 或只替换 `throws` 子串。预算与取消 MUST 保持现有操作停止及原子输出契约。

#### Scenario: 方法自有异常变量的空正文
- **WHEN** Java 8 顶层非泛型类的 `public <X extends Exception> void run() throws X {}` 具有同轮完整空正文证明，且无未证明的覆写与本类调用绑定
- **THEN** 输出同时含 `<X extends Exception>` 与 `throws X`，完整类的反射保留一个方法类型参数和异常变量 `X`，显式 `<RuntimeException>` 的强类型调用方在目标 Java 8 下重编并运行

#### Scenario: 不依赖调试信息
- **WHEN** 同一个正例分别以 `-g` 与 `-g:none` 编译
- **THEN** 两种 class-source 输出都保留相同的方法类型参数和泛型异常来源

#### Scenario: 不完整来源局部回退
- **WHEN** 异常变量未绑定、第一界与物理异常擦除矛盾，或正文/继承/调用绑定不能完整证明
- **THEN** 输出 MUST 保留方法的物理身份和物理异常声明，并可核对地说明拒绝，不得产生部分泛型方法头

#### Scenario: 类泛型头不进入首片
- **WHEN** 类本身声明 `Signature` 属性，即使其类头投影失败而交给方法的已发布类型变量作用域为空
- **THEN** 本切片 MUST 保留方法的物理声明，不得把空作用域误认作已证明的非泛型类

#### Scenario: 操作停止
- **WHEN** 正文候选或完整声明在预算耗尽或取消前无法交付
- **THEN** 操作 SHALL 报告对应停止状态，任何已交付源码不得含有部分 `<X>` 或 `throws X` 投影
