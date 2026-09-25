## ADDED Requirements

### Requirement: A method type variable shadows the same-named class variable in a proven generic declaration

Java 8 类声明与方法声明各自拥有同名类型变量时，方法的 `Signature` 中对该名称的使用 SHALL 按最近的方法作用域解析，并以方法自己的界验证物理参数、返回及异常擦除。已支持的静态直接参数返回方法 SHALL 保留该方法变量和其界，且类声明的变量及反射签名 MUST 保持独立。缺少完整证明时 SHALL 保留物理声明与可追源拒绝，不得把方法变量解释成类变量或从调用点推断泛型。

#### Scenario: Same name with Object bounds

- **WHEN** Java 8 类 `ShadowPlain<T>` 定义静态 `<T> T echo(T value)`，方法正文已证明直接返回参数
- **THEN** Jarde 完整类源码 SHALL 同时包含类级与方法级 `T`，原始强类型调用方对其重编、执行 MUST 与原 class 相同；类/方法泛型反射声明 MUST 分别存在

#### Scenario: Method bound differs from class bound

- **WHEN** 类变量 `T` 的第一界为 `Number`，同名静态方法变量 `T` 的第一界为 `CharSequence`，物理方法参数与返回都是 `CharSequence`
- **THEN** 方法 `T` SHALL 按 `CharSequence` 擦除并呈现为 `<T extends CharSequence> T echo(T)`；强类型调用方的 Java 8 重编及运行 MUST 成功，不得因同名误用 `Number` 或退化为非泛型 `CharSequence` 方法

#### Scenario: Invalid local scope or erasure remains refused

- **WHEN** 方法作用域内部重复声明变量、引用未绑定变量、界循环，或方法变量按自身界擦除后与物理 descriptor/Exceptions 不一致
- **THEN** 该方法的泛型声明 SHALL 被拒绝并保留物理事实和诊断；不得借同名类变量掩盖错误，也不得发布未证明的强类型方法头

#### Scenario: Other scopes and stops retain their contracts

- **WHEN** 类变量与方法变量名称不同、请求只选必要或全部证据、证明期间预算耗尽或取消
- **THEN** 不同名路径 SHALL 保持现有正确结果，完整请求的源码决定 SHALL 一致，停止请求 MUST 明确未完成且不得发布半个泛型声明
